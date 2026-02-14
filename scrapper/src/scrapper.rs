use async_trait::async_trait;
use post::NewsPost;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;

/// Represents a web scrapper which is can be scraped by the engine.
#[async_trait]
pub(crate) trait ScrappableWebPage: Send + Sync {
    fn get_url(&self) -> String;
    fn get_posts(&self, html: String) -> Result<Vec<NewsPost>, anyhow::Error>;
}

/// The web scraper engine is used to scrape web pages.
pub struct WebScrapperEngine {
    client: reqwest_middleware::ClientWithMiddleware,
}

impl Default for WebScrapperEngine {
    fn default() -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        WebScrapperEngine { client }
    }
}

impl WebScrapperEngine {
    pub async fn get_posts<P>(&self, web_page: P) -> Result<Vec<NewsPost>, anyhow::Error>
    where
        P: ScrappableWebPage,
    {
        let body = self
            .client
            .get(web_page.get_url())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let results = web_page.get_posts(body)?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct TestScrapper<'a> {
        mock_server: &'a MockServer,
    }

    impl<'a> TestScrapper<'a> {
        fn new(mock_server: &'a MockServer) -> Self {
            TestScrapper { mock_server }
        }
    }

    impl<'a> ScrappableWebPage for TestScrapper<'a> {
        fn get_url(&self) -> String {
            format!("{}/testing", self.mock_server.uri())
        }

        fn get_posts(&self, _html: String) -> Result<Vec<NewsPost>, Error> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_request_is_done() {
        // Setup
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/testing"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;

        let test_scraper = TestScrapper::new(&mock_server);
        let default_engine = WebScrapperEngine::default();
        // Test
        let _ = default_engine.get_posts(test_scraper).await;

        // Assert
        mock_server.verify().await;
    }

    #[tokio::test]
    async fn test_request_is_retried() {
        // Setup
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/testing"))
            .respond_with(ResponseTemplate::new(500))
            .expect(4)
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;

        let test_scraper = TestScrapper::new(&mock_server);
        let default_engine = WebScrapperEngine::default();
        // Test
        let _ = default_engine.get_posts(test_scraper).await;

        // Assert
        mock_server.verify().await;
    }
}
