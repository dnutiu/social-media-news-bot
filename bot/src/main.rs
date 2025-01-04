use crate::bluesky::BlueSkyClient;
use crate::cli::{CliArgs, Command};
use crate::mastodon::api::PostStatusRequest;
use crate::mastodon::MastodonClient;
use anyhow::anyhow;
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
mod mastodon;

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

    // Create consumer group for stream.
    let result = redis_service
        .create_group(&args.redis_stream_name, &args.redis_consumer_group, 0)
        .await;
    if let Err(err) = result {
        warn!("{}", err);
    }

    match args.platform {
        Command::Bluesky(bluesky) => {
            let mut bluesky_client =
                BlueSkyClient::new(&bluesky.bluesky_handle, &bluesky.bluesky_password).await?;

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
                        let mut data: bluesky::atproto::ATProtoRepoCreateRecord =
                            post.clone().into();
                        data.repo = bluesky.bluesky_handle.clone();

                        if let Some(image_link) = post.image.clone() {
                            let result =
                                add_image_to_post(&mut bluesky_client, &image_link, &mut data)
                                    .await;
                            if let Err(err) = result {
                                warn!("Failed to upload image: {err}")
                            }
                        }
                        let json = serde_json::to_string(&data);
                        match json {
                            Ok(json) => {
                                if let Err(err) = bluesky_client.post(json).await {
                                    error!("failed to post: {post:?} {err}")
                                } else {
                                    info!("Published a post! 🦀")
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
        }
        Command::Mastodon(mastodon) => {
            let mut mastodon_client = MastodonClient::new(mastodon.access_token);
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
                        // Step1: Upload image to Mastodon
                        let media_response = if post.image.is_some() {
                            let response = mastodon_client
                                .upload_media_by_url(post.image.clone().unwrap().as_str())
                                .await;

                            match response {
                                Ok(response) => Ok(response),
                                Err(err) => Err(anyhow!("failed to upload image: {err}")),
                            }
                        } else {
                            Err(anyhow!("No image exists on post."))
                        };

                        // Step2: Post to Mastodon.
                        let mut status: PostStatusRequest = post.into();
                        match media_response {
                            Ok(response) => {
                                status.media_ids.push(response.id);
                            }
                            Err(err) => {
                                error!("Error uploading image: {err}")
                            }
                        }
                        let response = mastodon_client.post_status(status).await;
                        match response {
                            Ok(response) => {
                                info!("Posted tooth on Mastodon! {response:?}")
                            }
                            Err(err) => {
                                error!("Failed to post toot on Mastodon: {err}")
                            }
                        }
                    }
                    Err(err) => {
                        error!("error reading stream: {err}")
                    }
                }
            }
        }
    }

    info!("Stopping the program");
    Ok(())
}
