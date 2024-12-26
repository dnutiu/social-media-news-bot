use log::error;
use post::NewsPost;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};

pub struct RedisService {
    multiplexed_connection: MultiplexedConnection,
    stream_name: String,
}

impl RedisService {
    /// Creates a new RedisService instance.
    pub async fn new(connection_string: &str, stream_name: &str) -> Self {
        let client = redis::Client::open(connection_string).unwrap();
        let con = client.get_multiplexed_async_connection().await.unwrap();

        RedisService {
            multiplexed_connection: con,
            stream_name: stream_name.to_string(),
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
    /// Returns a `bool` that is true if the post was published and false otherwise.
    pub async fn publish(&mut self, post: &NewsPost) -> bool {
        let serialized_post = serde_json::to_string(&post).unwrap();
        let result = redis::cmd("XADD")
            .arg(format!("posts:{}", self.stream_name))
            .arg("*")
            .arg("post_data")
            .arg(serialized_post)
            .exec_async(&mut self.multiplexed_connection)
            .await;
        if result.is_err() {
            error!("Failed to publish {:?} to stream", result);
            return false;
        };
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::{Alphanumeric, DistString};
    use redis::RedisResult;
    use serial_test::serial;

    const REDIS_CONNECTION_STRING: &str = "redis://localhost:6379";

    /// Cleans up the database
    async fn cleanup(redis_service: &mut RedisService) {
        redis::cmd("flushall")
            .exec_async(&mut redis_service.multiplexed_connection)
            .await
            .expect("failed to clean db")
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_new() {
        let _ = RedisService::new(REDIS_CONNECTION_STRING, "a").await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_is_post_seen_false() {
        // Setup
        let random_stream_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
        let random_post = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(REDIS_CONNECTION_STRING, &random_stream_name).await;

        // Test
        let result = service.is_post_seen(&random_post).await;

        // Assert
        assert_eq!(result, false);
        cleanup(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_is_post_seen_true() {
        // Setup
        let random_stream_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
        let random_post = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(REDIS_CONNECTION_STRING, &random_stream_name).await;
        service.mark_post_seen(&random_post, 10).await;

        // Test
        let result = service.is_post_seen(&random_post).await;

        // Assert
        assert_eq!(result, true);
        cleanup(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_publish() {
        // Setup
        let random_stream_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(REDIS_CONNECTION_STRING, &random_stream_name).await;

        // Test
        let post = NewsPost {
            image: Some(String::from("i")),
            title: Some(String::from("t")),
            summary: Some(String::from("s")),
            link: Some(String::from("l")),
            author: Some(String::from("a")),
        };
        let result = service.publish(&post).await;

        let stream_length: RedisResult<i32> = redis::cmd("XLEN")
            .arg(&format!("posts:{}", random_stream_name))
            .query_async(&mut service.multiplexed_connection)
            .await;

        // Assert
        assert_eq!(result, true);
        assert_eq!(stream_length, Ok(1));
        cleanup(&mut service).await;
    }
}
