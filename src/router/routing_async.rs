use std::pin::Pin;
use std::sync::Arc;

use futures::future::BoxFuture;
use tauri::Manager;
use tauri::{AppHandle, Runtime};
use crate::router::parser::URLInfo;

pub type Request = tauri::http::Request<Vec<u8>>;
pub type Response = tauri::http::Response<Vec<u8>>;

//
// Public: Router
//
pub struct Router<R: Runtime> {
    routes: Arc<Vec<(&'static str, RouteHandler<R>)>>,
}

// Enum to represent either sync or async handlers
pub enum RouteHandler<R: Runtime> {
    Sync(fn(&AppHandle<R>, Request, URLInfo) -> Response),
    Async(Box<dyn Fn(&AppHandle<R>, Request, URLInfo) -> BoxFuture<'static, Response> + Send + Sync>),
}

impl<R: Runtime> RouteHandler<R> {
    // Helper method to call the appropriate handler
    async fn call(&self, app: &AppHandle<R>, req: Request, url_info: URLInfo) -> Response {
        match self {
            RouteHandler::Sync(handler) => handler(app, req, url_info),
            RouteHandler::Async(handler) => handler(app, req, url_info).await,
        }
    }
}

impl<R: Runtime> Router<R> {
    pub fn register(routes: Vec<(&'static str, RouteHandler<R>)>) -> Router<R> {
        Router {
            routes: Arc::new(routes),
        }
    }

    pub async fn verify(
        &self,
        path: &str,
        req: Request,
        app: &AppHandle<R>,
        querys: Option<&str>,
    ) -> Option<Response> {
        for (route, handler) in self.routes.iter() {
            if let Some(parsed) = super::parser::parse_path(path, route, querys) {
                return Some(handler.call(app, req, parsed).await);
            }
        }
        None
    }
}

//
// Public: Routes
//
pub struct Routes<R: Runtime> {
    routes: Vec<(&'static str, RouteHandler<R>)>,
}

impl<R: Runtime> Routes<R> {
    pub fn new() -> Self {
        Routes { routes: Vec::new() }
    }

    pub fn add_sync(
        mut self,
        route: &'static str,
        handler: fn(&AppHandle<R>, tauri::http::Request<Vec<u8>>, URLInfo) -> tauri::http::Response<Vec<u8>>,
    ) -> Self {
        self.routes.push((route, RouteHandler::Sync(handler)));
        self
    }

    pub fn add_async<F, Fut>(mut self, route: &'static str, handler: F) -> Self 
    where
        F: for<'a> Fn(&'a AppHandle<R>, tauri::http::Request<Vec<u8>>, URLInfo) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = tauri::http::Response<Vec<u8>>> + Send + 'static,
    {
        let boxed_handler = Box::new(move |app: &AppHandle<R>, req, url_info| {
            let fut = handler(app, req, url_info);
            Box::pin(fut) as Pin<Box<dyn futures::Future<Output = tauri::http::Response<Vec<u8>>> + Send>>
        });
        self.routes.push((route, RouteHandler::Async(boxed_handler)));
        self
    }

    pub fn add(mut self, route: &'static str, handler: RouteHandler<R>) -> Self {
        self.routes.push((route, handler));
        self
    }

    pub fn merge(mut self, routes: Routes<R>) -> Self {
        self.routes.extend(routes.routes);
        self
    }

    pub fn build(self) -> Vec<(&'static str, RouteHandler<R>)> {
        self.routes
    }
}