// Our custom layer
#[derive(Clone)]
struct MyLayer {
    state: AppState,
}

// impl<S> Layer<S> for MyLayer {
//     type Service = MyService<S>;

//     fn layer(&self, inner: S) -> Self::Service {
//         MyService {
//             inner,
//             state: self.state.clone(),
//         }
//     }
// }

impl<S> Layer<S> for MyLayer {
    type Service = MyService<CatchPanic<S, DefaultResponseForPanic>>;

    fn layer(&self, inner: S) -> Self::Service {
        let svc = CatchPanicLayer::new().layer(inner);

        MyService {
            inner: svc,
            state: self.state.clone(),
        }
    }
}

// Our service wrapper
#[derive(Clone)]
struct MyService<S> {
    inner: S,
    state: AppState,
}

// Implement Service for the wrapper
impl<S> Service<Request> for MyService<S>
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

    // fn call(&mut self, req: Request<B>) -> Self::Future {
    //     let mut inner = self.inner.clone();
    //     let _state = self.state.clone();

    //     Box::pin(async move {
    //         let res = inner.call(req).await;

    //         match res {
    //             Ok(response) => Ok(response),

    //             Err(err) => {
    //                 println!("Caught error in middleware: {}", err.into());

    //                 use axum::Json;
    //                 use axum::body::boxed;
    //                 use serde_json::json;

    //                 // Make axum error JSON
    //                 let body = Json(json!({
    //                     "error": "Internal server error"
    //                 }))
    //                 .into_response()
    //                 .map(boxed); // ← THIS MAKES TYPES MATCH

    //                 Ok(body)
    //             }
    //         }
    //     })
    // }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
