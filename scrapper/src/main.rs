use crate::cli::CliArgs;
use crate::scrapper::{ScrappableWebPage, WebScrapperEngine};
use crate::targets::{GFourMedia, HotNews};
use clap::Parser;
use clokwerk::{AsyncScheduler, Interval, TimeUnits};
use infrastructure::RedisService;
use log::{debug, error, info};
use post::NewsPost;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

mod cli;
mod scrapper;
mod targets;

/// Runs the scheduler in a background task until shutdown is requested.
fn run_scheduler(
    mut scheduler: AsyncScheduler,
    mut shutdown_rx: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        debug!("User requested shutdown.");
                        break;
                    }
                }
                _ = ticker.tick() => {
                    scheduler.run_pending().await;
                }
            }
        }
    })
}

async fn scrape_and_send<S>(
    engine: &WebScrapperEngine,
    source: S,
    tx: &mpsc::Sender<NewsPost>,
    max_posts: u64,
) where
    S: ScrappableWebPage + Default,
{
    match engine.get_posts(source).await {
        Ok(posts) => {
            for p in posts
                .iter()
                .filter(|p| p.is_complete())
                .take(max_posts as usize)
            {
                if tx.send(p.clone()).await.is_err() {
                    error!("Receiver has been dropped. Could not send post: {:?}", p);
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to get posts for source {}: {:?}",
                std::any::type_name::<S>(),
                e
            );
        }
    }
    info!("Scrape job finished for {}", std::any::type_name::<S>())
}

/// Runs the scraping job at the specified interval.
fn run_scrapping_job(
    scheduler: &mut AsyncScheduler,
    tx: mpsc::Sender<NewsPost>,
    interval: Interval,
    max_posts: u64,
) {
    scheduler.every(interval).run(move || {
        let tx = tx.clone();
        info!("Running the scrapping job.");
        async move {
            let engine: WebScrapperEngine = WebScrapperEngine::default();

            tokio::join!(
                scrape_and_send::<HotNews>(&engine, HotNews::default(), &tx, max_posts),
                scrape_and_send::<GFourMedia>(&engine, GFourMedia::default(), &tx, max_posts)
            );
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::parse();
    info!("Starting the program");

    let mut redis_service = RedisService::new(&args.redis_connection_string).await;
    let mut scheduler = AsyncScheduler::new();
    let (tx, mut rx) = mpsc::channel::<NewsPost>(256);

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    tokio::spawn({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                error!("Failed to listen for shutdown signal: {}", e);
            } else {
                info!("Shutdown signal received");
                let _ = shutdown_tx.send(true);
            }
        }
    });

    run_scrapping_job(
        &mut scheduler,
        tx,
        args.scrape_interval_minutes.minutes(),
        args.max_posts_per_run,
    );

    let handle = run_scheduler(scheduler, shutdown_rx.clone());
    let mut main_shutdown_rx = shutdown_rx;

    loop {
        tokio::select! {
            _ = main_shutdown_rx.changed() => {
                if *main_shutdown_rx.borrow() {
                    debug!("User requested shutdown.");
                    break;
                }
            }
            maybe_post = rx.recv() => {
                let Some(news_post) = maybe_post else {
                    debug!("Scrape channel closed.");
                    break;
                };

                info!("Received post {:?}", news_post);
                if news_post.is_complete() {
                    let title = news_post.title.clone().unwrap();
                    let unique_post_key = format!("{}-{}", &args.redis_stream_name, &title);
                    let digest = format!("{:x}", md5::compute(unique_post_key));
                    if !redis_service.is_key_flagged(&digest).await {
                        let published = redis_service
                            .publish(&args.redis_stream_name, &news_post)
                            .await;
                        if published {
                            info!("Published {:?}", news_post);
                            redis_service.flag_key(&digest, 60 * 60 * 24 * 90).await;
                        }
                    };
                }
            }
        }
    }

    info!("Stopped the program");

    let _ = shutdown_tx.send(true);
    handle.await?;

    Ok(())
}
