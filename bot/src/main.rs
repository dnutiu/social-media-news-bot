use crate::bluesky::BlueSkyClient;
use crate::cli::{CliArgs, Command};
use crate::mastodon::MastodonClient;
use clap::Parser;
use infrastructure::RedisService;
use log::{error, info, warn};
use post::{NewsPost, Publisher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

mod bluesky;
mod cli;
mod mastodon;

//noinspection DuplicatedCode
/// Sets up a signal handler in a separate thread to handle SIGINT and SIGTERM signals.
fn setup_graceful_shutdown(running: &Arc<AtomicBool>) {
    let running = running.clone();
    thread::spawn(async move || {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed to listen for shutdown signal: {}", e);
        } else {
            info!("Shutdown signal received");
            running.store(false, Ordering::SeqCst);
        }
    });
}

/// Embeds an image to a post.
async fn add_image_to_post(
    client: &mut BlueSkyClient,
    image_url: &str,
    record: &mut bluesky::atproto::ATProtoRepoCreateRecord,
) -> Result<(), anyhow::Error> {
    let thumb = client.upload_image_by_url(image_url).await?;
    record.record.embed.as_mut().unwrap().external.thumb = Some(thumb.blob);

    Ok(())
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

    // Create a consumer group for stream.
    let result = redis_service
        .create_group(&args.redis_stream_name, &args.redis_consumer_group, 0)
        .await;
    if let Err(err) = result {
        warn!("{}", err);
    }

    let mut publisher_client: Box<dyn Publisher> = match args.platform {
        Command::Bluesky(bluesky) => {
            Box::new(BlueSkyClient::new(&bluesky.bluesky_handle, &bluesky.bluesky_password).await?)
        }
        Command::Mastodon(mastodon) => {
            Box::new(MastodonClient::new(mastodon.access_token))
        }
    };

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
                match publisher_client.publish_post(post.clone()).await {
                    Ok(_) => {}
                    Err(_) => {
                        error!("Failed to publish post: {post:?}");
                    }
                }

                // Sleep to avoid overwhelming service.
                tokio::time::sleep(time::Duration::from_secs(args.post_pause_time)).await
            }
            Err(err) => {
                error!("error reading stream: {err}")
            }
        }
    }

    info!("Stopping the program");
    Ok(())
}
