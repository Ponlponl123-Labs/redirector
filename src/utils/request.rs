use mysql_async::prelude::Queryable;
use ::redis::Commands;

use crate::utils::{db, redis};

pub type HttpRequest = Vec<String>;

pub struct Request {
    pub http: HttpRequest,
    pub referer: Option<String>,
    pub user_agent: Option<String>,
    pub ip: Option<String>
}

pub trait Redirector {
    async fn get_uri(&self, id: &str) -> String;
}

impl Redirector for Request {
    async fn get_uri(&self, id: &str) -> String {
        let id_clone: String = id.to_string();
        let http_clone: Vec<String> = self.http.clone();
        let ip_clone: String = self.ip.clone().unwrap_or("".to_string());
        let referer: Option<String> = self.referer.clone();
        let user_agent: Option<String> = self.user_agent.clone();
        let _ = tokio::spawn(async move {
            let req = Request { http: http_clone, ip: Some(ip_clone), referer, user_agent };
            req.send_to_db(&id_clone).await;
        });
        let redis_conn: Result<::redis::Connection, ::redis::RedisError> = redis::get_connection();
        match redis_conn {
            Ok(mut conn) => {
                let uri: Option<String> = conn.get(format!("ponlponl123:apps:redirector:url:{}", id)).unwrap();
                if !uri.is_none() && !uri.clone().unwrap().is_empty() {
                    return uri.unwrap();
                }
            },
            Err(_) => {
                println!("Cannot connect to Redis, fetch directly from DB");
            }
        }
        
        let db = db::get_pool().await;
        let db_conn = db.get_conn().await;
        if !db_conn.is_ok() {
            return "".to_string();
        }

        let result: Vec<mysql_async::Row> = db_conn.unwrap().exec("SELECT uri, `url`, disabled FROM endpoint WHERE `url` = ? AND disabled = 0 LIMIT 1", (id,)).await.unwrap();
        // println!("DB Query result: {:?}", result);
        if let Some(row) = result.into_iter().next() {
            let uri: String = row.get("uri").unwrap_or_default();
            if let Ok(mut conn) = redis::get_connection() {
                let _: () = conn
                    .set_ex(format!("ponlponl123:apps:redirector:url:{}", id), uri.clone(), 60 * 60 * 3)
                    .unwrap();
            } else {
                println!("DB fetched but cannot store to Redis, ignored.");
            }
            return uri;
        }
        return "".to_string();
    }
}

pub trait Analyze {
    async fn send_to_db(&self, id: &str);
}

impl Analyze for Request {
    async fn send_to_db(&self, id: &str) {
        if !db::is_ok().await {
            println!("Cannot store traffic log, Database pool connection is not ok");
            return;
        }

        let db: &'static mysql_async::Pool = db::get_pool().await;
        let mut conn: mysql_async::Conn = db.get_conn().await.unwrap();

        let http_string: String = self.http.join(" "); // Convert Vec<String> to a single String
        let result: Result<(), mysql_async::Error> = conn.exec_drop("INSERT INTO requests (`ip`, `string`, `referer`, `user_agent`, `to`) VALUES (?, ?, ?, ?, ?)",
            (&self.ip, http_string, &self.referer, &self.user_agent, id))
            .await;

        if let Err(_e) = result {
            // eprintln!("Failed to insert request: {e}");
            return;
        }

        let _ = conn.exec_drop("UPDATE endpoint SET used = used + 1 WHERE url = ?", (id,))
            .await;
    }
}