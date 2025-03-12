use crate::{
    request::{Method, Request},
    response::{IntoResponse, Response, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::RwLock,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub completed_at: bool,
    pub title: String,
}

pub async fn get_state(_: Request, state: Arc<RwLock<Vec<Todo>>>) -> impl IntoResponse {
    let state = state.read().await;
    let todos = state.iter().map(|todo| todo).collect::<Vec<_>>();
    let todos = serde_json::to_string(&todos).expect("Failed to serialize todos");

    let headers = HashMap::from([("Content-Type".to_string(), "application/json".to_string())]);
    (StatusCode::OK, headers, todos)
}

type Handler<S> =
    Box<(dyn Fn(Request, S) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync)>;

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

    pub fn route<F, Fut, T>(mut self, method: Method, path: &'static str, handler: F) -> Self
    where
        F: (Fn(Request, S) -> Fut) + Send + Sync + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: IntoResponse,
    {
        let handler = Arc::new(handler);
        self.routes.insert(
            (path, method),
            Box::new(move |req, state| {
                let handler = handler.clone();
                let state = state.clone();

                Box::pin(async move { (handler)(req, state).await.into_response() })
            }),
        );

        self
    }

    async fn handle(&self, req: Request) -> Response {
        for ((path, method), handler) in &self.routes {
            if method == req.method() && *path == req.path() {
                return handler(req, self.state.clone()).await;
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
    pub async fn run(self, listener: TcpListener) -> std::io::Result<()> {
        loop {
            let (mut stream, _) = listener.accept().await?;
            let router = self.router.clone();

            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                let n = stream.read(&mut buffer).await.unwrap();
                let buffer = &buffer[..n];

                if let Some(req) = Request::parse(buffer) {
                    let res = router.handle(req).await;
                    if let Err(e) = stream.write_all(&res.to_bytes()).await {
                        eprintln!("Failed to write to stream: {}", e);
                    }
                } else {
                    eprintln!("Failed to parse request");
                    let res: Response = (StatusCode::BadRequest, "Bad Request").into();
                    if let Err(e) = stream.write_all(&res.to_bytes()).await {
                        eprintln!("Failed to write error response: {}", e);
                    }
                }
            });
        }
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
