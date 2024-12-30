use crate::bluesky::BlueSkyClient;
use crate::cli::CliArgs;
use clap::Parser;
use infrastructure::RedisService;
use log::{error, info, warn};
use post::NewsPost;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

mod bluesky;
mod cli;

//noinspection DuplicatedCode
/// Sets up a signal handler in a separate thread to handle SIGINT and SIGTERM signals.
fn setup_graceful_shutdown(running: &Arc<AtomicBool>) {
    let r = running.clone();
    thread::spawn(move || {
        let signals = Signals::new([SIGINT, SIGTERM]);
        match signals {
            Ok(mut signal_info) => {
                if signal_info.forever().next().is_some() {
                    r.store(false, Ordering::SeqCst);
                }
            }
            Err(error) => {
                error!("Failed to setup signal handler: {error}")
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::parse();
    info!("Starting the program");

    // Graceful shutdown.
    let running = Arc::new(AtomicBool::new(true));
    setup_graceful_shutdown(&running);

    // Redis setup
    let mut redis_service = RedisService::new(&args.redis_connection_string).await;

    // Create consumer group for stream.
    let result = redis_service
        .create_group(&args.redis_stream_name, &args.redis_consumer_group, 0)
        .await;
    if let Err(err) = result {
        warn!("{}", err);
    }

    let mut bluesky_client =
        BlueSkyClient::new(&args.bluesky_handle, &args.bluesky_password).await?;

    // Read from stream
    while running.load(Ordering::SeqCst) {
        match redis_service
            .read_stream::<NewsPost>(
                &args.redis_stream_name,
                &args.redis_consumer_group,
                &args.redis_consumer_name,
                5000,
            )
            .await
        {
            Ok(post) => {
                let mut data: bluesky::atproto::ATProtoRepoCreateRecord = post.clone().into();
                data.repo = args.bluesky_handle.clone();
                let json = serde_json::to_string(&data);
                match json {
                    Ok(json) => {
                        if let Err(err) = bluesky_client.post(json).await {
                            error!("failed to post: {post:?} {err}")
                        }
                    }
                    Err(err) => {
                        error!("failed to convert post to json: {post:?} {err}")
                    }
                }
            }
            Err(err) => {
                error!("error reading stream: {err}")
            }
        }
    }

    info!("Stopping the program");
    Ok(())
}
