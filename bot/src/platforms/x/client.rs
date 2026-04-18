use std::option::Option;
use anyhow::anyhow;
use async_trait::async_trait;
use post::{NewsPost, Publisher};

#[allow(dead_code)]
pub struct XApiClient {
    bearer_code: String
}

impl XApiClient {
    /// Constructs a new instance of XAPIClient.
    #[allow(dead_code)]
    pub fn new(bearer_code: String) -> Self {
        panic!("XApiClient is not implemented yet, feel free to send a PR.");
        Self { bearer_code }
    }

    /// Uploads a media by a given link and returns the media id.
    #[allow(dead_code)]
    pub async fn upload_media_by_link(&self, media_link: String) -> Result<String, anyhow::Error> {
        todo!("upload_media_by_link is not implemented, got {}", media_link)
    }

    /// Posts the tweet on X api with media if available.
    #[allow(dead_code)]
    pub async fn post_tweet(&self, text: String, media_id: Option<String>) -> Result<(), anyhow::Error> {
        todo!("post_tweet is not implemented, got {} - {}", text, media_id.unwrap_or_default())
    }
}

#[async_trait]
impl Publisher for XApiClient {

    /// Publishes a post on X.
    async fn publish_post(&mut self, post: NewsPost) -> Result<(), anyhow::Error> {
        let media_id = if post.link.is_some() {
            self.upload_media_by_link(post.link.clone().unwrap_or_default()).await
        } else { Err(anyhow!("no media link found")) };

        let post_text = post::extract_text_from_post(post, 280);

        match media_id {
            Ok(media_id) => {
                self.post_tweet(post_text, Some(media_id)).await
            },
            Err(_) => {
                self.post_tweet(post_text, None).await
            },
        }
    }
}

