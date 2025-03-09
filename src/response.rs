use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct Response {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode, mut headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        headers.insert("Content-Length".into(), body.len().to_string());
        headers.insert("Connection".into(), "close".into());

        Response {
            status,
            headers,
            body,
        }
    }

    #[inline]
    pub fn status(&self) -> &StatusCode {
        &self.status
    }
    #[inline]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    #[inline]
    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status as u16,
            self.status().as_str()
        );

        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str("\r\n");

        let mut response_bytes = response.into_bytes();
        response_bytes.extend_from_slice(&self.body);
        response_bytes
    }
}

impl From<(StatusCode, String)> for Response {
    fn from(value: (StatusCode, String)) -> Self {
        Response::new(value.0, HashMap::default(), value.1.as_bytes().to_vec())
    }
}

impl From<(StatusCode, &str)> for Response {
    fn from(value: (StatusCode, &str)) -> Self {
        Response::new(value.0, HashMap::default(), value.1.as_bytes().to_vec())
    }
}

pub trait IntoResponse {
    fn into_response(&self) -> Response;
}
impl IntoResponse for (StatusCode, String) {
    fn into_response(&self) -> Response {
        Response::new(self.0, HashMap::default(), self.1.as_bytes().to_vec())
    }
}
impl IntoResponse for (StatusCode, &str) {
    fn into_response(&self) -> Response {
        Response::new(self.0, HashMap::default(), self.1.as_bytes().to_vec())
    }
}
impl IntoResponse for &str {
    fn into_response(&self) -> Response {
        Response::new(StatusCode::OK, HashMap::default(), self.as_bytes().to_vec())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StatusCode {
    OK = 200,
    Created = 201,
    Accepted = 202,
    MovedPermanently = 301,
    Found = 302,
    NotModified = 304,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    InternalServerError = 500,
}

impl StatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusCode::OK => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::NotModified => "Not Modified",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::PaymentRequired => "Payment Required",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::InternalServerError => "Internal Server Error",
        }
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
