use axum::{response::IntoResponse, http::StatusCode};

pub use crate::handlers::room::create_room;
pub use crate::handlers::ws::chat_handler;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
