use crate::cli::CliArgs;
use crate::scrapper::gfourmedia::G4Media;
use crate::scrapper::WebScrapperEngine;
use clap::Parser;
use clokwerk::{AsyncScheduler, Interval, TimeUnits};
use infrastructure::RedisService;
use log::{debug, error, info};
use post::NewsPost;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::task::JoinHandle;

mod cli;
mod scrapper;

/// Runs the scheduler in a separated thread.
///
/// If CTRL+C is pressed it will set `running` to `true`.
fn run_scheduler(mut scheduler: AsyncScheduler, running: Arc<AtomicBool>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if !running.load(Ordering::SeqCst) {
                debug!("Used requested shutdown.");
                break;
            }
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
}

/// Runs the scraping job at the specified interval.
fn run_scrapping_job(scheduler: &mut AsyncScheduler, tx: Sender<NewsPost>, interval: Interval) {
    scheduler.every(interval).run(move || {
        let tx = tx.clone();
        async move {
            let posts = WebScrapperEngine::get_posts(G4Media::default())
                .await
                .expect("failed to get posts");
            posts.iter().filter(|p| p.is_complete()).for_each(|p| {
                let result = tx.send(p.clone());
                if result.is_err() {
                    error!("Failed to send post {:?} to the channel", p)
                }
            });
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::parse();
    info!("Starting the program");

    // Redis setup
    let mut redis_service =
        RedisService::new(&args.redis_connection_string, &args.redis_stream_name).await;

    // Scheduler setup
    let mut scheduler = AsyncScheduler::new();

    // Channel for synchronizing the scrapper and the bot
    let (tx, rx): (Sender<NewsPost>, Receiver<NewsPost>) = mpsc::channel();

    // Graceful shutdown.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    run_scrapping_job(&mut scheduler, tx, args.scrape_interval_minutes.minutes());

    // Run the scheduler in a separate thread.
    let handle = run_scheduler(scheduler, running.clone());

    for news_post in rx.iter() {
        if !running.load(Ordering::SeqCst) {
            debug!("Used requested shutdown.");
            break;
        }
        info!("Received post {:?}", news_post);
        if news_post.is_complete() {
            let title = news_post.title.clone().unwrap();
            if !redis_service.is_post_seen(&title).await {
                let published = redis_service.publish(&news_post).await;
                if published {
                    info!("Published {:?}", news_post);
                    redis_service.mark_post_seen(&title, 60 * 60 * 24 * 3).await;
                }
            };
        }
    }

    info!("Stopped the program");

    handle.await?;

    Ok(())
}
