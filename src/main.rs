use {
    client::prelude::*,
    clap::Parser
};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct ClientArgs {
    #[clap(short, long)]
    remote: String,

    #[clap(short, long)]
    local: String,

    #[clap(short, long, default_value_t = 0)]
    port: u16,

    #[clap(short, long)]
    magic: Option<String>
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = ClientArgs::parse();
    if args.remote.find(':').is_none() {
        args.remote += ":6567";
    }

    run_client(&args.remote, &args.local,
                args.port, args.magic).await
}
