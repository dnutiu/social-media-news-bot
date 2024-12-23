use crate::scrapper::NewsPost;
use log::error;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};

pub struct RedisService {
    multiplexed_connection: MultiplexedConnection,
    stream_name: String,
}

impl RedisService {
    /// Creates a new RedisService instance.
    pub async fn new(connection_string: String, stream_name: String) -> Self {
        let client = redis::Client::open(connection_string).unwrap();
        let con = client.get_multiplexed_async_connection().await.unwrap();

        RedisService {
            multiplexed_connection: con,
            stream_name,
        }
    }

    //noinspection RsSelfConvention
    /// Returns true if the key exists in Redis, false otherwise.
    pub async fn is_post_seen(&mut self, title: &str) -> bool {
        let digest = md5::compute(title);
        let result: Result<bool, RedisError> = self
            .multiplexed_connection
            .get(format!("{:x}", digest))
            .await;
        result.unwrap_or(false)
    }

    /// Marks the post as seen
    pub async fn mark_post_seen(&mut self, title: &str, ttl: u64) {
        let digest = md5::compute(title);
        let _ = self
            .multiplexed_connection
            .set_ex::<String, bool, bool>(format!("{:x}", digest), true, ttl)
            .await;
    }

    /// Publishes the post to the redis stream.
    pub async fn publish(&mut self, post: NewsPost) {
        let serialized_post = serde_json::to_string(&post).unwrap();
        let result = redis::cmd("XADD")
            .arg(format!("posts:{}", self.stream_name))
            .arg("*")
            .arg(serialized_post)
            .exec_async(&mut self.multiplexed_connection)
            .await;
        if result.is_err() {
            error!("Failed to publish {:?} to stream", post);
        }
    }
}
