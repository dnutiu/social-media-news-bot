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

    /// The scraping interval in minutes
    #[arg(short, long, default_value_t = 60)]
    pub scrape_interval_minutes: u32,

    /// Scrape the maximum posts.
    #[arg(short = 'm', long, default_value_t = 100)]
    pub max_posts_per_run: u64,
}
