use std::{collections::HashMap, str::FromStr};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    POST,
    DELETE,
}

impl FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "DELETE" => Ok(Method::DELETE),
            _ => Err(format!("Invalid method: {}", s)),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    headers: HashMap<String, String>,
    body: String,
    method: Method,
    path: String,
    version: String,
}

impl Request {
    pub fn parse(buf: &[u8]) -> Option<Self> {
        let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
        let headers_bytes = &buf[..header_end];

        let body = &buf[header_end + 4..];
        let headers_str = std::str::from_utf8(headers_bytes).ok()?;
        let mut lines = headers_str.split("\r\n");

        let request_line = lines.next()?.trim();
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.parse().ok()?;
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        let mut headers = HashMap::new();

        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Some(Request {
            headers,
            body: String::from_utf8_lossy(body).to_string(),
            method,
            path,
            version,
        })
    }
    #[inline]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    #[inline]
    pub fn body(&self) -> &String {
        &self.body
    }
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }
    #[inline]
    pub fn path(&self) -> &String {
        &self.path
    }
}
