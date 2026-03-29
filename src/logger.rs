use hyper::{body::Incoming, service::Service, Request};

#[derive(Debug, Clone)]
pub struct Logger<S> {
    inner: S,
}
impl<S> Logger<S> {
    pub fn new(inner: S) -> Self {
        Logger { inner }
    }
}

impl<S: Service<Request<Incoming>>> Service<Request<Incoming>> for Logger<S> {
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    fn call(&self, req: Request<Incoming>) -> Self::Future {
        println!("processing request: {} {}", req.method(), req.uri().path());
        self.inner.call(req)
    }
}
