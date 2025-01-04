use crate::mastodon::api::{PartialMediaResponse, PartialPostStatusResponse, PostStatusRequest};

mod api;

/// The Mastodon client for interacting with the platform.
pub struct MastodonClient {
    access_token: String,
    client: reqwest::Client,
}

impl MastodonClient {
    /// Creates a new mastodon client from the given access token.
    fn new(access_token: String) -> Self {
        let client = reqwest::Client::new();
        MastodonClient {
            access_token,
            client,
        }
    }

    async fn post_status<T>(&mut self, data: T) -> Result<PartialPostStatusResponse, anyhow::Error>
    where
        T: Into<PostStatusRequest>,
    {
        unimplemented!()
    }

    async fn upload_media_by_url(
        &mut self,
        image_url: &str,
    ) -> Result<PartialMediaResponse, anyhow::Error> {
        unimplemented!()
    }
}
