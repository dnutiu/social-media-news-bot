use crate::scrapper::gfourmedia::G4Media;
use crate::scrapper::WebScrapperEngine;
mod scrapper;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    println!("Hello, world!");

    let scrapper = WebScrapperEngine::new(G4Media::default()).await?;
    let posts = scrapper.get_posts().await?;

    posts.iter().for_each(|p| println!("{:?}", p));

    Ok(())
}
