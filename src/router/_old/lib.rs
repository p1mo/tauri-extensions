

mod create_router;
mod parser;
mod router;

use std::sync::OnceLock;

use tauri::AppHandle;
use tauri::{
    Manager,
    Runtime,
    http::header,
};

use tauri::plugin::{
    Builder,
    TauriPlugin
};

use percent_encoding::percent_decode_str;

#[allow(dead_code)]
pub use tauri::http::Response as TauriResponse;

#[allow(dead_code)]
pub use tauri::http::StatusCode;

pub type Request  = tauri::http::Request<Vec<u8>>;
pub type Response = tauri::http::Response<Vec<u8>>;

pub type Method   = tauri::http::Method;

pub use router::Routes;
pub use parser::URLInfo;



static ALLOW_ORIGIN: OnceLock<String> = OnceLock::new();

struct RouterState<'a, R: Runtime>(router::Router<'a, R>);

struct SendResp(std::sync::mpsc::Sender<Response>);

pub fn init<R: Runtime>(route_set : Routes<'static, R>) -> TauriPlugin<R> {

    ALLOW_ORIGIN.set("*".to_string()).unwrap();

    let routes = route_set.build();
    
    Builder::<R>::new("router")
        .setup(|app, _api| {

            app.manage(RouterState(router::Router::register(routes)));

            println!("Router initialized");

            Ok(())

        })
        .register_asynchronous_uri_scheme_protocol("router", move |app, request, responder| {

            let url = request.uri().clone().to_string();

            let handler = app.clone();

            std::thread::spawn(move || {

                let router = handler.state::<RouterState<R>>();

                let raw_uri = percent_decode_str(&url).decode_utf8().unwrap();

                let uri = url::Url::options().parse(&raw_uri).unwrap();
                
                if let Some(res) = router.0.verify(uri.path(), request, &handler.clone(), uri.query()) {

                    responder.respond(res)
            
                } else {
            
                    responder.respond(not_found(uri.path()))
            
                }

            });

        })
        .build()

}



fn not_found(url : &str) -> Response {
    TauriResponse::builder()
        .header("Access-Control-Allow-Origin", ALLOW_ORIGIN.get().unwrap())
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(format!("CP request failed on: {}", url).as_bytes().to_vec())
        .unwrap() 
}



pub fn response(status : StatusCode, mime : &str, data : Vec<u8>) -> Response {
    TauriResponse::builder()
        .header("Access-Control-Allow-Origin", ALLOW_ORIGIN.get().unwrap())
        .status(status)
        .header(header::CONTENT_TYPE, mime)
        .body(data)
        .unwrap() 
}