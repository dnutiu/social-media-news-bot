use crate::mastodon::api::{PartialMediaResponse, PartialPostStatusResponse, PostStatusRequest};

pub mod api;

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
        T: Into<PostStatusRequest>,
    {
        let post_status_request: PostStatusRequest = data.into();
        let json = serde_json::to_string(&post_status_request)?;

        Ok(self
            .client
            .post("https://mastodon.social/api/v1/statuses")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .body(json)
            .send()
            .await?
            .json()
            .await?)
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
            .json()
            .await?)
    }
}
