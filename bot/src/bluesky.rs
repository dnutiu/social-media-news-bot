mod atproto;

use crate::bluesky::atproto::ATProtoServerCreateSession;
use crate::token::Token;
use reqwest::Body;

/// The BlueSky client used to interact with the platform.
pub struct BlueSkyClient {
    auth_token: Token,
    client: reqwest::Client,
}

impl BlueSkyClient {
    pub async fn new(user_handle: &str, user_password: &str) -> Result<Self, anyhow::Error> {
        let client = reqwest::Client::new();
        let server_create_session = ATProtoServerCreateSession {
            identifier: user_handle.to_string(),
            password: user_password.to_string(),
        };
        let body = serde_json::to_string(&server_create_session)?;
        let token: Token = client
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
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

    pub async fn post<T>(&mut self, body: T) -> Result<(), anyhow::Error>
    where
        T: Into<Body>,
    {
        let token_expired = self.auth_token.is_expired()?;
        if token_expired {
            self.renew_token().await?;
        }
        self.client
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
            .header(
                "Authorization",
                format!("Bearer, {}", self.auth_token.access_jwt),
            )
            .body(body)
            .send()
            .await?;
        Ok(())
    }

    async fn renew_token(&mut self) -> Result<(), anyhow::Error> {
        let result: Token = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.server.refreshSession")
            .header(
                "Authorization",
                format!("Bearer, {}", self.auth_token.refresh_jwt),
            )
            .send()
            .await?
            .json()
            .await?;
        self.auth_token = result;
        Ok(())
    }
}
