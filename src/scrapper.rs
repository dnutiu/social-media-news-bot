mod gfourmedia;

/// NewsPost represents a news post.
pub struct NewsPost {
    /// A URL containing the image of the post.
    pub image: Option<String>,
    /// The title of the post.
    pub title: String,
    /// A summary of the post.
    pub summary: Option<String>,
    /// The content of the post.
    pub content: Option<String>,
    /// A link to the post.
    pub link: String,
    /// The author of the post.
    pub author: String,
}
