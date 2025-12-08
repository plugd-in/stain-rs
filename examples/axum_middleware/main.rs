use anyhow::Result;
use axum::{routing::get, Router};
use reqwest::StatusCode;
use stain::Store as _;
use std::{future::Future, pin::Pin};
use tokio::sync::oneshot::{channel, Receiver};

use crate::middleware::store::Store;

mod logger;
mod middleware;
mod not_found;

// Utility type to prevent complex types.
type BoxedFuture<Output> = Pin<Box<dyn Future<Output = Output> + Send + 'static>>;

// Runs the Axum server...
async fn start_server(shutdown: Receiver<()>) {
    // Build our Axum router...
    let app = Router::new().route(
        "/",
        get(|| async { "Hello, World!" }).layer(Store::collect()), // Add our Store as a layer...
    );

    // Bind to localhost 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // serve our app, shutting down when we receive an indication
    // from the client side that it is done...
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown.await;
        })
        .await
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    // used to tell the background axum task to shutdown
    let (tx, rx) = channel::<()>();

    // start listening in the background
    let server_task = tokio::spawn(async {
        start_server(rx).await;
    });

    // make our request
    let response = reqwest::get("http://127.0.0.1:3000/").await?;
    let response_status = response.status();
    let response_text = response.text().await?;

    // We expect "Hello, World!" based on our `get(...)` method router.
    assert_eq!(response_text, "Hello, World!");
    // Initially, our `get(...)` method router should respond with a
    // 200 status code. However, our NotFound middleware rewrites all
    // responses to use a 404 status code.
    assert_eq!(response_status, StatusCode::NOT_FOUND);

    // Indicate to the server to shut down gracefully.
    tx.send(()).expect("Open Channel");
    // Wait for the server to shut down gracefully.
    server_task.await?;

    Ok(())
}
