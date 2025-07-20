use crate::utils::db;
use crate::utils::request::{HttpRequest, Redirector, Request};
use std::{
    io::{prelude::*, BufReader}, net::{TcpStream}
};

pub struct Handler;

impl Handler {
    pub async fn handle_connection(mut stream: TcpStream) {
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
        
        let raw_http_request: HttpRequest = lines
            .by_ref()
            .filter_map(|result: Result<String, std::io::Error>| result.ok())
            .take_while(|line: &String| !line.is_empty())
            .collect();
        let referer: Option<String> = raw_http_request.iter()
            .find(|line: &&String| line.to_lowercase().starts_with("referer:"))
            .and_then(|line: &String| line.splitn(2, ':').nth(1))
            .map(|s| s.trim().to_string());
        let user_agent: Option<String> = raw_http_request.iter()
            .find(|line: &&String| line.to_lowercase().starts_with("user-agent:"))
            .and_then(|line: &String| line.splitn(2, ':').nth(1))
            .map(|s: &str| s.trim().to_string());
        let fallback_ip: Option<String> = stream.peer_addr().ok().map(|addr: std::net::SocketAddr| addr.ip().to_string());
        let real_ip: Option<String> = raw_http_request.iter()
            .find(|line: &&String| line.to_lowercase().starts_with("x-forwarded-for:"))
            .and_then(|line: &String| line.splitn(2, ':').nth(1))
            .map(|s| s.trim().to_string())
            .or(fallback_ip);

        let request: Request = Request {
            http: raw_http_request,
            ip: real_ip,
            referer,
            user_agent,
        };
        
        let (status_line, content) = if request_line.starts_with("GET /") && (request_line.ends_with(" HTTP/1.1") || request_line.ends_with(" HTTP/2.0")) {
            let path: &str = request_line
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim()
                .strip_prefix('/')
                .unwrap_or("");
            
            match path.is_empty() {
                true => ("HTTP/1.1 200 OK", "Hello World!".to_string()),
                false => {
                    match db::is_ok().await {
                        true => {
                            let uri: String = request.get_uri(path).await;
                            if !uri.is_empty() {
                                ("HTTP/1.1 302 OK", format!("Redirecting to {}...", uri))
                            } else {
                                ("HTTP/1.1 404 NOT FOUND", format!("Endpoint [{}] not valid.", path))
                            }
                        }
                        false => {
                            ("HTTP/1.1 503 Service Unavaliable", "Service Unavailable".to_string())
                        }
                    }
                }
            }
        } else {
            ("HTTP/1.1 400 BAD REQUEST", "BAD REQUEST".to_string())
        };

        let length: usize = content.len();

        let response: String = format!(
            "{status_line}\r\nContent-Length: {length}\r\nContent-Type: text/plain\r\n\r\n{content}"
        );

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}