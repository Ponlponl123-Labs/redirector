use std::env;
use mysql_async::{Pool, Opts};
use tokio::sync::OnceCell;
use urlencoding::encode;

static DB_POOL: OnceCell<Pool> = OnceCell::const_new();

async fn create_pool() -> Pool {
    let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("DB_USER").unwrap_or_else(|_| "username".to_string());
    let pass = env::var("DB_PASS").unwrap_or_else(|_| "password".to_string());
    let port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let name = env::var("DB_NAME").unwrap_or_else(|_| "database".to_string());

    let port = port.trim();
    // println!("Port after trim: '{}'", port);

    let port_num: u16 = port.parse().expect("DB_PORT must be a valid port number");

    let user_enc = encode(&user);
    let pass_enc = encode(&pass);

    let url = format!("mysql://{}:{}@{}:{}/{}", user_enc, pass_enc, host, port_num, name);
    // println!("DB URL: {}", url);

    let opts = Opts::from_url(&url).expect("Invalid DB URL");
    
    println!("Connecting to MySQL at {}:{}...", host, port);
    let pool = Pool::new(opts);

    // Do a test connection here
    match pool.get_conn().await {
        Ok(conn) => {
            // You can drop the conn, it goes back to the pool
            drop(conn);
            println!("Connected to MySQL!");
        }
        Err(e) => {
            eprintln!("Failed to connect to DB: {e}");
            std::process::exit(1);
        }
    }

    pool
}

/// Public function to get global pool
pub async fn get_pool() -> &'static Pool {
    DB_POOL.get_or_init(create_pool).await
}

pub async fn is_ok() -> bool {
    get_pool().await.get_conn().await.is_ok()
}