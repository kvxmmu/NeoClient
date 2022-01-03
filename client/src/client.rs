use {
    tokio::{
        net::TcpStream,

        io::{AsyncWriteExt, AsyncReadExt},
        sync::mpsc::{Sender, Receiver, channel}
    },

    std::{
        collections::HashMap
    },

    crate::{
        wrapper::*, types::*,
        compressor::*
    },

    neoproto::prelude::*
};

pub async fn handle_pkt(local: &str,
                        port: u16, magic: &Option<String>,
                        
                        tx: &Sender<MainRequest>,
                    
                        type_: PacketType, flags: Flags,
                        stream: &mut TcpStream, clients: &mut HashMap<ClientId, Sender<ConnectorRequest>>) -> std::io::Result<()> {
    match type_ {
        protocol::PING => {
            println!(">> Received server name {}", stream.read_string().await);

            if magic.is_some() {
                stream.authorize(magic.as_ref().unwrap()).await;
            } else {
                stream.request_server(port).await;
            }
        },

        protocol::CONNECT => {
            let client_id = stream.read_client_id(flags).await;
            println!(">> Connected ID#{}", client_id);

            let client_stream = match TcpStream::connect(local).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("<< Failed to connect to {}: {}", local, e.to_string());
                    std::process::exit(1);
                }
            };

            let tx = tx.clone();
            let (rtx, rx) = channel(32);

            tokio::spawn(async move {
                client_connector(&tx, rx, client_stream, client_id).await.unwrap_or_default();
                tx.send(MainRequest::EndOfLife(client_id)).await.unwrap_or_default();
            });

            clients.insert(client_id, rtx);
        },

        protocol::PACKET => {
            let length = stream.read_length(flags).await as usize;
            let client_id = stream.read_client_id(flags).await;
            let mut buf = vec![0; length];

            ok_or_exit(stream.read_exact(&mut buf).await);

            match clients.get(&client_id) {
                Some(sender) => {
                    if (flags & protocol::COMPRESSED) != 0 {
                        let (decompressed, dec_length) = decompress_data(&buf);
                        if dec_length == 0 {
                            eprintln!("<< Can't decompress packet for ID#{}", client_id);
                            std::process::exit(1);
                        }

                        sender.send(
                            ConnectorRequest::Forward(Vec::from(&decompressed[..dec_length]))
                        ).await.unwrap_or_default();
                    } else {
                        sender.send(
                            ConnectorRequest::Forward(buf)
                        ).await.unwrap_or_default();
                    }

                },

                None => {
                    eprintln!("<< Sent packet to unknown ID#{}", client_id);
                }
            }
        },

        protocol::DISCONNECT => {
            let client_id = stream.read_client_id(flags).await;
            match clients.get(&client_id) {
                Some(sender) => {
                    sender.send(ConnectorRequest::Disconnect).await.unwrap_or_default();
                },

                None => {
                    eprintln!("<< Disconnected unknown ID#{}", client_id);
                }
            }
        },

        protocol::ERROR => {
            let error = stream.read_error().await;

            println!(">> Received error 0x{:x}", error);
        },

        protocol::RIGHTS_UPDATE => {
            let flags = stream.read_rights().await;
            println!(">> Received rights: {}", right::to_string(flags));

            stream.request_server(port).await;
        },

        protocol::SERVER => {
            let listening_port = stream.read_port().await;

            println!(">> Listening on port {}", listening_port);
        },

        _ => {
            println!("<< Unknown command 0x{:x} sent", type_);
        }
    }

    Ok(())
}

pub async fn run_client(remote: &str, local: &str,
                        port: u16, magic: Option<String>) -> std::io::Result<()> {
    println!(">> Connecting to {}", remote);
    let mut stream = match TcpStream::connect(remote).await {
        Ok(r) => r,
        Err(e) => {
            println!("<< Failed to connect to {}", remote);
            return Err(e);
        }
    };

    let mut map: HashMap<ClientId, Sender<ConnectorRequest>> = Default::default();
    let (tx, mut rx) = channel::<MainRequest>(32);

    stream.request_ping().await;
    loop {
        tokio::select! {
            request = rx.recv() => {
                if request.is_none() {
                    println!("<< Failed to recv request from channel");
                    return Ok(());
                }

                let request = request.unwrap();
                match request {
                    MainRequest::Forward(id, buf, flags) => {
                        stream.write_packet_header(id, buf.len() as u16,
                                                   flags).await;
                        ok_or_exit(stream.write_all(&buf).await);
                    },

                    MainRequest::EndOfLife(id) => {
                        map.remove(&id);
                        println!("<< Disconnected ID#{}", id);
                    },

                    MainRequest::Disconnect(id) => {
                        stream.write_disconnect(id).await;
                    }
                }
            },

            type_ = stream.read_type() => {
                handle_pkt(local,
                           port, &magic,

                           &tx,

                           type_.0, type_.1, &mut stream,
                           &mut map).await?;
            }
        }
    }
}

pub async fn client_connector(tx: &Sender<MainRequest>, mut rx: Receiver<ConnectorRequest>,
                              mut stream: TcpStream, id: ClientId) -> std::io::Result<()> {
    let mut buf = [0; 4096];
    loop {
        tokio::select! {
            request = rx.recv() => {
                if request.is_none() { break; }
                let request = request.unwrap();

                match request {
                    ConnectorRequest::Forward(buf) => {
                        stream.write_all(&buf).await?;
                    },

                    ConnectorRequest::Disconnect => {
                        break;
                    }
                }
            },

            received = stream.read(&mut buf) => {
                if received.as_ref().unwrap_or(&0) == &0 {
                    tx.send(MainRequest::Disconnect(id)).await.unwrap_or_default();
                    break;
                }

                let received = received.unwrap();
                let (compressed, length) = compress_data(&buf[..received], 7,
                                                         5.0);

                if length == 0 {
                    tx.send(
                        MainRequest::Forward(id, Vec::from(&buf[..received]), 0)
                    ).await.unwrap_or_default();
                } else {
                    tx.send(
                        MainRequest::Forward(id, Vec::from(&compressed[..length]), protocol::COMPRESSED)
                    ).await.unwrap_or_default();
                }
            }
        }
    }

    Ok(())
}
