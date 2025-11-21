// use bytes::Bytes;
// use http::{
//     Request, Response, StatusCode,
//     header::{self, HeaderName},
// };
// use http_body_util::Full;
// use std::{any::Any, convert::Infallible};
// use tower::{Service, ServiceBuilder, ServiceExt, layer::util::Identity, service_fn};
// use tower_http::catch_panic::CatchPanicLayer;

// async fn handle(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//     panic!("something went wrong...")
// }

// fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response<Full<Bytes>> {
//     let details = if let Some(s) = err.downcast_ref::<String>() {
//         s.clone()
//     } else if let Some(s) = err.downcast_ref::<&str>() {
//         s.to_string()
//     } else {
//         "Unknown panic message".to_string()
//     };

//     let body = serde_json::json!({
//         "error": {
//             "kind": "panic",
//             "details": details,
//         }
//     });
//     let body = serde_json::to_string(&body).unwrap();

//     Response::builder()
//         .status(StatusCode::INTERNAL_SERVER_ERROR)
//         .header(header::CONTENT_TYPE, "application/json")
//         .body(Full::from(body))
//         .unwrap()
// }
