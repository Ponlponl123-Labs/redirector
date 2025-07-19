use std::{env, sync::LazyLock};
use redis::{Client, Connection, RedisResult};

static REDIS_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    let pass = env::var("REDIS_PASS").unwrap_or_else(|_| "".to_string());
    let db = env::var("REDIS_DB").unwrap_or_else(|_| "0".to_string());

    let url = if pass.is_empty() {
        format!("redis://{}:{}/{}", host, port, db)
    } else {
        format!("redis://:{}@{}:{}/{}", pass, host, port, db)
    };

    println!("Connecting to Redis at {}:{}...", host, port);
    Client::open(url).expect("Failed to create Redis client")
});

/// Get a new Redis connection from the global client
pub fn get_connection() -> RedisResult<Connection> {
    REDIS_CLIENT.get_connection()
}