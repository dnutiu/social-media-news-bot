use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// NewsPost represents a news post.
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
pub struct NewsPost {
    /// A URL containing the image of the post.
    pub image: Option<String>,
    /// The title of the post.
    pub title: Option<String>,
    /// A summary of the post.
    pub summary: Option<String>,
    /// A link to the post.
    pub link: Option<String>,
    /// The author of the post.
    pub author: Option<String>,
}

impl NewsPost {
    /// Is complete checks if the news post contains the minimum fields.
    pub fn is_complete(&self) -> bool {
        self.title.is_some() && self.link.is_some()
    }
}

/// Publisher trait defines the contract for publishing news posts.
#[async_trait]
pub trait Publisher {
    /// publish_post publishes the NewsPost.
    /// Returns an error if the publishing fails.
    async fn publish_post(&mut self, post: NewsPost) -> Result<(), anyhow::Error>;
}

/// Extracts the tweet's text from a newspost.
pub fn extract_text_from_post(value: NewsPost, character_budget: i32) -> String {
    let mut status = String::new();

    // The character budget for mastodon.social.
    let mut character_budget: i32 = character_budget;
    let title = value.title.unwrap_or(String::from("Post Title"));
    let summary = value.summary.unwrap_or(String::from(""));
    let link = value.link.unwrap_or(String::from(""));

    // reserve space for the link + one space
    character_budget -= link.len() as i32 + 1;

    // Push the title
    if character_budget > 0 {
        status.push_str(
            title
                .get(0..character_budget as usize)
                .unwrap_or(title.as_str()),
        );
        character_budget -= title.len() as i32 + 2;
        status.push('\n')
    }

    // Push the summary
    if character_budget > 0 {
        status.push_str(
            summary
                .get(0..character_budget as usize)
                .unwrap_or(summary.as_str()),
        );
        status.push('\n')
    }

    // Push the link
    status.push_str(link.as_str());
    status
}
