use crate::utils::db;
use crate::utils::request::{HttpRequest, Redirector, Request};
use std::{
    io::{BufReader, prelude::*},
    net::TcpStream,
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
            }
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
        let referer: Option<String> = raw_http_request
            .iter()
            .find(|line: &&String| line.to_lowercase().starts_with("referer:"))
            .and_then(|line: &String| line.splitn(2, ':').nth(1))
            .map(|s| s.trim().to_string());
        let user_agent: Option<String> = raw_http_request
            .iter()
            .find(|line: &&String| line.to_lowercase().starts_with("user-agent:"))
            .and_then(|line: &String| line.splitn(2, ':').nth(1))
            .map(|s: &str| s.trim().to_string());
        let fallback_ip: Option<String> = stream
            .peer_addr()
            .ok()
            .map(|addr: std::net::SocketAddr| addr.ip().to_string());
        let real_ip: Option<String> = raw_http_request
            .iter()
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
        let mut uri: String = "".to_string();
        let mut url: String = "".to_string();

        let (status_line, content) = if request_line.starts_with("GET /")
            && (request_line.ends_with(" HTTP/1.1") || request_line.ends_with(" HTTP/2.0"))
        {
            let path: &str = request_line
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim()
                .strip_prefix('/')
                .unwrap_or("");

            match path.is_empty() {
                true => ("HTTP/1.1 200 OK".to_string(), "Hello World!".to_string()),
                false => {
                    uri = request.get_uri(path).await;
                    url = path.to_string();
                    if !uri.is_empty() {
                        let status_line =
                            format!("HTTP/1.1 302 Temporary Redirect\r\nLocation: {}", uri);
                        (status_line, format!("Redirecting to {}...", uri))
                    } else if !db::is_ok().await {
                        (
                            "HTTP/1.1 503 Service Unavailable".to_string(),
                            "Service Unavailable".to_string(),
                        )
                    } else {
                        (
                            "HTTP/1.1 404 NOT FOUND".to_string(),
                            "NOT FOUND".to_string(),
                        )
                    }
                }
            }
        } else {
            (
                "HTTP/1.1 400 BAD REQUEST".to_string(),
                "BAD REQUEST".to_string(),
            )
        };

        let length: usize = content.len();

        let response: String = format!(
            "{status_line}\r\nContent-Length: {length}\r\nContent-Type: text/plain\r\n\r\n{content}"
        );

        let time: String = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        println!("[{time}] Redirecting user({ip}) from {url}({referer}) to {uri}",
            time = time,
            ip = request.ip.unwrap_or("unknown".to_string()),
            referer = request.referer.unwrap_or("unknown".to_string()),
            url = url,
            uri = uri);

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
