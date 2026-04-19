use clap::Args;

/// X CLI command arguments
#[derive(Args, Debug)]
pub struct XCliArgs {
    /// The consumer key for Oauth1 flow.
    #[arg(short = 'c', long)]
    pub consumer_key: String,

    /// The consumer secret for Oauth1 flow.
    #[arg(short = 'c', long)]
    pub consumer_secret: String,

    /// The access token.
    #[arg(short = 'c', long)]
    pub access_token: String,

    /// The access token secret.
    #[arg(short = 'c', long)]
    pub access_token_secret: String,
}
