use clap::{Parser, Subcommand};

use crate::platforms::cli::{BlueskyCliArgs, MastodonCliArgs};

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

    /// Represents the time in seconds to pause between posts.
    #[arg(short = 's', long, default_value_t = 120)]
    pub post_pause_time: u64,

    /// Platform
    #[command(subcommand)]
    pub platform: Command,
}

/// Available Subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Command to start bot for the Bluesky platform.
    Bluesky(BlueskyCliArgs),
    /// Command to start bot for the Mastodon platform, also called the Fediverse.
    Mastodon(MastodonCliArgs),
}
