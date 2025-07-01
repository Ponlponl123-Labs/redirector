use redirector::ThreadPool;
mod health_check;
use health_check::{get_service_config, check_service_health};
use std::{
    env, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};

fn handle_connection(mut stream: TcpStream) {
    let buf_reader: BufReader<&TcpStream> = BufReader::new(&stream);
    let mut lines: std::io::Lines<BufReader<&TcpStream>> = buf_reader.lines();
    
    let request_line: String = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(_)) => {
            let response: &'static str = "HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 19\r\nContent-Type: text/plain\r\n\r\nError reading request";
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
            return;
        },
        None => {
            let response: &'static str = "HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 13\r\nContent-Type: text/plain\r\n\r\nEmpty request";
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
            return;
        }
    };
    
    let _http_request: Vec<_> = lines
        .by_ref()
        .filter_map(|result: Result<String, std::io::Error>| result.ok())
        .take_while(|line: &String| !line.is_empty())
        .collect();
    
    let (status_line, content) = if request_line.starts_with("GET /") && (request_line.ends_with(" HTTP/1.1") || request_line.ends_with(" HTTP/2.0")) {
        let path: &str = request_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .trim()
            .strip_prefix('/')
            .unwrap_or("");
        
        match path {
            "" => ("HTTP/1.1 200 OK", "Hello World!".to_string()),
            service_name => {
                match get_service_config(service_name) {
                    Some(config) => {
                        if check_service_health(&config) {
                            ("HTTP/1.1 200 OK", format!("Service {} is healthy", service_name))
                        } else {
                            ("HTTP/1.1 503 Service Unavailable", format!("Service {} is not responding", service_name))
                        }
                    },
                    None => ("HTTP/1.1 404 NOT FOUND", format!("Service {} not valid", service_name)),
                }
            }
        }
    } else {
        ("HTTP/1.1 400 BAD REQUEST", "Invalid request".to_string())
    };

    let length: usize = content.len();

    let response: String = format!(
        "{status_line}\r\nContent-Length: {length}\r\nContent-Type: text/plain\r\n\r\n{content}"
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let port: String = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener: TcpListener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    let is_single_thread: bool = env::var("SINGLE_THREADED").unwrap_or_else(|_| "false".to_string()) == "true";

    if is_single_thread {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_connection(stream);
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    } else {
        let max_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let pool: ThreadPool = ThreadPool::new(max_threads);
        
        println!("Server listening on port {}", listener.local_addr().unwrap().port());

        for stream in listener.incoming() {
            let pool: &ThreadPool = &pool;
            match stream {
                Ok(stream) => {
                    pool.execute(move || {
                        handle_connection(stream);
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }
}