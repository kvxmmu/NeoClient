use {
    clap::{Parser, Subcommand},
    client::{
        runner::tcp::*,

        compression::*,
        buffer::*,
    },
    num_cpus::get as logical_cpu_number,
    tokio::runtime::Builder
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

fn main() {
    let mut args = Args::parse();
    if !args.remote.contains(":") {
        args.remote.push_str(":6567");
    }

    let compression = Compression {
        profit: args.compression_profit,
        level: args.compression_level as u32,
        threshold: args.compression_threshold,
    };
    let buffer_size = BufferSize {
        read: args.buffer_read,
        write: args.buffer_write
    };

    println!("{:#?}", args);
}
