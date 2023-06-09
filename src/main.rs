use axum::routing::get;
use axum::routing::Router;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let app = Router::new().route("/", get(root));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr).serve(app.into_make_service()).await
}

async fn root() -> &'static str {
    "Hello, World!"
}