use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::Router;
use axum::routing::{get, patch, post};
use dotenv::dotenv;
use sqlx::PgPool;

use crate::handlers::todos::{all_todo, create_todo, delete_todo, find_todo, update_todo};
use crate::repositories::postgres::PostgresRepository;
use crate::repositories::todos::TodoRepository;

mod handlers;
mod repositories;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let log_level = env::var("RUST_LOG").unwrap_or("debug".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    let database_url = &env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    tracing::debug!("start connecting to the database...");
    let pool = PgPool::connect(database_url).await.unwrap_or_else(|_| {
        panic!(
            "failed to connect to the database whose url is: [{}]",
            database_url
        )
    });
    let repository = PostgresRepository::new(pool);
    let app = create_app(repository.into());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    tracing::debug!("http://localhost:3000");
    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?)
}

fn create_app<T: TodoRepository>(repository: Arc<T>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<T>).get(all_todo::<T>))
        .with_state(Arc::clone(&repository))
        .route(
            "/todos/:id",
            patch(update_todo::<T>)
                .get(find_todo::<T>)
                .delete(delete_todo::<T>),
        )
        .with_state(Arc::clone(&repository))
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http;
    use axum::http::{header, Method, StatusCode};
    use axum::response::Response;
    use http::Request;
    use hyper::body::to_bytes;
    use serde::Deserialize;
    use serde_json::json;
    use tower::ServiceExt;

    use crate::repositories::hash_map::test_utils::HashMapRepository;
    use crate::repositories::todos::{CreateTodo, Todo};

    use super::*;

    fn build_request_with_json(
        path: &str,
        method: Method,
        json_body: String,
    ) -> http::Result<Request<Body>> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
    }

    async fn response_to_result<T: for<'a> Deserialize<'a>>(res: Response) -> T {
        let bytes = to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let todo = serde_json::from_str(&body).unwrap();
        todo
    }

    #[tokio::test]
    async fn hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let repository = HashMapRepository::new();
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::OK, res.status());

        let bytes = to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!");
    }

    #[tokio::test]
    async fn create_todo() -> http::Result<()> {
        let repository = HashMapRepository::new();
        let req = build_request_with_json(
            "/todos",
            Method::POST,
            r#"{"text": "todo","completed": false}"#.to_string(),
        )?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::CREATED, res.status());
        let todo = response_to_result::<Todo>(res).await;
        let expected = Todo::new(1, "todo".to_string());
        assert_eq!(expected, todo);
        Ok(())
    }

    #[tokio::test]
    async fn post_validation_empty() -> http::Result<()> {
        let repository = HashMapRepository::new();
        let req = build_request_with_json(
            "/todos",
            Method::POST,
            r#"{"text": "","completed": false}"#.to_string(),
        )?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, res.status());
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!("Validation error: [text: text must not be empty]", body);
        Ok(())
    }

    #[tokio::test]
    async fn post_validation_too_long_text() -> http::Result<()> {
        let repository = HashMapRepository::new();
        let text = "a".repeat(101);
        let body = json!({
            "text": text,
            "completed": false
        })
        .to_string();
        let req = build_request_with_json("/todos", Method::POST, body)?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, res.status());
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(
            "Validation error: [text: text length exceeds the limit]",
            body
        );
        Ok(())
    }

    #[tokio::test]
    async fn update_todo() -> http::Result<()> {
        let expected = Todo::new(1, "should_update_todo".to_string());

        let repository = HashMapRepository::new();
        repository
            .create(CreateTodo::new("before_update_todo".to_string()))
            .await
            .expect("failed to create todo");
        let req = build_request_with_json(
            "/todos/1",
            Method::PATCH,
            r#"{"text": "should_update_todo","completed": false}"#.to_string(),
        )?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::CREATED, res.status());
        let todo = response_to_result::<Todo>(res).await;
        assert_eq!(expected, todo);
        Ok(())
    }

    #[tokio::test]
    async fn get_all_todos() -> http::Result<()> {
        let payload = CreateTodo::new("temp".to_string());
        let repository = HashMapRepository::new();
        repository
            .create(payload)
            .await
            .expect("failed to create todo");
        let req = build_request_with_json("/todos", Method::GET, String::default())?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        let todo = response_to_result::<Vec<Todo>>(res).await;
        assert_eq!(vec![Todo::new(1, "temp".to_string())], todo);
        Ok(())
    }

    #[tokio::test]
    async fn find_todos() -> http::Result<()> {
        let payload = CreateTodo::new("temp".to_string());
        let repository = HashMapRepository::new();
        repository
            .create(payload)
            .await
            .expect("failed to create todo");
        let req = build_request_with_json("/todos/1", Method::GET, String::default())?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::OK, res.status());
        let todo = response_to_result::<Todo>(res).await;
        assert_eq!(Todo::new(1, "temp".to_string()), todo);
        Ok(())
    }

    #[tokio::test]
    async fn not_found_todos() -> http::Result<()> {
        let payload = CreateTodo::new("temp".to_string());
        let repository = HashMapRepository::new();
        repository
            .create(payload)
            .await
            .expect("failed to create todo");
        let req = build_request_with_json("/todos/2", Method::GET, String::default())?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, res.status());
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn delete_todo() -> http::Result<()> {
        let payload = CreateTodo::new("temp".to_string());
        let repository = HashMapRepository::new();
        repository
            .create(payload)
            .await
            .expect("failed to create todo");
        let req = build_request_with_json("/todos/1", Method::DELETE, String::default())?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
        Ok(())
    }

    #[tokio::test]
    async fn not_deleted_todo() -> http::Result<()> {
        let payload = CreateTodo::new("temp".to_string());
        let repository = HashMapRepository::new();
        repository
            .create(payload)
            .await
            .expect("failed to create todo");
        let req = build_request_with_json("/todos/2", Method::DELETE, String::default())?;
        let res = create_app(repository.into()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, res.status());
        Ok(())
    }
}
