use {
    tokio::{
        net::TcpStream,
        io::{AsyncWriteExt, AsyncReadExt},

        sync::{
            mpsc::{UnboundedSender, UnboundedReceiver},
        }
    },

    crate::{
        ipc::*,
    },

    neoproto::prelude::*,
};

pub async fn run_proxy(
    mut stream: TcpStream,

    master: UnboundedSender<MasterFrame>,
    mut rx: UnboundedReceiver<SlaveFrame>,

    id: ClientId,
) {
    let mut send_disconnected = true;
    let mut buffer = [0; 4096];

    loop {
        tokio::select! {
            request = rx.recv() => {
                if request.is_none() {
                    log::debug!("ID#{} is disconnected (no senders left)", id);
                    break;
                }
                let request = request.unwrap();

                match request {
                    SlaveFrame::Forward(buf, length) => {
                        if stream.write_all(&buf[..length]).await.is_err() {
                            break;
                        }

                        log::debug!("{}bytes sent to the ID#{}", length, id);
                    },

                    SlaveFrame::ForceDisconnect => {
                        send_disconnected = false;
                        break;
                    }
                }
            },

            received = stream.read(&mut buffer) => {
                if received.as_ref().unwrap_or(&0) == &0 {
                    break;
                }

                let received = received.unwrap();
                if master.send(MasterFrame::Forward(id, Vec::from(&buffer[..received]))).is_err() {
                    log::debug!("ID#{} can't communicate with the server", id);
                    send_disconnected = false;

                    break;
                }

                log::debug!("ID#{} sent {}bytes to the master", id, received);
            },
        }
    }

    if send_disconnected {
        master.send(MasterFrame::Disconnected(
            id
        ))
         .unwrap_or_default();
    }
}
