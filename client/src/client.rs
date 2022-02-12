use {
    tokio::{
        net::TcpStream,

        sync::{
            mpsc::{
                unbounded_channel,
            }
        }
    },

    crate::{
        reader::*,
        frame::*,
        session::*,

        client_impl::*,

        net::{self, ok_or_exit},
        ipc::*,
    },

    std::{
        process::exit,
        time::Duration,
    }
};

pub async fn run_app(
    compression_profit: f32,
    compression_level: i32,

    remote: String,
    local: String,

    magic: Option<String>,
    request_port: u16,
    timeout: Duration,
) -> std::io::Result<()> {
    let mut stream = match TcpStream::connect(&remote).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Can't connect to the {:?}: {}", remote, e);
            exit(1);
        }
    };
    let mut session = Session::new(
        magic,
        compression_profit,
        compression_level,
        request_port,
        timeout
    );

    let (tx, mut rx) = unbounded_channel::<MasterFrame>();

    ok_or_exit(net::send_ping(&mut stream).await);
    loop {
        tokio::select! {
            packed = net::read_type(&mut stream) => {
                let (type_, flags) = ok_or_exit(packed);
                let frame = ok_or_exit(read_frame(type_, flags, &mut stream).await);

                ok_or_exit(
                    handle_server(
                        &mut stream,
                        &mut session,
                        frame,
                        &tx,
                        &local,
                    ).await
                );
            },

            master_req = rx.recv() => {
                if master_req.is_none() {
                    log::error!("Can't fetch data from queue");
                    exit(1);
                }

                ok_or_exit(
                    handle_server(
                        &mut stream,
                        &mut session,
                        Frame::HandleSlave(master_req.unwrap()),
                        &tx,
                        &local,
                    ).await
                );
            },
        }

        
    }
}
