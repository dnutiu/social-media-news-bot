pub(crate) mod atproto;
mod token;

use crate::bluesky::atproto::{ATProtoServerCreateSession, BlobResponse};
use anyhow::anyhow;
use reqwest::Body;
use token::Token;

/// The BlueSky client used to interact with the platform.
pub struct BlueSkyClient {
    auth_token: Token,
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
            .json()
            .await?;

        Ok(BlueSkyClient {
            auth_token: token,
            client,
        })
    }

    /// Makes a new tweet.
    pub async fn post<T>(&mut self, body: T) -> Result<(), anyhow::Error>
    where
        T: Into<Body>,
    {
        let token_expired = self.auth_token.is_expired()?;
        if token_expired {
            self.renew_token().await?;
        }
        let response_code = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.auth_token.access_jwt),
            )
            .body(body)
            .send()
            .await?
            .status();

        if response_code != 200 {
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
            .json()
            .await?)
    }
}
