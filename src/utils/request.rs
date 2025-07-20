use mysql_async::prelude::Queryable;
use ::redis::Commands;

use crate::utils::{db, redis};

pub type HttpRequest = Vec<String>;

pub struct Request {
    pub http: HttpRequest
}

pub trait Redirector {
    async fn get_uri(&self, id: &str) -> String;
}

impl Redirector for Request {
    async fn get_uri(&self, id: &str) -> String {
        let http_clone = self.http.clone();
        let _ = tokio::spawn(async move {
            let req = Request { http: http_clone };
            req.send_to_db().await;
        });
        let redis_client: Result<::redis::Connection, ::redis::RedisError> = redis::get_connection();
        match redis_client {
            Ok(mut conn) => {
                let uri: String = conn.get(format!("ponlponl123:apps:redirector:url:{}", id)).unwrap();
                if !uri.is_empty() {
                    return uri;
                }

                let db = db::get_pool().await;
                let db_conn = db.get_conn().await;
                if !db_conn.is_ok() {
                    return "".to_string();
                }

                let http_string = self.http.join(" "); // Convert Vec<String> to a single String
                let result: Vec<mysql_async::Row> = db_conn.unwrap().exec("SELECT uri, url FROM endpoint WHERE string = ? LIMIT 1", (http_string,)).await.unwrap();
                println!("DB Query result: {:?}", result);
                if result.len() > 0 {
                    let uri: String = format!("{:?}", &result[0]);
                    let _: () = conn.set_ex(format!("ponlponl123:apps:redirector:url:{}", id), uri.clone(), 60 * 60 * 3).unwrap();
                    return uri;
                }
                return "".to_string();
            },
            Err(_) => {
                let db = db::get_pool().await;
                let db_conn = db.get_conn().await;
                if !db_conn.is_ok() {
                    return "".to_string();
                }

                let http_string = self.http.join(" "); // Convert Vec<String> to a single String
                let result: Vec<mysql_async::Row> = db_conn.unwrap().exec("SELECT uri, url FROM endpoint WHERE string = ? LIMIT 1", (http_string,)).await.unwrap();
                println!("DB Query result: {:?}", result);
                if result.len() > 0 {
                    let uri: String = format!("{:?}", &result[0]);
                    return uri;
                }
                return "".to_string();
            }
        }
    }
}

pub trait Analyze {
    async fn send_to_db(&self);
}

impl Analyze for Request {
    async fn send_to_db(&self) {
        let db = db::get_pool().await;
        let conn = db.get_conn().await;
        if !conn.is_ok() {
            println!("Cannot store traffic log, Database pool connection is not ok");
            return;
        }

        let http_string = self.http.join(" "); // Convert Vec<String> to a single String
        let result: Vec<mysql_async::Row> = conn.unwrap().exec("INSERT INTO requests (string) VALUES (?)", (http_string,)).await.unwrap();
        println!("DB Query result: {:?}", result);
    }
}