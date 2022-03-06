use {
    tokio::{
        net::TcpStream,

        sync::{
            mpsc::{UnboundedSender, unbounded_channel},
        },

        time::timeout,
    },

    crate::{
        frame::*,
        session::*,
        proxy_impl::*,

        net,
        ipc::*,
    },

    std::{
        process::exit,
    },

    neoproto::prelude::*,
};

pub async fn handle_server(
    stream: &mut TcpStream,
    session: &mut Session,

    frame: Frame,
    master: &UnboundedSender<MasterFrame>,

    local: &str,
) -> std::io::Result<()> {
    match frame {
        // Proxy functionality

        Frame::Connected(id) => {
            let connected = match timeout(session.timeout, TcpStream::connect(local)).await {
                Ok(Ok(l)) => l,
                Ok(Err(e)) => {
                    log::error!("Failed to connect to {}: {}", local, e);
                    return net::send_disconnect(stream, id).await;
                },

                Err(_) => {
                    log::error!("Failed to connect to {}: timed out ({:?})", local, session.timeout);
                    return net::send_disconnect(stream, id).await;
                }
            };
            log::info!("ID#{} connected to the {}", id, local);

            let master = master.clone();
            let (tx, rx) = unbounded_channel();
            session.channels.add_client(id, tx);

            tokio::spawn(async move {
                run_proxy(
                    connected,
                    master,
                    rx,
                    id
                ).await;
            });
        },

        Frame::Packet(id, buf, size) => {
            if !session.channels.send(id, SlaveFrame::Forward(buf, size)) {
                log::error!("Failed to forward packet to the ID#{} ({})", id, local);
                return net::send_disconnect(stream, id).await;
            }
        },

        Frame::Disconnected(id) => {
            session.channels.send(
                id, SlaveFrame::ForceDisconnect,
            );
            log::info!("ID#{} is disconnected from the {}", local, id);
            session.channels.remove_client(id);
        },

        // State functionality

        Frame::Pong(pong) => {
            log::info!("Connected to the {}", pong);
            if session.compression_level == 0 {
                net::send_sync_info(stream).await?;
            } else {
                // There's no need in client synchronization: compression level is overwritten
                log::info!("Compression level is scompression_levelt to {} (client settings)", session.compression_level);
                return jump_state(session, stream).await;
            }
        },

        Frame::Synchronize(_initial_rights, _magic_rights, compression_level) => {
            session.compression_level = compression_level;
            log::info!("Compression level is set to {} (server settings)", compression_level);

            return jump_state(session, stream).await;
        },

        Frame::UpdateRights(rights) => {
            if (rights & rights_flags::CREATE_SERVER) == 0 {
                log::error!("Can't create server: no rights granted");
                exit(1);
            }

            log::info!("Updated rights: {}", UserMeta::rights_to_string(rights));
            if session.make_sent() {
                net::send_port(stream, session.request_port).await?;
            }
        },

        Frame::CretedTCP(port) => {
            log::info!("Server created on port {}", port);
        },

        Frame::Error(error_code) => {
            log::error!("Received error 0x{:x}: {}", error_code, error_to_string(error_code));
            if error_code == protocol::ACCESS_DENIED {
                exit(1);
            }
        },

        Frame::HandleSlave(request) => {
            match request {
                MasterFrame::Forward(id, buf) => {
                    return net::forward_packet(
                        stream,
                        id,
                        buf.len() as u16,
                        buf,
                        session.compression_profit,
                        session.compression_level
                    ).await;
                },

                MasterFrame::Disconnected(id) => {
                    session.channels.remove_client(id);
                    log::info!("ID#{} is disconnected from the server", id);
                }
            }
        },

        _ => {
            log::debug!("Unhandled frame {:?}", frame);
        }
    }

    Ok(())
}

async fn jump_state(
    session: &mut Session,
    stream: &mut TcpStream
) -> std::io::Result<()> {
    if session.magic.is_none() {
        net::send_port(stream, session.request_port).await
    } else {
        net::send_magic_auth(stream, session.magic.as_ref().unwrap()).await
    }
}
