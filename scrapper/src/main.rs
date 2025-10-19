use crate::cli::CliArgs;
use crate::scrapper::gfourmedia::G4Media;
use crate::scrapper::hotnews::HotNews;
use crate::scrapper::{ScrappableWebPage, WebScrapperEngine};
use clap::Parser;
use clokwerk::{AsyncScheduler, Interval, TimeUnits};
use infrastructure::RedisService;
use log::{debug, error, info};
use post::NewsPost;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread;
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

// Helper function to handle the logic for a single scrape source
async fn scrape_and_send<S>(source: S, tx: &Sender<NewsPost>)
where
    S: ScrappableWebPage + Default,
{
    match WebScrapperEngine::get_posts(source).await {
        Ok(posts) => {
            for p in posts.iter().filter(|p| p.is_complete()) {
                // Log an error if the channel is closed, but don't panic
                if tx.send(p.clone()).is_err() {
                    error!("Receiver has been dropped. Could not send post: {:?}", p);
                }
            }
        }
        Err(e) => {
            // Log the error from the scraping itself
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
fn run_scrapping_job(scheduler: &mut AsyncScheduler, tx: Sender<NewsPost>, interval: Interval) {
    scheduler.every(interval).run(move || {
        let tx = tx.clone();
        info!("Running the scrapping job.");
        async move {
            // Run scrapping jobs concurrently.
            tokio::join!(
                scrape_and_send::<G4Media>(G4Media::default(), &tx),
                scrape_and_send::<HotNews>(HotNews::default(), &tx)
            );
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::parse();
    info!("Starting the program");

    // Redis setup
    let mut redis_service = RedisService::new(&args.redis_connection_string).await;

    // Scheduler setup
    let mut scheduler = AsyncScheduler::new();

    // Channel for synchronizing the scrapper and the bot
    let (tx, rx): (Sender<NewsPost>, Receiver<NewsPost>) = mpsc::channel();

    // Graceful shutdown.
    let running = Arc::new(AtomicBool::new(true));
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
            if !redis_service
                .is_key_flagged(format!("{}-{}", &args.redis_stream_name, &title).as_str())
                .await
            {
                let published = redis_service
                    .publish(&args.redis_stream_name, &news_post)
                    .await;
                if published {
                    info!("Published {:?}", news_post);
                    redis_service.flag_key(&title, 60 * 60 * 24 * 14).await;
                }
            };
        }
    }

    info!("Stopped the program");

    handle.await?;

    Ok(())
}
