use crate::platforms::mastodon::api::{PartialMediaResponse, PartialPostStatusResponse, PostStatusRequest};
use anyhow::{Error, anyhow};
use async_trait::async_trait;
use log::{debug, error, info};
use post::{NewsPost, Publisher};
use reqwest::StatusCode;
use std::fmt;

pub mod api;
pub mod cli;

/// The Mastodon client for interacting with the platform.
pub struct MastodonClient {
    access_token: String,
    client: reqwest::Client,
}

impl MastodonClient {
    /// Creates a new mastodon client from the given access token.
    pub fn new(access_token: String) -> Self {
        let client = reqwest::Client::new();
        MastodonClient {
            access_token,
            client,
        }
    }

    /// Posts a new status to Mastodon.
    pub async fn post_status<T>(
        &mut self,
        data: T,
    ) -> Result<PartialPostStatusResponse, anyhow::Error>
    where
        T: Into<PostStatusRequest> + fmt::Debug,
    {
        let post_status_request: PostStatusRequest = data.into();
        let json = serde_json::to_string(&post_status_request)?;

        let response = self
            .client
            .post("https://mastodon.social/api/v1/statuses")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .body(json)
            .send()
            .await?;

        let response_status: StatusCode = response.status();
        if response_status != 200 {
            let response_text = response.text().await?;
            debug!("Request:\n{post_status_request:?}\nEND");
            debug!("Response:\n{response_text}\nEND");
            return Err(anyhow!("Failed to post on Mastodon, got {response_status}"));
        }

        Ok(response.json().await?)
    }

    /// Uploads an image to Mastodon.
    pub async fn upload_media_by_url(
        &mut self,
        image_url: &str,
    ) -> Result<PartialMediaResponse, anyhow::Error> {
        let data: Vec<u8> = self
            .client
            .get(image_url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();

        let file_part = reqwest::multipart::Part::bytes(data)
            .file_name("image.jpg")
            .mime_str("image/jpg")?;

        let form = reqwest::multipart::Form::new().part("file", file_part);

        Ok(self
            .client
            .post("https://mastodon.social/api/v2/media")
            .header("Content-Type", "multipart/form-data")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

#[async_trait]
impl Publisher for MastodonClient {
    async fn publish_post(&mut self, post: NewsPost) -> Result<(), Error> {
        // Step1: Upload image to Mastodon
        let media_response = if post.image.is_some() {
            let response = self
                .upload_media_by_url(post.image.clone().unwrap().as_str())
                .await;

            match response {
                Ok(response) => Ok(response),
                Err(err) => Err(anyhow!("failed to upload image: {err}")),
            }
        } else {
            Err(anyhow!("No image exists on post."))
        };

        // Step2: Post to Mastodon.
        let mut status: PostStatusRequest = post.into();
        match media_response {
            Ok(response) => {
                status.media_ids.push(response.id);
            }
            Err(err) => {
                error!("Error uploading image: {err}")
            }
        }
        let response = self.post_status(status).await;
        match response {
            Ok(response) => {
                info!("Posted tooth on Mastodon! {response:?}");
                Ok(())
            }
            Err(err) => {
                error!("Failed to post toot on Mastodon: {err}");
                Err(err)
            }
        }
    }
}
