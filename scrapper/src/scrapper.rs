use post::NewsPost;

pub(crate) mod gfourmedia;
pub(crate) mod hotnews;

/// Represents a web scrapper which is can be scraped by the engine.
pub(crate) trait ScrappableWebPage {
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
        let body = reqwest::get(web_page.get_url()).await?.text().await?;

        let results = web_page.get_posts(body)?;
        Ok(results)
    }
}
