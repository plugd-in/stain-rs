use std::{convert::Infallible, task::Poll};

use axum::{extract::Request, response::Response};
use stain::{create_stain, Store as _};
use tower::{Layer, Service};

use crate::BoxedFuture;

use store::Store;

/// Our middleware trait is simple. We just let implementors alter
/// requests and responses.
pub(crate) trait Middleware {
    fn request(&self, request: Request) -> BoxedFuture<Request> {
        Box::pin(async { request })
    }

    fn response(&self, response: Response) -> BoxedFuture<Response> {
        Box::pin(async { response })
    }
}

create_stain! {
    trait Middleware;
    store: pub(crate) mod store;
}

// Allow our Store to act as a layer on top of other
// services.
impl<S> Layer<S> for Store {
    type Service = MiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MiddlewareService {
            store: self.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
/// Our layered middleware service.
pub(crate) struct MiddlewareService<S> {
    store: Store,
    inner: S,
}

// We allow our layered service to act like a service.
impl<S> Service<Request> for MiddlewareService<S>
where
    // Error = Infallible for simplicity... Real use cases should probably be
    // more general/forgiving.
    S: Service<Request, Response = Response, Error = Infallible> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = BoxedFuture<Result<Self::Response, Self::Error>>;

    /// Always ready...
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let inner = self.inner.clone();
        let store = self.store.clone();

        Box::pin(async move {
            let mut inner = inner;
            let store = store;

            // Here, we get the initial request and then we loop over our implementations,
            // allowing them to alter the request.
            let mut request = req;
            for middleware in store.iter() {
                request = middleware.request(request).await;
            }

            // Same as above, but for the response.
            let mut response = inner.call(request).await?;
            for middleware in store.iter() {
                response = middleware.response(response).await;
            }

            Ok(response)
        })
    }
}
