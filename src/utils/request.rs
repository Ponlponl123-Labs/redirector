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
                let result: Vec<mysql_async::Row> = db_conn.unwrap().exec("SELECT uri FROM endpoint WHERE url = ? AND disabled = 0 LIMIT 1", (http_string,)).await.unwrap();
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
                let result: Vec<mysql_async::Row> = db_conn.unwrap().exec("SELECT uri FROM endpoint WHERE url = ? AND disabled = 0 LIMIT 1", (http_string,)).await.unwrap();
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
    async fn send_to_db(&self, id: &str);
}

impl Analyze for Request {
    async fn send_to_db(&self, id: &str) {
        if !db::is_ok().await {
            println!("Cannot store traffic log, Database pool connection is not ok");
            return;
        }

        let db = db::get_pool().await;
        let mut conn = db.get_conn().await.unwrap();

        let http_string = self.http.join(" "); // Convert Vec<String> to a single String
        let _ = conn.exec_drop("INSERT INTO requests (ip, string, referer, user_agent, to) VALUES (?, ?, ?, ?, ?)",
            (&self.ip, http_string, &self.referer, &self.user_agent, id))
            .await
            .unwrap();

        let _ = conn.exec_drop("UPDATE endpoint SET used = used + 1 WHERE url = ?", (id,))
            .await
            .unwrap();
    }
}