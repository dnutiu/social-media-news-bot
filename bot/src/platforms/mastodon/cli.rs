use clap::Args;

/// Mastodon command arguments
#[derive(Args, Debug)]
pub struct MastodonCliArgs {
    /// The Bluesky bot user's handle.
    #[arg(short = 'a', long)]
    pub access_token: String,
}