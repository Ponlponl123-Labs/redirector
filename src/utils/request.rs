use crate::utils::db;

pub type HttpRequest = Vec<String>;

pub struct Request {
    pub http: HttpRequest
}

pub trait Analyze {
    fn send_to_db(self);
}

impl Analyze for Request {
    fn send_to_db(self) {
    }
}