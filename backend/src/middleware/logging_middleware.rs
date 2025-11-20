use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{error, info};

pub async fn logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // You can also create a span if you want structured logs
    info!("➡️  Incoming request: {} {}", method, path);

    // let response = next.run(req).await;
    // info!("Response sent: {}", path);

    // Run the handler
    let response = next.run(req).await;

    let status = response.status();

    if status.is_server_error() {
        error!("❌ {} {} -> {}", method, path, status);
    } else {
        info!("{} {} -> {}", method, path, status);
    }

    response
}
