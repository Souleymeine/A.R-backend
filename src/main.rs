use axum::{http::StatusCode, response::Html, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use const_format::formatcp;
use dotenvy_macro::dotenv;
use std::{fs, net::SocketAddr};

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
    // TODO : HSTS
    let app = Router::new()
        .route("/", get(HTML_from_path!(MAIN_PAGE_PATH)))
        .fallback((StatusCode::NOT_FOUND, HTML_from_path!(NOT_FOUND_PAGE_PATH)));

    let tls_config = RustlsConfig::from_pem_file(CERT_PATH, PRIVATE_KEY_PATH)
        .await
        .expect(formatcp!(
            "Please export {} and {}",
            stringify!(CERT_PATH),
            stringify!(PRIVATE_KEY_PATH)
        ));

    let addr = SocketAddr::from(([0, 0, 0, 0], HTTPS_PORT));

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
