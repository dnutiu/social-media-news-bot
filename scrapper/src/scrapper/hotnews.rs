use crate::scrapper::ScrappableWebPage;
use anyhow::anyhow;
use post::NewsPost;
use scraper::{Html, Selector};
use std::string::String;

#[derive(Debug)]
/// HotNews website scraper
pub struct HotNews {
    url: String,
    default_author: String,
}

impl Default for HotNews {
    fn default() -> Self {
        HotNews {
            url: String::from("https://www.hotnews.ro"),
            default_author: String::from("HotNews"),
        }
    }
}

impl ScrappableWebPage for HotNews {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn get_posts(&self, html: String) -> Result<Vec<NewsPost>, anyhow::Error> {
        let document = Html::parse_document(&html);
        let mut posts: Vec<NewsPost> = vec![];

        let posts_selector =
            Selector::parse("article").map_err(|_e| anyhow!("failed to make selector"))?;

        let post_img_selector =
            Selector::parse("figure > a > img").map_err(|_e| anyhow!("failed to make selector"))?;
        let post_title_selector = Selector::parse("div.entry-wrapper > h2.entry-title > a")
            .map_err(|_e| anyhow!("failed to make selector"))?;
        let selected_posts = document.select(&posts_selector);

        for element in selected_posts {
            let mut news_post = NewsPost {
                image: None,
                title: None,
                summary: None,
                link: None,
                author: None,
            };

            if let Some(post_title) = element.select(&post_title_selector).next() {
                news_post.title = Some(post_title.inner_html().trim().to_string());
                if let Some(href) = post_title.value().attr("href") {
                    news_post.link = Some(href.to_owned());
                }
            }

            if let Some(selected_image) = element.select(&post_img_selector).next()
                && let Some(image_source) = selected_image.attr("src")
            {
                news_post.image = Some(image_source.to_string());
            }

            news_post.author = Option::from(self.default_author.clone());
            news_post.summary = Option::from(String::from(""));

            posts.push(news_post);
        }

        Ok(posts)
    }
}

#[cfg(test)]
mod tests {
    use crate::WebScrapperEngine;
    use crate::scrapper::hotnews::HotNews;

    #[tokio::test]
    async fn sanity_test() {
        let posts = WebScrapperEngine::get_posts(HotNews::default()).await;

        assert!(posts.is_ok());

        let posts = posts.unwrap();

        assert!(!posts.is_empty());

        assert!(posts[0].image.is_some());
        assert!(posts[0].title.is_some());
        assert!(posts[0].link.is_some());
        assert!(posts[0].author.is_some());
    }
}
