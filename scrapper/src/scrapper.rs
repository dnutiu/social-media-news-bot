use async_trait::async_trait;
use post::NewsPost;

/// Represents a web scrapper which is can be scraped by the engine.
#[async_trait]
pub(crate) trait ScrappableWebPage: Send + Sync {
    fn get_url(&self) -> &str;
    fn get_posts(&self, html: String) -> Result<Vec<NewsPost>, anyhow::Error>;
}

/// The web scraper engine is used to scrape web pages.
pub struct WebScrapperEngine;

impl WebScrapperEngine {
    pub async fn get_posts<P>(web_page: P) -> Result<Vec<NewsPost>, anyhow::Error>
    where
        P: ScrappableWebPage,
    {
        let body = reqwest::get(web_page.get_url())
            .await?
            .error_for_status()?
            .text()
            .await?;

        let results = web_page.get_posts(body)?;
        Ok(results)
    }
}
