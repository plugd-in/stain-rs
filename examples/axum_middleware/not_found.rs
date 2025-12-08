use axum::response::Response;
use http::StatusCode;
use stain::stain;

use crate::{
    middleware::{store, Middleware},
    BoxedFuture,
};

#[derive(Default)]
struct NotFound;

impl Middleware for NotFound {
    fn response(&self, mut response: Response) -> BoxedFuture<Response> {
        *(response.status_mut()) = StatusCode::NOT_FOUND;

        Box::pin(async { response })
    }
}

stain! {
    store: store;
    item: NotFound;
    ordering: 0;
}
