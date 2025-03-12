use crate::{
    request::{Method, Request},
    response::{IntoResponse, Response, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::{Arc, RwLock},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub completed_at: bool,
    pub title: String,
}

pub fn get_state(_: Request, state: Arc<RwLock<Vec<Todo>>>) -> impl IntoResponse {
    let state = state.read().unwrap();
    let todos = state.iter().map(|todo| todo).collect::<Vec<_>>();
    let todos = serde_json::to_string(&todos).expect("Failed to serialize todos");

    let headers = HashMap::from([("Content-Type".to_string(), "application/json".to_string())]);

    (StatusCode::OK, headers, todos)
}
// fn static_index(_: Request, _: Arc<Mutex<String>>) -> impl IntoResponse {
//     let data = include_str!("../../static/index.html");
//     let headers: HashMap<String, String> =
//         HashMap::from([("Content-Type".into(), "text/html".into())]);
//     (StatusCode::OK, headers, data)
// }

type Handler<S> = Box<(dyn Fn(Request, S) -> Response + Send + Sync)>;

pub struct Router<S> {
    routes: HashMap<(&'static str, Method), Handler<S>>,
    state: S,
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(state: S) -> Self {
        Router {
            routes: HashMap::new(),
            state,
        }
    }

    pub fn route<F, T>(mut self, method: Method, path: &'static str, handler: F) -> Self
    where
        F: (Fn(Request, S) -> T) + Send + Sync + 'static,
        T: IntoResponse,
    {
        self.routes.insert(
            (path, method),
            Box::new(move |req, state| handler(req, state).into_response()),
        );
        self
    }

    fn handle(&self, req: Request) -> Response {
        for ((path, method), handler) in &self.routes {
            if method == req.method() && *path == req.path() {
                return handler(req, self.state.clone());
            }
        }
        (StatusCode::NotFound, "Not found").into()
    }
}

pub struct HttpServer<S> {
    router: Arc<Router<S>>,
}

impl<S> HttpServer<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(router: Router<S>) -> Self {
        HttpServer {
            router: Arc::new(router),
        }
    }

    pub fn run(self, listener: TcpListener) -> std::io::Result<()> {
        for stream in listener.incoming() {
            match stream {
                Ok(mut socket) => {
                    let router = self.router.clone();
                    std::thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        let n = socket.read(&mut buffer).unwrap();
                        let buffer = &buffer[..n];

                        if let Some(req) = Request::parse(buffer) {
                            let res = router.handle(req);
                            if let Err(e) = socket.write_all(&res.to_bytes()) {
                                eprintln!("Failed to write to socket: {}", e);
                            }
                        } else {
                            eprintln!("Failed to parse request");
                            let res: Response = (StatusCode::BadRequest, "Bad Request").into();
                            if let Err(e) = socket.write_all(&res.to_bytes()) {
                                eprintln!("Failed to write error response: {}", e);
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn it_can_serialize() {
        let todo = Todo {
            id: "123".to_string(),
            completed_at: false,
            title: "Gg".to_string(),
        };
        let str = serde_json::to_string(&todo).expect("Failed to serialize");

        assert_eq!(
            str,
            "{\"id\":\"123\",\"completed_at\":false,\"title\":\"Gg\"}"
        );
    }
    #[test]
    fn it_can_deserialize() {
        let str = "{\"id\":\"123\",\"completed_at\":false,\"title\":\"Gg\"}";

        let todo: Todo = serde_json::from_str(&str).expect("Failed to deserialize");

        assert_eq!(
            todo,
            Todo {
                id: "123".to_string(),
                completed_at: false,
                title: "Gg".to_string(),
            }
        );
    }
}
