pub(crate) mod gfourmedia;

/// NewsPost represents a news post.
#[derive(Debug)]
pub struct NewsPost {
    /// A URL containing the image of the post.
    pub image: Option<String>,
    /// The title of the post.
    pub title: Option<String>,
    /// A summary of the post.
    pub summary: Option<String>,
    /// The content of the post.
    pub content: Option<String>,
    /// A link to the post.
    pub link: Option<String>,
    /// The author of the post.
    pub author: Option<String>,
}

impl NewsPost {
    /// Is complete checks if the news post contains the minimum fields.
    pub fn is_complete(&self) -> bool {
        self.title.is_some() && self.summary.is_some() && self.link.is_some()
    }
}

/// Represents a web scrapper which is can be scraped by the engine.
pub(crate) trait ScrappableWebPage {
    fn get_url(&self) -> &str;
    fn get_posts(&self, html: String) -> Result<Vec<NewsPost>, anyhow::Error>;
}

/// The web scraper engine is used to scrape web pages.
pub struct WebScrapperEngine<P>
where
    P: ScrappableWebPage,
{
    web_page: P,
}

impl<P> WebScrapperEngine<P>
where
    P: ScrappableWebPage,
{
    /// Creates a new instance of WebScrapperEngine
    pub async fn new(web_page: P) -> Result<Self, anyhow::Error> {
        Ok(WebScrapperEngine { web_page })
    }

    pub async fn get_posts(&self) -> Result<Vec<NewsPost>, anyhow::Error> {
        let body = reqwest::get(self.web_page.get_url()).await?.text().await?;

        let results = self.web_page.get_posts(body)?;
        Ok(results)
    }
}
