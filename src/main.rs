mod utils;
mod handler;
use handler::Handler;
use utils::{db, redis};
use redirector::ThreadPool;
use std::{
    env, net::{TcpListener}
};

#[tokio::main]
async fn main() {
    let port: String = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener: TcpListener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    let is_single_thread: bool = env::var("SINGLE_THREADED").unwrap_or_else(|_| "false".to_string()) == "true";
    let mut _redis_client: Result<::redis::Connection, ::redis::RedisError> = redis::get_connection();
    let mut _db_client = db::get_pool();

    if is_single_thread {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    Handler::handle_connection(stream).await;
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    } else {
        let max_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let pool: ThreadPool = ThreadPool::new(max_threads);
        let handle = tokio::runtime::Handle::current();
        
        println!("Server listening on port {}", listener.local_addr().unwrap().port());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handle = handle.clone();
                    pool.execute(move || {
                        handle.block_on(Handler::handle_connection(stream));
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }
}