use std::env;
use mysql_async::{Pool, Conn, prelude::*};

pub struct DB;

impl DB {
    pub async fn create(&self) -> Result<Conn, Box<dyn std::error::Error>> {
        let host: String = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let user: String = env::var("DB_USER").unwrap_or_else(|_| "username".to_string());
        let pass: String = env::var("DB_PASS").unwrap_or_else(|_| "password".to_string());
        let port: String = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
        let name: String = env::var("DB_NAME").unwrap_or_else(|_| "database".to_string());

        let url = format!("mysql://{}:{}@{}:{}/{}", user, pass, host, port, name);
        let opts = mysql_async::Opts::from_url(&url)?;
        let pool = Pool::new(opts);
        let conn = pool.get_conn().await?;
        println!("Connecting to MySQL at {}:{}", host, port);
        
        Ok(conn)
    }
}
