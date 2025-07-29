#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware::{self, Next},
        response::Response,
        Router,
    };
    use tower::ServiceExt;

    async fn dummy_handler() -> &'static str {
        "OK"
    }

    async fn test_middleware(req: Request<Body>, next: Next) -> Response {
        let mut response = next.run(req).await;
        response
            .headers_mut()
            .insert("X-Test-Header", "test-value".parse().unwrap());
        response
    }

    #[tokio::test]
    async fn test_middleware_chain_execution() {
        let app = Router::new()
            .route("/", axum::routing::get(dummy_handler))
            .layer(middleware::from_fn(test_middleware));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("X-Test-Header").unwrap(), "test-value");
    }

    #[tokio::test]
    async fn test_middleware_ordering() {
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let call_order = Arc::new(Mutex::new(Vec::<String>::new()));
        let order1 = call_order.clone();
        let order2 = call_order.clone();

        async fn middleware1(req: Request<Body>, next: Next) -> Response {
            let order = req
                .extensions()
                .get::<Arc<Mutex<Vec<String>>>>()
                .unwrap()
                .clone();
            order.lock().await.push("middleware1_before".to_string());
            let response = next.run(req).await;
            order.lock().await.push("middleware1_after".to_string());
            response
        }

        async fn middleware2(req: Request<Body>, next: Next) -> Response {
            let order = req
                .extensions()
                .get::<Arc<Mutex<Vec<String>>>>()
                .unwrap()
                .clone();
            order.lock().await.push("middleware2_before".to_string());
            let response = next.run(req).await;
            order.lock().await.push("middleware2_after".to_string());
            response
        }

        async fn handler(req: Request<Body>) -> &'static str {
            let order = req
                .extensions()
                .get::<Arc<Mutex<Vec<String>>>>()
                .unwrap()
                .clone();
            order.lock().await.push("handler".to_string());
            "OK"
        }

        let app = Router::new()
            .route("/", axum::routing::get(handler))
            .layer(middleware::from_fn(move |mut req: Request<Body>, next: Next| {
                req.extensions_mut().insert(order1.clone());
                middleware1(req, next)
            }))
            .layer(middleware::from_fn(move |mut req: Request<Body>, next: Next| {
                req.extensions_mut().insert(order2.clone());
                middleware2(req, next)
            }));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let order = call_order.lock().await;
        assert_eq!(
            *order,
            vec![
                "middleware2_before",
                "middleware1_before",
                "handler",
                "middleware1_after",
                "middleware2_after"
            ]
        );
    }

    #[tokio::test]
    async fn test_response_modification_in_middleware() {
        async fn add_header_middleware(req: Request<Body>, next: Next) -> Response {
            let mut response = next.run(req).await;
            response
                .headers_mut()
                .insert("X-Custom-Header", "custom-value".parse().unwrap());
            response
        }

        let app = Router::new()
            .route("/", axum::routing::get(dummy_handler))
            .layer(middleware::from_fn(add_header_middleware));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("X-Custom-Header").unwrap(), "custom-value");
    }
}
