use anyhow::anyhow;
use log::error;
use redis::Value::BulkString;
use redis::aio::MultiplexedConnection;
use redis::streams::StreamReadReply;
use redis::{AsyncCommands, RedisError, RedisResult, Value};
use serde::{Deserialize, Serialize};

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

    /// Creates a group for the given stream that consumes from the specified starting id.
    pub async fn create_group(
        &mut self,
        stream_name: &str,
        group_name: &str,
        starting_id: u32,
    ) -> Result<(), anyhow::Error> {
        redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(stream_name)
            .arg(group_name)
            .arg(starting_id)
            .exec_async(&mut self.multiplexed_connection)
            .await
            .map_err(|e| {
                anyhow!("failed to create group {group_name} for stream {stream_name}: {e}")
            })
    }

    /// Reads a stream from Redis and in a blocking fashion.
    ///
    /// Messages are acknowledged automatically when read.
    ///
    /// stream_name - is the name of the stream
    /// consumer_group - is the name of the consumer group
    /// consumer_name - is the name of the current consumer
    /// block_timeout - is the timeout in milliseconds to block for messages.
    pub async fn read_stream<T>(
        &mut self,
        stream_name: &str,
        consumer_group: &str,
        consumer_name: &str,
        block_timeout: u32,
    ) -> Result<T, anyhow::Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        let result: RedisResult<StreamReadReply> = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(consumer_group)
            .arg(consumer_name)
            .arg("BLOCK")
            .arg(block_timeout)
            .arg("COUNT")
            .arg(1)
            .arg("NOACK")
            .arg("STREAMS")
            .arg(stream_name)
            .arg(">")
            .query_async(&mut self.multiplexed_connection)
            .await;

        match result {
            Ok(data) => {
                let stream_data: Option<&Value> = data
                    .keys
                    .first()
                    .and_then(|f| f.ids.first().and_then(|i| i.map.get("data")));

                if let Some(BulkString(data)) = stream_data {
                    let string_data = std::str::from_utf8(data);
                    match string_data {
                        Ok(string_data) => Ok(serde_json::from_str(string_data)?),
                        Err(err) => Err(anyhow!("can't convert data to string: {err}")),
                    }
                } else {
                    Err(anyhow!(
                        "invalid type read from streams, expected BulkString"
                    ))
                }
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use post::NewsPost;
    use rand::distributions::{Alphanumeric, DistString};
    use redis::RedisResult;
    use serial_test::serial;
    use std::env;

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
        let redis_connection_string: String = env::var("REDIS_TESTS_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "redis://localhost:6379".to_string());

        let _ = RedisService::new(&redis_connection_string).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_key_exists_false() {
        // Setup
        let redis_connection_string: String = env::var("REDIS_TESTS_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "redis://localhost:6379".to_string());
        let random_post = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(&redis_connection_string).await;

        // Test
        let result = service.is_key_flagged(&random_post).await;

        // Assert
        assert!(!result);
        cleanup(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_key_exists_true() {
        // Setup
        let redis_connection_string: String = env::var("REDIS_TESTS_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "redis://localhost:6379".to_string());
        let random_post = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(&redis_connection_string).await;
        service.flag_key(&random_post, 10).await;

        // Test
        let result = service.is_key_flagged(&random_post).await;

        // Assert
        assert!(result);
        cleanup(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_publish() {
        // Setup
        let redis_connection_string: String = env::var("REDIS_TESTS_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "redis://localhost:6379".to_string());
        let random_stream_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(&redis_connection_string).await;

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
        assert!(result);
        assert_eq!(stream_length, Ok(1));
        cleanup(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_redis_service_read() -> Result<(), anyhow::Error> {
        // Setup
        let redis_connection_string: String = env::var("REDIS_TESTS_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "redis://localhost:6379".to_string());
        let random_stream_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

        let mut service = RedisService::new(&redis_connection_string).await;
        let post = NewsPost {
            image: Some(String::from("i")),
            title: Some(String::from("t")),
            summary: Some(String::from("s")),
            link: Some(String::from("l")),
            author: Some(String::from("a")),
        };
        let _ = service.publish(&random_stream_name, &post).await;

        // Test
        service
            .create_group(&random_stream_name, &random_stream_name, 0)
            .await?;
        let result = service
            .read_stream::<NewsPost>(
                &random_stream_name,
                &random_stream_name,
                &random_stream_name,
                10_000,
            )
            .await?;

        // Assert
        assert_eq!(result, post);
        cleanup(&mut service).await;
        Ok(())
    }
}
