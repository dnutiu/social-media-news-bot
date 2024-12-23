use crate::scrapper::{NewsPost, ScrappableWebPage};
use anyhow::anyhow;
use scraper::{Html, Selector};

#[derive(Debug)]
/// G4 Media website scraper
pub struct G4Media {
    url: String,
}

impl Default for G4Media {
    fn default() -> Self {
        G4Media {
            url: String::from("https://www.g4media.ro"),
        }
    }
}

impl ScrappableWebPage for G4Media {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn get_posts(&self, html: String) -> Result<Vec<NewsPost>, anyhow::Error> {
        let document = Html::parse_document(&html);
        let mut posts: Vec<NewsPost> = vec![];
        let posts_selector =
            Selector::parse(".post-review").map_err(|_e| anyhow!("failed to make selector"))?;

        let anchor_selector =
            Selector::parse("a").map_err(|_e| anyhow!("failed to make selector"))?;
        let post_img_selector = Selector::parse(".post-img > a > img")
            .map_err(|_e| anyhow!("failed to make selector"))?;
        let post_title_selector =
            Selector::parse(".post-title").map_err(|_e| anyhow!("failed to make selector"))?;
        let post_summary_selector =
            Selector::parse(".post-content p").map_err(|_e| anyhow!("failed to make selector"))?;
        let post_metadata_author_selector = Selector::parse(".post-medatada .entry-author a")
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

            if let Some(selected_post_title) = element.select(&post_title_selector).next() {
                if let Some(post_link) = selected_post_title.select(&anchor_selector).next() {
                    if let Some(href) = post_link.value().attr("href") {
                        news_post.link = Some(href.to_owned());
                    }
                    if let Some(title) = post_link.value().attr("title") {
                        news_post.title = Some(title.to_owned())
                    }
                }
            }
            if let Some(selected_summary) = element.select(&post_summary_selector).next() {
                news_post.summary = Some(selected_summary.inner_html().trim().replace("&nbsp;", ""))
            }
            if let Some(selected_author) = element.select(&post_metadata_author_selector).next() {
                news_post.author = Some(selected_author.inner_html());
            }
            if let Some(selected_image) = element.select(&post_img_selector).next() {
                if let Some(image_source) = selected_image.attr("data-src") {
                    news_post.image = Some(image_source.to_string());
                }
            }

            posts.push(news_post);
        }

        Ok(posts)
    }
}
