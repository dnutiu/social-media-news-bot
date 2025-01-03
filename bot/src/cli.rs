use clap::{Args, Parser, Subcommand};

/// Bluesky Command Arguments
#[derive(Args, Debug)]
pub struct BlueskyCommand {
    /// The Bluesky bot user's handle.
    #[arg(short = 'u', long)]
    pub bluesky_handle: String,

    /// The Bluesky bot user's password.
    #[arg(short = 'p', long)]
    pub bluesky_password: String,
}

#[derive(Parser, Debug)]
#[command(version, about = "Social media posting bot.", long_about = None)]
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

    /// Platform
    #[command(subcommand)]
    pub platform: Command,
}

/// Available Subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Post on bluesky platform.
    Bluesky(BlueskyCommand),
    /// Post on Mastodon, the FediVerse
    Mastodon,
}
