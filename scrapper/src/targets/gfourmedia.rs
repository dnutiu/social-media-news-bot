use crate::scrapper::ScrappableWebPage;
use anyhow::anyhow;
use post::NewsPost;
use scraper::{Html, Selector};
use std::string::String;

#[derive(Debug)]
/// HotNews website scraper
pub struct GFourMedia {
    url: String,
    default_author: String,
}

impl Default for GFourMedia {
    fn default() -> Self {
        GFourMedia {
            url: String::from("https://www.g4media.ro/"),
            default_author: String::from("G4Media"),
        }
    }
}

impl ScrappableWebPage for GFourMedia {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn get_posts(&self, html: String) -> Result<Vec<NewsPost>, anyhow::Error> {
        let document = Html::parse_document(&html);

        let mut posts: Vec<NewsPost> = vec![];

        // G4Media structure (homepage/list):
        // - container: div.article
        // - title/link: h2.article__title > a
        // - image: figure picture img (or any .article__media img)
        // - authors: .article__eyebrow a[rel='author'] (one or more)
        let posts_selector =
            Selector::parse("div.article").map_err(|_e| anyhow!("failed to make selector"))?;

        let post_img_selector = Selector::parse("figure picture img, .article__media img")
            .map_err(|_e| anyhow!("failed to make selector"))?;
        let post_title_selector = Selector::parse("h2.article__title > a")
            .map_err(|_e| anyhow!("failed to make selector"))?;
        let post_authors_selector = Selector::parse(".article__eyebrow a[rel='author']")
            .map_err(|_e| anyhow!("failed to make selector"))?;
        let post_excerpt_selector = Selector::parse(".article__excerpt")
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

            if let Some(selected_image) = element.select(&post_img_selector).next() {
                // prefer data-src or src attributes
                if let Some(image_source) = selected_image
                    .attr("src")
                    .or_else(|| selected_image.attr("data-src"))
                {
                    news_post.image = Some(image_source.to_string());
                }
            }

            // Collect authors if present; otherwise fallback to default author
            let authors: Vec<String> = element
                .select(&post_authors_selector)
                .map(|a| a.inner_html().trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            news_post.author = if !authors.is_empty() {
                Some(authors.join(", "))
            } else {
                Some(self.default_author.clone())
            };

            // Extract summary/excerpt if present
            if let Some(excerpt_el) = element.select(&post_excerpt_selector).next() {
                let excerpt_text: String = excerpt_el.text().collect::<Vec<_>>().join(" ");
                let excerpt_trimmed = excerpt_text.trim();
                if !excerpt_trimmed.is_empty() {
                    news_post.summary = Some(excerpt_trimmed.to_string());
                }
            }

            posts.push(news_post);
        }

        Ok(posts)
    }
}

#[cfg(test)]
mod tests {
    use crate::GFourMedia;
    use crate::WebScrapperEngine;

    #[tokio::test]
    async fn sanity_test() {
        let posts = WebScrapperEngine::get_posts(GFourMedia::default()).await;

        assert!(posts.is_ok());

        let posts = posts.unwrap();

        assert!(!posts.is_empty());

        assert!(posts[0].image.is_some());
        assert!(posts[0].title.is_some());
        assert!(posts[0].link.is_some());
        assert!(posts[0].author.is_some());
    }
}
