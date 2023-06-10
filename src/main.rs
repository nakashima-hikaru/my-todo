mod repositories;
mod handlers;

use axum::routing::{get, post};
use axum::routing::Router;
use std::net::SocketAddr;
use std::env;
use std::sync::{Arc};
use axum::{Extension};
use crate::handlers::create_todo;
use crate::repositories::{TodoRepository, TodoRepositoryForMemory};


#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let log_level = env::var("RUST_LOG").unwrap_or("debug".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    tracing::debug!("http://localhost:3000");
    axum::Server::bind(&addr).serve(app.into_make_service()).await
}

fn create_app<T: TodoRepository>(repository: T) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<T>))
        .layer(Extension(Arc::new(repository)))
}

async fn root() -> &'static str {
    "Hello, World!"
}


#[cfg(test)]
mod test {
    use super::*;
    use axum::body::Body;
    use axum::http;
    use http::{Request};

    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let repository = TodoRepositoryForMemory::new();
        let res = create_app(repository).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!");
    }
}