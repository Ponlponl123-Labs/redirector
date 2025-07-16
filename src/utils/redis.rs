use std::env;
use redis::{Connection, RedisResult};

pub struct Redis;

impl Redis {
    pub fn create(&self) -> RedisResult<Connection> {
        let host: String = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: String = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let pass: String = env::var("REDIS_PASS").unwrap_or_else(|_| "".to_string());
        let db: String = env::var("REDIS_DB").unwrap_or_else(|_| "0".to_string());
        
        let url = if pass.is_empty() {
            format!("redis://{}:{}/{}", host, port, db)
        } else {
            format!("redis://:{}@{}:{}/{}", pass, host, port, db)
        };

        let client = redis::Client::open(url)?;
        let con = client.get_connection()?;

        Ok(con)
    }
}
