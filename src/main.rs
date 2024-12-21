use crate::scrapper::gfourmedia::G4Media;
use crate::scrapper::WebScrapperEngine;
use clokwerk::{AsyncScheduler, TimeUnits};
use std::time::Duration;
mod scrapper;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let mut scheduler = AsyncScheduler::new();
    scheduler.every(60.seconds()).run(|| async {
        let posts = WebScrapperEngine::get_posts(G4Media::default())
            .await
            .expect("failed to get posts");
        posts
            .iter()
            .filter(|p| p.is_complete())
            .for_each(|p| println!("{:?}", p));
    });
    // Manually run the scheduler forever
    loop {
        scheduler.run_pending().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}
