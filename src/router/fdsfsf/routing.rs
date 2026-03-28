use tauri::Manager;
use tauri::{AppHandle, Runtime};
use crate::router::parser::URLParams;
use crate::router::parser::URLInfo;

pub type Request = tauri::http::Request<Vec<u8>>;
pub type Response = tauri::http::Response<Vec<u8>>;

//
// Public: Router
//
#[derive(Debug, Clone)]
pub struct Router<R: Runtime> {
    routes: HashMap<String, fn(&AppHandle<R>, Request, URLInfo) -> Response>,
}

impl<R: Runtime> Router<R> {
    pub fn register(routes: Vec<(String, fn(&AppHandle<R>, Request, URLInfo) -> Response)>) -> Router<R> {
        Router {
            routes: routes.into_iter().collect(), // Convert Vec into HashMap
        }
    }

    pub fn verify(&self, path: &str, req: Request, app: &AppHandle<R>, querys: Option<&str>) -> Option<Response> {
        if let Some(handler) = self.routes.get(path) {
            // Exact match
            if let Some(parsed) = super::parser::parse_path(path, path, querys) {
                return Some(handler(app, req, parsed));
            }
        } else {
            // Fallback for parameterized routes
            for (route, handler) in self.routes.iter() {
                if let Some(parsed) = super::parser::parse_path(path, route, querys) {
                    return Some(handler(app, req, parsed));
                }
            }
        }
        None
    }
}

//
// Public: Routes
//
#[derive(Debug, Clone)]
pub struct Routes<R: Runtime> {
    routes: Vec<(String, fn(&AppHandle<R>, Request, URLInfo) -> Response)>,
}

impl<R: Runtime> Routes<R> {
    pub fn new() -> Self {
        Routes { routes: Vec::new() }
    }

    pub fn add(mut self, route: &str, handler: fn(&AppHandle<R>, Request, URLInfo) -> Response) -> Self {
        self.routes.push((route.to_string(), handler));
        self
    }

    pub fn build(self) -> Vec<(String, fn(&AppHandle<R>, Request, URLInfo) -> Response)> {
        self.routes
    }
}
