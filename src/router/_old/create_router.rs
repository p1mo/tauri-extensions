use std::convert::Infallible;
use std::sync::Arc;
use tauri::http::StatusCode;
use tokio::sync::RwLock;
use futures::future::BoxFuture;
use tokio::sync::RwLockWriteGuard;



use crate::Request;
use crate::Response;
use crate::Method;
use crate::response;

type SyncHandler = Box<dyn Fn(Request) -> Response + Send + Sync>;
type AsyncHandler = Box<dyn Fn(Request) -> BoxFuture<'static, Response> + Send + Sync>;

enum Handler {
    Sync(SyncHandler),
    Async(AsyncHandler),
}

type Router = Arc<RwLock<Vec<(Method, String, Handler)>>>;

type RouterWriter<'a> = RwLockWriteGuard<'a, Vec<(Method, String, Handler)>>;


async fn main() {
    let router: Router = Arc::new(RwLock::new(Vec::new()));

    // Adding routes
    {
        let mut r: RouterWriter = router.write().await;

        add_sync_route(&mut r, Method::GET, "/", |req| {

            response(StatusCode::OK, "text/plain", "Hello, World!".as_bytes().to_vec())
            
        });

        add_sync_route(&mut r, Method::GET, "/sync", |req| {
            
            response(StatusCode::OK, "text/plain", "This is a sync handler!".as_bytes().to_vec())

        });

        add_async_route(&mut r, Method::GET, "/async", |req| {
            Box::pin(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                response(StatusCode::OK, "text/plain", "This is an async handler!".as_bytes().to_vec())

            })
        });
    }

}

async fn handle_request(req: Request, router: Router) -> Result<Response, Infallible> {
    let router = router.read().await;

    for (method, path, handler) in router.iter() {
        if method == &req.method() && path == req.uri().path() {
            return match handler {
                Handler::Sync(f) => Ok(f(req)),
                Handler::Async(f) => Ok(f(req).await),
            };
        }
    }

    Ok(response(StatusCode::NOT_FOUND, "text/plain", "Not Found".as_bytes().to_vec()))
}

fn add_sync_route<F>(router: &mut RouterWriter, method: Method, path: &str, handler: F)
where
    F: Fn(Request) -> Response + Send + Sync + 'static,
{
    router.push((method, path.to_string(), Handler::Sync(Box::new(handler))));
}

fn add_async_route<F>(router: &mut RouterWriter, method: Method, path: &str, handler: F)
where
    F: Fn(Request) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    router.push((method, path.to_string(), Handler::Async(Box::new(handler))));
}
