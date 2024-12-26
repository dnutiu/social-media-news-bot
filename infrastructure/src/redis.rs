use log::error;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};
use serde::Serialize;

pub struct RedisService {
    multiplexed_connection: MultiplexedConnection,
}

impl RedisService {
    /// Creates a new RedisService instance.
    pub async fn new(connection_string: &str) -> Self {
        let client = redis::Client::open(connection_string).unwrap();
        let con = client.get_multiplexed_async_connection().await.unwrap();

        RedisService {
            multiplexed_connection: con,
        }
    }

    //noinspection RsSelfConvention
    /// Returns true if the key exists in Redis, false otherwise.
    pub async fn is_key_flagged(&mut self, key: &str) -> bool {
        let result: Result<bool, RedisError> = self.multiplexed_connection.get(key).await;
        result.unwrap_or(false)
    }

    /// Flags the key by setting it to true.
    pub async fn flag_key(&mut self, key: &str, ttl: u64) {
        let _ = self
            .multiplexed_connection
            .set_ex::<String, bool, bool>(key.to_string(), true, ttl)
            .await;
    }

    /// Publishes the data to the redis stream.
    /// Returns a `bool` that is true if the data was published and false otherwise.
    pub async fn publish<ST>(&mut self, stream_name: &str, data: &ST) -> bool
    where
        ST: Serialize,
    {
        let serialized_data = serde_json::to_string(&data).unwrap();
        let result = redis::cmd("XADD")
            .arg(stream_name)
            .arg("*")
            .arg("data")
            .arg(serialized_data)
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
    use post::NewsPost;
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
        let _ = RedisService::new(REDIS_CONNECTION_STRING).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_key_exists_false() {
        // Setup
        let random_post = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(REDIS_CONNECTION_STRING).await;

        // Test
        let result = service.is_key_flagged(&random_post).await;

        // Assert
        assert_eq!(result, false);
        cleanup(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_key_exists_true() {
        // Setup
        let random_post = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(REDIS_CONNECTION_STRING).await;
        service.flag_key(&random_post, 10).await;

        // Test
        let result = service.is_key_flagged(&random_post).await;

        // Assert
        assert_eq!(result, true);
        cleanup(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_publish() {
        // Setup
        let random_stream_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(REDIS_CONNECTION_STRING).await;

        // Test
        let post = NewsPost {
            image: Some(String::from("i")),
            title: Some(String::from("t")),
            summary: Some(String::from("s")),
            link: Some(String::from("l")),
            author: Some(String::from("a")),
        };
        let result = service.publish(&random_stream_name, &post).await;

        let stream_length: RedisResult<i32> = redis::cmd("XLEN")
            .arg(random_stream_name)
            .query_async(&mut service.multiplexed_connection)
            .await;

        // Assert
        assert_eq!(result, true);
        assert_eq!(stream_length, Ok(1));
        cleanup(&mut service).await;
    }
}
