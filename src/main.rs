use axum::{
    http::{Request, StatusCode},
    response::{Html, Response},
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use const_format::formatcp;
use dotenvy_macro::dotenv;
use std::{fs, net::SocketAddr, time::Duration};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;

const HTTPS_PORT: u16 = 8443;

const CERT_PATH: &'static str = dotenv!("CERT_PATH");
const PRIVATE_KEY_PATH: &'static str = dotenv!("PRIVATE_KEY_PATH");

const MAIN_PAGE_PATH: &'static str = env!("MAIN_PAGE_PATH");
const NOT_FOUND_PAGE_PATH: &'static str = env!("NOT_FOUND_PAGE_PATH");

macro_rules! HTML_from_path {
    ($const_path:path) => {
        Html(
            fs::read_to_string($const_path)
                .expect(formatcp!("{} is invalid", stringify!($const_path))),
        )
    };
}

#[tokio::main]
async fn main() {
    // TODO : Write events in a custom format with custom middleware and no tower with .layer(middleware::from_fn(...))
    let logger = TraceLayer::new_for_http()
        .on_request(|request: &Request<_>, _: &Span| {
            const LOG_PREFIX: &str = "Request";
            const LOG_PRECIX_LEN: usize = LOG_PREFIX.len();
            const LOG_INNER_LEN: usize = LOG_PRECIX_LEN + 4;
            println!(
                "{LOG_PREFIX} - method : {} | uri : {} | HTTP version: {:?}",
                request.method(),
                request.uri(),
                request.version()
            );
            println!("{:LOG_PRECIX_LEN$} - headers:", "");
            request.headers().iter().for_each(|header| {
                println!(
                    "{:LOG_INNER_LEN$} - {} : {}",
                    "",
                    header.0.as_str(),
                    header.1.to_str().unwrap(),
                )
            });
            //todo!("Print body");
        })
        .on_response(|response: &Response, latency: Duration, _: &Span| {
            println!(
                "Response - status : {} after {}ms",
                response.status(),
                latency.as_millis()
            );
        })
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, _: &Span| {
                println!("FAILURE! - {} after {}ms", error, latency.as_millis());
            },
        );

    // TODO : HSTS
    let app = Router::new()
        .route("/", get(HTML_from_path!(MAIN_PAGE_PATH)))
        .fallback((StatusCode::NOT_FOUND, HTML_from_path!(NOT_FOUND_PAGE_PATH)))
        .route_layer(logger);

    let tls_config = RustlsConfig::from_pem_file(CERT_PATH, PRIVATE_KEY_PATH)
        .await
        .expect(formatcp!(
            "Please export {} and {}",
            stringify!(CERT_PATH),
            stringify!(PRIVATE_KEY_PATH)
        ));

    let addr = SocketAddr::from(([0, 0, 0, 0], HTTPS_PORT));

    let start_message: String = format!("Started server at address {addr}");
    println!("{start_message}");
    println!("{:-<len$}", "", len = start_message.len());

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
