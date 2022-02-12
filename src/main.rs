use {
    clap::Parser,
    client::prelude::*,
    std::time::Duration,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ConnectArgs {
    // Remote NeoGrok address in domain[:port] format
    #[clap(short, long)]
    remote: String,

    /// Local server address in domain[:port] format
    #[clap(short, long)]
    local: String,

    /// Port to be requested
    #[clap(short, long, default_value_t = 0)]
    port: u16,

    /// Local server address
    #[clap(short, long)]
    magic: Option<String>,

    /// Minimal compression profit in percents(difference between original and compressed size in percent representation)
    #[clap(long, default_value_t = 5.0)]
    compression_profit: f32,

    /// Overwritten compression level (0 - for synchronization with server)
    #[clap(long, default_value_t = 0)]
    compression_level: i32,

    /// Local server connect timeout
    #[clap(short, long, default_value_t = 5)]
    timeout: u64,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let mut args = ConnectArgs::parse();
    if !args.remote.contains(':') {
        args.remote.push_str(":6567");
    }

    pretty_env_logger::init();
    log::debug!("Running in debug mode");

    run_app(
        args.compression_profit,
        args.compression_level,
        args.remote,
        args.local,
        args.magic,
        args.port,
        Duration::from_secs(args.timeout)
    ).await
}
