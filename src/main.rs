use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

fn main() -> std::io::Result<()> {
    let port: u16 = 8080;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Listening on {}", addr);

    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        let stream = stream?;
        handle_conn(stream);
    }
    Ok(())
}

fn handle_conn(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let n = stream.read(&mut buffer).unwrap();

    let buf = &buffer[..n];

    if let Some(req) = Request::parse(buf) {
        println!("Headers: {:#?}", req.headers);
        println!("Body: {:?}", req.body);
    } else {
        println!("Failed to parse request");
    }

    let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n<p> Test </p>";
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub struct Request {
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    fn parse(buf: &[u8]) -> Option<Self> {
        let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
        let headers_bytes = &buf[..header_end];

        let body = &buf[header_end + 4..];
        let headers_str = std::str::from_utf8(headers_bytes).ok()?;
        let mut headers = HashMap::new();

        for line in headers_str.split("\r\n").skip(1) {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Some(Request {
            headers,
            body: String::from_utf8_lossy(body).to_string(),
        })
    }
}
