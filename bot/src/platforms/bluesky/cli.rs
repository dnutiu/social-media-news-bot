use clap::Args;

/// Bluesky command arguments
#[derive(Args, Debug)]
pub struct BlueskyCliArgs {
    /// The Bluesky bot user's handle.
    #[arg(short = 'u', long)]
    pub bluesky_handle: String,

    /// The Bluesky bot user's password.
    #[arg(short = 'p', long)]
    pub bluesky_password: String,
}
