use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Redis host
    #[arg(short, long)]
    pub redis_connection_string: String,

    /// Redis stream name
    #[arg(short = 't', long)]
    pub redis_stream_name: String,

    /// Redis consumer group name
    #[arg(short = 'c', long)]
    pub redis_consumer_group: String,

    /// The current consumer name
    #[arg(short = 'n', long)]
    pub redis_consumer_name: String,
}
