use post::NewsPost;
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
    pub media_ids: Vec<String>,
}

impl From<NewsPost> for PostStatusRequest {
    fn from(value: NewsPost) -> Self {
        let mut status = String::new();

        // The character budget for mastodon.social.
        let mut character_budget = 500;
        let title = value.title.unwrap();
        let summary = value.summary.unwrap();
        let link = value.link.unwrap();

        // reserve space for the link + one space
        character_budget -= link.len() + 2;

        // Push the title
        if character_budget > 0 {
            status.push_str(title.get(0..character_budget).unwrap_or(title.as_str()));
            character_budget -= title.len() + 2;
            status.push('\n')
        }

        // Push the summary
        if character_budget > 0 {
            status.push_str(summary.get(0..character_budget).unwrap_or(summary.as_str()));
            status.push('\n')
        }

        // Push the link
        status.push_str(link.as_str());

        PostStatusRequest {
            status,
            language: String::from("ro"),
            visibility: String::from("public"),
            media_ids: vec![],
        }
    }
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
