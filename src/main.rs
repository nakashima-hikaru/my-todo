use axum::routing::{get, post};
use axum::routing::Router;
use std::net::SocketAddr;
use std::env;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let log_level = env::var("RUST_LOG").unwrap_or("debug".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let app = create_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    tracing::debug!("http://localhost:3000");
    axum::Server::bind(&addr).serve(app.into_make_service()).await
}

fn create_app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    Json(payload): Json<CreateUser>
) -> impl IntoResponse {
    let user = User {
        id: 1337,
        username: payload.username,
    };
    (StatusCode::CREATED, Json(user))
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::body::Body;
    use axum::http;
    use http::{Request, header, Method};

    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/")
            .body(Body::empty()).unwrap();
        let res = create_app().oneshot(req).await.unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!")
    }

    #[tokio::test]
    async fn should_return_user_data(){
        let req = Request::builder().uri("/users").method(Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(r#"{"username": "ゲストユーザー"}"#)).unwrap();
        let res = create_app().oneshot(req).await.unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let user: User = serde_json::from_str(&body).unwrap();
        assert_eq!(user, User{id: 1337, username: "ゲストユーザー".to_string()});
    }
}