use clap::Args;

/// X CLI command arguments
#[derive(Args, Debug)]
pub struct XCliArgs {
    /// The X Api bearer code.
    #[arg(short = 'c', long)]
    pub bearer_code: String,
}
