use deadpool_redis::{Pool, Runtime};

use crate::configuration::RedisSettings;

pub fn get_redis_pool(configuration: &RedisSettings) -> Pool {
    configuration
        .from_config()
        .builder()
        .expect("Could not create redis pool builder")
        .runtime(Runtime::Tokio1)
        .build()
        .expect("Could not create redis connection pool")
}
