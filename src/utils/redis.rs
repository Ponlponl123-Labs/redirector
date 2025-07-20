use std::{env, sync::LazyLock};
use std::sync::RwLock;
use redis::{Client, Connection, RedisResult};

static REDIS_CLIENT: LazyLock<RwLock<Client>> = LazyLock::new(|| {
    RwLock::new(create_new_connection())
});

pub fn get_connection() -> RedisResult<Connection> {
    // Try read lock first
    if let Ok(client) = REDIS_CLIENT.read() {
        if let Ok(conn) = client.get_connection() {
            return Ok(conn);
        }
    }

    // If it failed, recreate and replace
    println!("Connect to Redis failed, replacing global client...");
    let new_client = create_new_connection();

    if let Ok(mut client) = REDIS_CLIENT.write() {
        *client = new_client;
        return client.get_connection();
    }

    Err(redis::RedisError::from((
        redis::ErrorKind::IoError,
        "Failed to acquire write lock for Redis client",
    )))
}


fn create_new_connection() -> Client {
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
}