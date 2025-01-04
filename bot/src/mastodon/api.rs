use serde::{Deserialize, Serialize};

/// Is a truncated response from Mastodon's /api/v2/media endpoint.
/// See: https://docs.joinmastodon.org/methods/media/#v2
#[derive(Serialize, Deserialize, Debug)]
pub struct PartialMediaResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub url: String,
}

/// PostStatusRequest is the request made to post a status on Mastodon.
/// See: https://docs.joinmastodon.org/methods/statuses/#create
#[derive(Serialize, Deserialize, Debug)]
pub struct PostStatusRequest {
    pub status: String,
    pub language: String,
    pub visibility: String,
    pub media_ids: Vec<u64>,
}

/// Is a partial response from /api/v1/statuses route.
/// See: https://docs.joinmastodon.org/methods/statuses/#create
#[derive(Serialize, Deserialize, Debug)]
pub struct PartialPostStatusResponse {
    pub id: String,
    pub created_at: String,
    pub visibility: String,
    pub uri: String,
    pub url: String,
}
