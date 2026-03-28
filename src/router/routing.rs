use std::sync::Arc;
use tauri::Manager;
use tauri::{AppHandle, Runtime};
use crate::router::parser::URLInfo;

pub type Request = tauri::http::Request<Vec<u8>>;
pub type Response = tauri::http::Response<Vec<u8>>;

//
// Public: Router
//
#[derive(Debug)]
pub struct Router<R: Runtime> {
    routes: Arc<Vec<(&'static str, RouteHandler<R>)>>,
}

type RouteHandler<R> = fn(&AppHandle<R>, Request, URLInfo) -> Response;

impl<R: Runtime> Router<R> {
    pub fn register(routes: Vec<(&'static str, RouteHandler<R>)>) -> Router<R> {
        Router {
            routes: Arc::new(routes),
        }
    }

    pub fn verify(&self, path: &str, req: Request, app: &AppHandle<R>, querys: Option<&str>) -> Option<Response> {
        for (route, handler) in self.routes.iter() {
            if let Some(parsed) = super::parser::parse_path(path, route, querys) {
                return Some(handler(app, req, parsed));
            }
        }
        None
    }
}

//
// Public: Routes Builder
//
#[derive(Debug)]
pub struct Routes<R: Runtime> {
    routes: Vec<(&'static str, RouteHandler<R>)>,
}

impl<R: Runtime> Routes<R> {
    pub fn new() -> Self {
        Routes { routes: Vec::new() }
    }

    pub fn add(
        mut self,
        route: &'static str,
        handler: RouteHandler<R>,
    ) -> Self {
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
