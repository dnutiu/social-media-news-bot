mod atproto;
pub mod cli;
mod token;

use crate::platforms::bluesky;
use crate::platforms::bluesky::atproto::{ATProtoServerCreateSession, BlobResponse};
use anyhow::{Error, anyhow};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use post::{NewsPost, Publisher};
use reqwest::Body;
use std::fmt;
use token::Token;

/// The BlueSky client used to interact with the platform.
pub struct BlueSkyClient {
    auth_token: Token,
    user_handle: String,
    client: reqwest::Client,
}

impl BlueSkyClient {
    /// Creates a new BlueSky client instance for the given account credentials.
    pub async fn new(user_handle: &str, user_password: &str) -> Result<Self, anyhow::Error> {
        let client = reqwest::Client::new();
        let server_create_session = ATProtoServerCreateSession {
            identifier: user_handle.to_string(),
            password: user_password.to_string(),
        };
        let body = serde_json::to_string(&server_create_session)?;
        let token: Token = client
            .post("https://bsky.social/xrpc/com.atproto.server.createSession")
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(BlueSkyClient {
            auth_token: token,
            user_handle: user_handle.to_string(),
            client,
        })
    }

    /// Makes a new tweet.
    pub async fn post<T>(&mut self, body: T) -> Result<(), anyhow::Error>
    where
        T: Into<Body> + fmt::Debug + Clone,
    {
        let token_expired = self.auth_token.is_expired()?;
        if token_expired {
            self.renew_token().await?;
        }
        let response = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.auth_token.access_jwt),
            )
            .body(body.clone())
            .send()
            .await?;

        let response_code = response.status();
        if response_code != 200 {
            let response_text = response.text().await?;
            debug!("Request:\n{body:?}\nEND");
            debug!("Response:\n{response_text}\nEND");
            return Err(anyhow!("Failed to post on BlueSky, got {response_code}"));
        }
        Ok(())
    }

    /// Renews the Authentication JWT bearer token using the refresh token.
    async fn renew_token(&mut self) -> Result<(), anyhow::Error> {
        let result: Token = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.server.refreshSession")
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.auth_token.refresh_jwt),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        self.auth_token = result;
        Ok(())
    }

    /// Uploads an image.
    pub async fn upload_image_by_url(
        &mut self,
        image_url: &str,
    ) -> Result<BlobResponse, anyhow::Error> {
        let data: Vec<u8> = self
            .client
            .get(image_url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();

        Ok(self
            .client
            .post("https://bsky.social/xrpc/com.atproto.repo.uploadBlob")
            .header("Content-Type", "image/jpeg")
            .header(
                "Authorization",
                format!("Bearer {}", self.auth_token.access_jwt),
            )
            .body(data)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

/// Embeds an image to a post.
async fn add_image_to_post(
    client: &mut BlueSkyClient,
    image_url: &str,
    record: &mut bluesky::atproto::ATProtoRepoCreateRecord,
) -> Result<(), anyhow::Error> {
    let thumb = client.upload_image_by_url(image_url).await?;
    record.record.embed.as_mut().unwrap().external.thumb = Some(thumb.blob);

    Ok(())
}

#[async_trait]
impl Publisher for BlueSkyClient {
    async fn publish_post(&mut self, post: NewsPost) -> Result<(), Error> {
        let mut data: atproto::ATProtoRepoCreateRecord = post.clone().into();
        data.repo = self.user_handle.clone();

        if let Some(image_link) = post.image.clone() {
            let result = add_image_to_post(self, &image_link, &mut data).await;
            if let Err(err) = result {
                warn!("Failed to upload image: {err}")
            }
        }
        let json = serde_json::to_string(&data);
        match json {
            Ok(json) => {
                if let Err(err) = self.post(json).await {
                    error!("failed to post: {post:?} {err}");
                    Err(err)
                } else {
                    info!("Published a post! 🦀");
                    Ok(())
                }
            }
            Err(err) => {
                error!("failed to convert post to json: {post:?} {err}");
                Err(err.into())
            }
        }
    }
}
