use request::{Method, Request};
use response::{IntoResponse, Response, StatusCode};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
};

mod request;
mod response;
// Exercise 1:
//
// 1. Add a state variable on the server that keeps track of String data that can be set and cleared. (This mimicks a database)
// 2. Create 4 routes:
//    - GET /state: returns the current state
//    - POST /state: sets the state to the body of the request
//    - DELETE /state: clears the state
//    - GET /: returns a simple "Hello, world!" response
//    - Any other route returns a 404 response

fn main() -> std::io::Result<()> {
    let port: u16 = 8080;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Listening on {}", addr);

    let listener = TcpListener::bind(addr)?;

    let mut state = String::new();

    for stream in listener.incoming() {
        let conn = stream?;
        handle_conn(conn, &mut state);
    }
    Ok(())
}

fn handle_conn(mut stream: TcpStream, state: &mut String) {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).unwrap();
    let buffer = &buffer[..n];

    if let Some(req) = Request::parse(&buffer) {
        let res = create_response(req, state);
        stream.write_all(&res.into_response().to_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        println!("Failed to parse request");
    }
}

fn create_response(req: Request, state: &mut String) -> impl IntoResponse {
    match (req.method(), req.path().as_str()) {
        (Method::GET, "/") => (StatusCode::OK, "Hello, world!".to_string()),
        (Method::GET, "/state") => (StatusCode::Created, state.to_owned()),
        (Method::POST, "/state") => {
            *state = req.body().to_owned();
            (StatusCode::Created, state.to_owned())
        }
        (Method::DELETE, "/state") => {
            *state = String::new();
            (StatusCode::OK, state.clone())
        }
        _ => (StatusCode::NotFound, "Not Found".to_string()),
    }
}
