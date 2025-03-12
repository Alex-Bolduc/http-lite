use request::{Method, Request};
use response::{IntoResponse, Response, StatusCode};
use server::{HttpServer, Router, Todo, get_state};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Listening on {}", addr);

    let state: Arc<RwLock<Vec<Todo>>> = Arc::new(RwLock::new(Vec::new()));

    let router = Router::new(state)
        .route(Method::GET, "/", async |_, _| {
            (StatusCode::OK, "Hello, world!")
        })
        .route(Method::GET, "/todos", get_state)
        .route(Method::POST, "/todos", async move |req, state| {
            let body = req.body().to_string();
            let todo = match serde_json::from_str(&body) {
                Ok(todo) => todo,
                Err(_) => {
                    return (StatusCode::BadRequest, "Invalid body".to_string());
                }
            };

            let mut state = state.write().await;
            state.push(todo);

            (StatusCode::OK, "State updated".to_string())
        })
        .route(Method::DELETE, "/todos", async |_, state| {
            let mut state = state.write().await;
            state.clear();
            (StatusCode::OK, "State cleared")
        })
        .route(Method::GET, "/index.html", static_index);
    let server = HttpServer::new(router);
    server.run(listener).await
}

async fn static_index(_: Request, _: Arc<RwLock<Vec<Todo>>>) -> impl IntoResponse {
    let data = include_str!("../index.html");
    let headers: HashMap<String, String> =
        HashMap::from([("Content-Type".into(), "text/html".into())]);
    (StatusCode::OK, headers, data)
}
