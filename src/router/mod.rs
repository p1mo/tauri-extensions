use std::sync::Arc;
use std::sync::OnceLock;

use tauri::Builder;
use tauri::Runtime;
use tauri::http::Response as TauriResponse;

use percent_encoding::percent_decode_str;

pub use tauri::http::StatusCode;

pub use tauri::http::header as HEADER;

mod parser;
mod routing;
mod routing_async;

pub use routing::Routes;
pub use routing::Router;
pub use routing::Request;
pub use routing::Response;

pub use parser::URLInfo;
pub use parser::URLParams;
pub use parser::URLQuerys;

#[derive(Debug)]
struct RouterState<R: Runtime>(Router<R>);

pub fn not_found(url : &str) -> Response {
    TauriResponse::builder()
        .header("Access-Control-Allow-Origin", "*")
        .status(StatusCode::NOT_FOUND)
        .header(HEADER::CONTENT_TYPE, "text/plain")
        .body(format!("CP request failed on: {}", url).as_bytes().to_vec())
        .unwrap() 
}

pub fn response(status : StatusCode, mime : &str, data : Vec<u8>) -> Response {
    TauriResponse::builder()
        .header("Access-Control-Allow-Origin", "*")
        .status(status)
        .header(HEADER::CONTENT_TYPE, mime)
        .body(data)
        .unwrap() 
}

pub fn normalize_path(path: &str) -> String {
    let mut normalized = path.trim_start_matches('/');
    normalized = normalized.trim();
    format!("/{}", normalized)
}
