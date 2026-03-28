use tauri::Manager;
use tauri::{ AppHandle, Runtime };

use crate::Request;
use crate::Response;
use crate::parser::URLParams;
use crate::parser::URLInfo;





//
// Public: Router
//
pub struct Router<'a, R: Runtime> {
    routes: Vec<(&'a str, fn(&AppHandle<R>, Request, URLInfo) -> Response)>,
}

impl<'a, R: Runtime> Router<'a, R> {
    pub fn register(routes: Vec<(&'a str, fn(&AppHandle<R>, Request, URLInfo) -> Response)>) -> Router<'a, R> {
        Router { routes }
    }

    pub fn verify(&self, path: &str, req: Request, app: &AppHandle<R>, querys : Option<&str>) -> Option<Response> {
        for (route, handler) in &self.routes {
            if let Some(parsed) = super::parser::parse_path(path, route, querys) {
                return Some(handler(app, req, parsed));
            }
        }
        None
    }
}


//
// Public: Routes
//
pub struct Routes<'a, R: Runtime> {
    routes: Vec<(&'a str, fn(&AppHandle<R>, Request, URLInfo) -> Response)>,
}

impl<'a, R: Runtime> Routes<'a, R> {

    pub fn new() -> Self {
        Routes { routes: Vec::new() }
    }

    pub fn add(mut self, route: &'a str, handler: fn(&AppHandle<R>, Request, URLInfo) -> Response) -> Self {
        self.routes.push((route, handler));
        self
    }

    pub fn build(self) -> Vec<(&'a str, fn(&AppHandle<R>, Request, URLInfo) -> Response)> {
        self.routes
    }
    
}