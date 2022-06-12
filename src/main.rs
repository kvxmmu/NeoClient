use {
    clap::{Parser, Subcommand},
    num_cpus::get as logical_cpu_number,
    tokio::runtime::Builder,

    client::{
        tcp::runner::*,
        compression::*,
        *,
    },

    anyhow::Result,

    std::{
        env::{var, set_var}
    }
};

#[derive(Debug, Parser)]
struct Args {
    /// Remote NeoGrok server, e.g. neogrok.someserver.su
    #[clap(long, short)]
    remote: String,

    /// NeoGrok server authorization magic
    #[clap(long, short)]
    magic: Option<String>,

    /// ZStandard compression level, from 1 to 10
    #[clap(long)]
    #[clap(default_value_t = 10)]
    compression_level: u8,

    /// Disable streaming synchronization with the server
    /// (Synchronizes only once by default)
    #[clap(long, short)]
    dont_sync: bool,

    /// ZStandard compression minimal profit
    /// this means packet will be sent compressed if
    /// After compression data was enshorted decreased by N percents
    /// e.g. --compression-profit 2.5
    /// Will be interpreted as 2.5 percents
    #[clap(long)]
    #[clap(default_value_t = 5.0)]
    compression_profit: f32,

    /// Network socket buffer size in bytes per proxy client
    /// for read
    #[clap(long)]
    #[clap(default_value_t = 4096)]
    pub buffer_read: usize,

    /// Number of workers for multithreaded runtime
    #[clap(long, short)]
    pub workers: Option<usize>,

    /// Network socket buffer size in bytes per proxy client
    /// for write
    #[clap(long)]
    #[clap(default_value_t = 512)]
    pub buffer_write: usize,

    /// Minimal threshold for compression
    /// This means that client will try compress payload
    /// if its size greater or equal to N bytes
    #[clap(long)]
    #[clap(default_value_t = 50)]
    compression_threshold: usize,

    #[clap(subcommand)]
    subcommands: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Create TCP proxy
    Tcp {
        /// Local server, e.g. localhost:25565
        local: String,

        /// Remote server port to bind
        #[clap(long, short)]
        #[clap(default_value_t = 0)]
        port: u16,
    }
}

fn main() -> Result<()> {
    let mut args = Args::parse();
    if !args.remote.contains(":") {
        args.remote.push_str(":6567");
    }

    if var("NEOGROK_LOG").is_err() {
        set_var("NEOGROK_LOG", "info");
    }

    pretty_env_logger::init_custom_env("NEOGROK_LOG");

    let workers = args.workers.unwrap_or(logical_cpu_number());
    let rt = Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("NeoGrok client worker")
                    .worker_threads(workers)
                    .build()
                    .expect("Failed to create tokio runtime");
    let compression = Compression {
        level: args.compression_level as i32,
        profit: args.compression_profit / 100.0,
        threshold: args.compression_threshold
    };
    let buffer_size = BufferSize {
        read: args.buffer_read,
        write: 1
    };

    match args.subcommands {
        Subcommands::Tcp { local, port } => {
            rt.block_on(run_tcp_client(
                local,
                args.remote,
                args.magic,
                port,
                buffer_size,
                compression,
                !args.dont_sync
            ))?;
        }
    }

    Ok(())
}
