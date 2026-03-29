use dotenv_codegen::dotenv;
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use std::{convert::Infallible, error::Error, fs, net::SocketAddr};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

mod logger;
use logger::Logger;

const MAIN_PAGE_PATH: &'static str = dotenv!("MAIN_PAGE_REL_PATH");
const NOTFOUND_PAGE_PATH: &'static str = dotenv!("NOTFOUND_PAGE_REL_PATH");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            let svc = hyper::service::service_fn(handle_request);
            let svc = ServiceBuilder::new().layer_fn(Logger::new).service(svc);
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("server error: {}", err);
            }
        });
    }
}

async fn handle_request(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full_html(MAIN_PAGE_PATH))),
        _ => {
            let mut not_found = Response::new(full_html(NOTFOUND_PAGE_PATH));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn full_html(path: &'static str) -> Full<Bytes> {
    let page = fs::read_to_string(path).expect(&format!("{path} does not exist!"));
    let page = Bytes::from(page);
    Full::new(page)
}
