use axum::{extract::Request, response::Response};
use stain::stain;

use crate::{middleware::{store, Middleware}, BoxedFuture};

#[derive(Default)]
struct Logger;

impl Middleware for Logger {
    fn request(&self, request: Request) -> BoxedFuture<Request> {
        let method = request.method();
        let uri = request.uri();

        println!("Request ({:?}): {:?}", method, uri);

        Box::pin(async { request })
    }

    fn response(&self, response: Response) -> BoxedFuture<Response> {
        let status = response.status();
        println!("Response Status: {:?}", status);

        Box::pin(async { response })
    }
}

stain! {
    store: store;
    item: Logger;
    ordering: 1;
}
