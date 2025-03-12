use request::{Method, Request};
use response::{IntoResponse, Response, StatusCode};
use server::{HttpServer, Router, Todo, get_state};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex, RwLock},
};

mod request;
mod response;
mod server;

// Exercise 1:
//
// 1. Add a state variable on the server that keeps track of String data that can be set and cleared. (This mimicks a database)
// 2. Create 4 routes:
//    - GET /state: returns the current state
//    - POST /state: sets the state to the body of the request
//    - DELETE /state: clears the state
//    - GET /: returns a simple "Hello, world!" response
//    - Any other route returns a 404 response

type State = Arc<RwLock<String>>;

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)?;

    println!("Listening on {}", addr);

    let state: Arc<RwLock<Vec<Todo>>> = Arc::new(RwLock::new(Vec::new()));

    let mut router = Router::new(state)
        .route(Method::GET, "/", |_, _| (StatusCode::OK, "Hello, world!"))
        .route(Method::GET, "/todos", get_state)
        .route(Method::POST, "/todos", move |req, state| {
            let body = req.body().to_string();
            let todo = match serde_json::from_str(&body) {
                Ok(todo) => todo,
                Err(_) => {
                    return (StatusCode::BadRequest, "Invalid body".to_string());
                }
            };

            let mut state = state.write().unwrap();
            state.push(todo);

            (StatusCode::OK, "State updated".to_string())
        })
        .route(Method::DELETE, "/todos", |_, state| {
            let mut state = state.write().unwrap();
            state.clear();
            (StatusCode::OK, "State cleared")
        })
        .route(Method::GET, "/index.html", static_index);
    let server = HttpServer::new(router);
    server.run(listener)
}

fn static_index(_: Request, _: Arc<RwLock<Vec<Todo>>>) -> impl IntoResponse {
    let data = include_str!("../index.html");
    let headers: HashMap<String, String> =
        HashMap::from([("Content-Type".into(), "text/html".into())]);
    (StatusCode::OK, headers, data)
}

fn handle_conn(mut stream: TcpStream, state: State) {
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

fn create_response(req: Request, state: State) -> impl IntoResponse {
    match (req.method(), req.path().as_str()) {
        (Method::GET, "/") => (StatusCode::OK, "Hello, world!".to_string()),
        (Method::GET, "/state") => {
            let guard = state.read().unwrap();

            (StatusCode::Created, guard.to_owned())
        }
        (Method::POST, "/state") => {
            let mut guard = state.write().unwrap();

            *guard = req.body().to_owned();
            (StatusCode::Created, guard.to_owned())
        }
        (Method::DELETE, "/state") => {
            let mut guard = state.write().unwrap();

            *guard = String::new();

            (StatusCode::OK, guard.clone())
        }
        _ => (StatusCode::NotFound, "Not Found".to_string()),
    }
}
