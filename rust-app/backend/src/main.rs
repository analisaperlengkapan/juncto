mod api;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Define the router
    // Serving "frontend/pkg"
    // fallback_service handles everything not matched by specific routes.
    // We configure ServeDir to serve files if they exist, and fallback to index.html if not (SPA behavior).
    let serve_dir = ServeDir::new("frontend/pkg")
        .not_found_service(ServeFile::new("frontend/pkg/index.html"));

    let app = Router::new()
        .route("/api/rooms", post(api::create_room))
        .route("/health", get(api::health_check))
        .fallback_service(serve_dir);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
