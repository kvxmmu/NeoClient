use {
    tokio::{
        sync::{ mpsc::{ UnboundedReceiver
                      , UnboundedSender } },
        net::TcpStream,
        io::{ AsyncReadExt
            , AsyncWriteExt },
        select,
    },

    crate::{
        tcp::proxy::{
            frame::*,
        }
    },

    idpool::ClientId,
};

pub async fn run_tcp_proxy_listener(
    local: String,

    mut slave: UnboundedReceiver<SlaveFrame>,
    master: UnboundedSender<MasterFrame>,

    id: ClientId,
    buffer_size: usize,
) {
    log::debug!("ID#{} is connecting to the {}...", id, local);

    let mut stream = match TcpStream::connect(&local).await {
        Ok(stream) => stream,
        Err(e) => {
            log::error!("Failed to connect to the {}: {}", local, e);

            master.send(MasterFrame::Disconnected { id })
                  .unwrap_or_default();
            return;
        },
    };
    let mut buffer = vec![0; buffer_size];
    let mut forcibly = false;

    log::info!("ID#{} is connected to the {}", id, local);

    loop {
        select! {
            read = stream.read(&mut buffer) => {
                let read = read.unwrap_or(0);
                if read == 0 {
                    break;
                }

                let frame = MasterFrame::Forward { id, buffer: Vec::from(&buffer[..read]) };
                if master.send(frame).is_err() {
                    break;
                }
            },

            slave_frame = slave.recv() => if let Some(frame) = slave_frame {
                match frame {
                    SlaveFrame::Forward { buffer } => {
                        if stream.write_all(&buffer).await.is_err() {
                            break;
                        }
                    },

                    SlaveFrame::ForceDisconnect => {
                        forcibly = true;
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    if !forcibly {
        master.send(MasterFrame::Disconnected { id })
              .unwrap_or_default();
    }
    log::info!("ID#{} disconnected", id);
}
