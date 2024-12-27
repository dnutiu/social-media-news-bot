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
        self.title.is_some() && self.summary.is_some() && self.link.is_some()
    }
}
