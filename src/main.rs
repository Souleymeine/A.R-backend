use dotenv_codegen::dotenv;
use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::{convert::Infallible, error::Error, fs, net::SocketAddr};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

mod logger;

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
            let svc = ServiceBuilder::new()
                .layer_fn(logger::Logger::new)
                .service(svc);
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("server error: {}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let main_page_bytes = Bytes::from(
                fs::read_to_string(MAIN_PAGE_PATH)
                    .expect("Put some existing path in .env fro MAIN_PAGE!"),
            );
            let body = Full::new(main_page_bytes);
            Ok(Response::new(body))
        }
        _ => {
            let not_found_page_bytes = Bytes::from(
                fs::read_to_string(NOTFOUND_PAGE_PATH)
                    .expect("Put some existing path in .env FOR NOTFOUND_PAGE!"),
            );
            let body = Full::new(not_found_page_bytes);
            let mut not_found = Response::new(body);
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
