use futures_util::future::BoxFuture;
use tower_http::catch_panic::CatchPanicLayer;

use crate::{
    config::Config,
    db::create_pool,
    routes::create_routes,
    services::{Services, sessions_service::SessionsService, user_service::UserService},
    state::AppState,
    utils::jwt::JwtManager,
};

use axum::{
    ServiceExt,
    body::Bytes,
    error_handling::HandleErrorLayer,
    extract::Request,
    http::{StatusCode, header},
    response::Response,
};
use std::{
    any::Any,
    task::{Context, Poll},
    time::Duration,
};

use http_body_util::Full;
use tower::{Layer, Service, ServiceBuilder, timeout::TimeoutLayer};

mod config;
mod db;
mod entities;
mod errors;
mod hell;
mod middleware;
mod models;
mod routes;
mod services;
mod state;
mod utils;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("fugue_backend=info") // reads RUST_LOG
        .with_file(false)
        .with_target(false) // optional: hide module path
        .with_thread_ids(false) // optional: show thread ids
        .init();
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    init_tracing();

    let db = create_pool(&config.database_url)
        .await
        .expect("Failed to init DB");

    let jwt = JwtManager::new(config.jwt_secret.clone(), config.jwt_expiration_hours);

    let services = Services {
        user: UserService { db: db.clone() },
        sessions: SessionsService { db: db.clone() },
    };

    let state = AppState {
        db,
        jwt,
        config: config.clone(),
        services,
    };

    // let app = create_routes(state.clone())
    //     .layer(HandleErrorLayer::new(handle_error))
    //     .with_state(state);

    // let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
    //     .await
    //     .unwrap();

    // println!("Running on http://localhost:{}", config.port);
    // axum::serve(listener, app).await.unwrap();

    fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response<Full<Bytes>> {
        println!("panic??");
        let details = if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic message".to_string()
        };

        let body = serde_json::json!({
            "error": {
                "kind": "panic",
                "details": details,
            }
        });
        let body = serde_json::to_string(&body).unwrap();

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::from(body))
            .unwrap()
    }

    // let huh = ServiceBuilder::new()
    //     // Use `handle_panic` to create the response.
    //     .layer(HandleErrorLayer::new(handle_panic));
    // // .layer(CatchPanicLayer::custom(handle_panic));

    let svc = ServiceBuilder::new()
        // Use `handle_panic` to create the response.
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(HandleErrorLayer::new(handle_panic));

    let app = create_routes(state.clone()).with_state(state);
    let x = svc.service(app);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    println!("Running on http://localhost:{}", config.port);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// use crate::errors::handle_error;
// use axum::body::Body;

// use axum::error_handling::HandleErrorLayer;
// use axum::http::{Error, Request, StatusCode};
// use axum::middleware::Next;
// use axum::{Router, response::IntoResponse, routing::get};
// use std::time::Duration;
// use tower::timeout::TimeoutLayer;
// use tower::{BoxError, ServiceBuilder};

// use tracing::info;

// pub mod errors;

// #[tokio::main]
// async fn main() {
//     // Init tracing for logs
//     tracing_subscriber::fmt().init();

//     // Minimal logging middleware
//     async fn logging_middleware(req: Request<Body>, next: Next) -> impl IntoResponse {
//         let path = req.uri().path().to_string();
//         info!("Incoming request to: {}", path);

//         let response = next.run(req).await;

//         info!("Response for {} sent", path);
//         response
//     }

//     let x = ServiceBuilder::new()
//         .layer(TimeoutLayer::new(Duration::from_secs(5)))
//         .layer(HandleErrorLayer::<_, BoxError>::new(
//             |_: BoxError| async move { StatusCode::BAD_REQUEST },
//         ));

//     // Minimal router with one route and one middleware
//     // let app = Router::new()
//     //     .route("/", get(|| async { "Hello, world!" }))
//     //     // .layer(axum::middleware::from_fn(logging_middleware))
//     //     .layer(x);

//     let app = Router::new()
//         .route("/", get(|| async { "Hello, world!" }))
//         .layer(MyLayer {
//             state: AppState {}.clone(),
//         });

//     let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 3000))
//         .await
//         .unwrap();

//     axum::serve(listener, app.into_make_service())
//         .await
//         .unwrap();
// }

#[derive(Clone)]
struct MyLayer {
    state: AppState,
}

impl<S> Layer<S> for MyLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MyMiddleware { inner }
    }
}

#[derive(Clone)]
struct MyMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for MyMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}

//
