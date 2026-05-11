use axum::{http::StatusCode, response::IntoResponse};

pub use crate::handlers::feedback::submit_feedback;
pub use crate::handlers::room::create_room;
pub use crate::handlers::ws::chat_handler;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = axum::Router::new().route("/health", get(health_check));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
