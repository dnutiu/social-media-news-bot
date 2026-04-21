use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use log::info;
use oauth1::Token;
use post::{NewsPost, Publisher};
use reqwest::multipart;
use serde_json::json;
use std::option::Option;

#[allow(dead_code)]
pub struct XApiClient {
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String,
    http_client: reqwest::Client,
}

impl XApiClient {
    /// Constructs a new instance of XApiClient.
    pub fn new(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent("SocialMediaNewsBot/1.0 (Rust OAuth1)")
            .build()
            .expect("Failed to build HTTP client");

        Self {
            consumer_key,
            consumer_secret,
            access_token,
            access_token_secret,
            http_client,
        }
    }

    /// Downloads media from a URL and uploads it to X, returning the media_id.
    pub async fn upload_media_by_link(&self, media_link: String) -> Result<String> {
        let download_resp = self
            .http_client
            .get(&media_link)
            .send()
            .await
            .context("Failed to download media")?;

        let media_bytes = download_resp
            .bytes()
            .await
            .context("Failed to read media bytes")?;

        let filename = media_link
            .split('/')
            .next_back()
            .unwrap_or("media.jpg")
            .to_string();

        let mime_type = if filename.ends_with(".png") {
            "image/png"
        } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
            "image/jpeg"
        } else if filename.ends_with(".gif") {
            "image/gif"
        } else {
            "image/jpeg"
        };

        let part = multipart::Part::bytes(media_bytes.to_vec())
            .file_name(filename)
            .mime_str(mime_type)?;

        let form = multipart::Form::new()
            .part("media", part)
            .text("media_category", "tweet_image");

        let upload_url = "https://upload.twitter.com/1.1/media/upload.json";

        let auth_header = self.sign_request("POST", upload_url);

        let upload_resp = self
            .http_client
            .post(upload_url)
            .header("Authorization", auth_header)
            .multipart(form)
            .send()
            .await
            .context("Media upload request failed")?;

        let upload_status = upload_resp.status();
        if !upload_status.is_success() {
            let error_text = upload_resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Media upload failed {}: {}",
                upload_status,
                error_text
            ));
        }

        let json: serde_json::Value = upload_resp.json().await?;
        let media_id = json["media_id_string"]
            .as_str()
            .ok_or_else(|| anyhow!("No media_id_string in response"))?
            .to_string();

        Ok(media_id)
    }
    /// Posts a tweet on X (with optional media).
    pub async fn post_tweet(&self, text: String, media_id: Option<String>) -> Result<()> {
        let tweet_url = "https://api.x.com/2/tweets";

        let mut body = json!({ "text": text });

        if let Some(id) = media_id {
            body["media"] = json!({ "media_ids": [id] });
        }

        let auth_header = self.sign_request("POST", tweet_url);

        let resp = self
            .http_client
            .post(tweet_url)
            .header("Authorization", auth_header)
            .json(&body)
            .send()
            .await
            .context("Tweet post request failed")?;

        let resp_status = resp.status();
        if !resp_status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Failed to post tweet {}: {}",
                resp_status,
                error_text
            ));
        }

        let json: serde_json::Value = resp.json().await?;
        if let Some(id) = json["data"]["id"].as_str() {
            info!("Tweet posted successfully! ID: {}", id);
        }

        Ok(())
    }

    fn sign_request(&self, method: &str, url: &str) -> String {
        let consumer = Token::new(&self.consumer_key, &self.consumer_secret);
        let access = Token::new(&self.access_token, &self.access_token_secret);

        oauth1::authorize(method, url, &consumer, Some(&access), None)
    }
}

#[async_trait]
impl Publisher for XApiClient {
    /// Publishes a post on X.
    async fn publish_post(&mut self, post: NewsPost) -> Result<()> {
        let post_text = post::extract_text_from_post(post.clone(), 280);

        let media_id = if let Some(link) = &post.link {
            match self.upload_media_by_link(link.clone()).await {
                Ok(id) => Some(id),
                Err(e) => {
                    eprintln!(
                        "Media upload failed: {}. Falling back to text-only post.",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        self.post_tweet(post_text, media_id).await
    }
}
