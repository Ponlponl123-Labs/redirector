use std::env;
use mysql_async::{Pool, Opts};
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<Pool> = OnceCell::const_new();

async fn init_pool() -> Pool {
    let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("DB_USER").unwrap_or_else(|_| "username".to_string());
    let pass = env::var("DB_PASS").unwrap_or_else(|_| "password".to_string());
    let port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let name = env::var("DB_NAME").unwrap_or_else(|_| "database".to_string());

    let url = format!("mysql://{}:{}@{}:{}/{}", user, pass, host, port, name);
    let opts = Opts::from_url(&url).expect("Invalid DB URL");

    println!("Connecting to MySQL at {}:{}...", host, port);
    Pool::new(opts)
}

/// Public function to get global pool
pub async fn get_pool() -> &'static Pool {
    DB_POOL.get_or_init(init_pool).await
}