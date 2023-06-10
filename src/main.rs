mod repositories;
mod handlers;

use axum::routing::{get, patch, post};
use axum::routing::Router;
use std::net::SocketAddr;
use std::env;
use std::sync::Arc;
use crate::handlers::{all_todo, create_todo, update_todo};
use crate::repositories::{TodoRepository, TodoRepositoryForMemory};


#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let log_level = env::var("RUST_LOG").unwrap_or("debug".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository.into());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    tracing::debug!("http://localhost:3000");
    axum::Server::bind(&addr).serve(app.into_make_service()).await
}

fn create_app<T: TodoRepository>(repository: Arc<T>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<T>).get(all_todo::<T>))
        .with_state(Arc::clone(&repository))
        .route(
            "/todos/:id",
            patch(update_todo::<T>),
        )
        .with_state(Arc::clone(&repository))
}

async fn root() -> &'static str {
    "Hello, World!"
}


#[cfg(test)]
mod test {
    use super::*;
    use axum::body::Body;
    use axum::http;
    use axum::http::{header, Method};
    use axum::response::Response;
    use http::{Request};
    use serde::Deserialize;

    use tower::ServiceExt;
    use crate::repositories::{CreateTodo, Todo};

    fn build_request_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    async fn res_to_todo<T: for<'a> Deserialize<'a>>(res: Response) -> T {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        println!("debug {}", body);
        let todo: T = serde_json::from_str(&body)
            .expect(&format!("cannot convert Todo instance. body: {}", body));
        todo
    }

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let repository = TodoRepositoryForMemory::new();
        let res = create_app(repository.into()).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!");
    }

    #[tokio::test]
    async fn should_update_todo() {
        let expected = Todo::new(1, "should_update_todo".to_string());

        let repository = TodoRepositoryForMemory::new();
        repository.create(CreateTodo::new("before_update_todo".to_string()));
        let req = build_request_with_json(
            "/todos/1",
            Method::PATCH,
            r#"{"text": "should_update_todo","completed": false}"#
                .to_string(),
        );
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        let todo = res_to_todo::<Todo>(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_get_all_todos() {
        let payload = CreateTodo::new("temp".to_string());
        let repository = TodoRepositoryForMemory::new();
        repository.create(payload);
        let req = build_request_with_json(
            "/todos",
            Method::GET,
            String::default(),
        );
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        let todo = res_to_todo::<Vec<Todo>>(res).await;
        assert_eq!(vec![Todo::new(1, "temp".to_string())], todo);
    }
}