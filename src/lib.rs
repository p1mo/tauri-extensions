use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::collections::HashMap;

use serde::{ser::Serializer, Serialize};
use percent_encoding::percent_decode_str;

use tauri::plugin::Builder as PluginBuilder;
use tauri::utils::config::WindowConfig;
use tauri::plugin::TauriPlugin;
use tauri::webview::WebviewWindowBuilder;
use tauri::{ App, AppHandle, Builder, Runtime, Manager, WebviewWindow };

use bincode::{Decode, Encode};

use std::path::PathBuf;
#[cfg(feature = "templates")]
use handlebars::Handlebars;
#[cfg(feature = "templates")]
use walkdir::WalkDir;

mod themes;
mod router;

mod utils;



pub use tauri::Emitter;


pub use crate::router::{Routes, Request, Response, URLInfo, URLParams, URLQuerys, StatusCode, response};

pub use crate::themes::ThemeList;




static CTM: CoreToolsManager = OnceLock::new();
static WINDOW_TIMEOUT: OnceLock<usize> = OnceLock::new();


const IS_MOBILE: bool = cfg!(target_os = "android") || cfg!(target_os = "ios");
const LABEL_PREFIX: &str = if cfg!(target_os = "android") || cfg!(target_os = "ios") { "mobile-" } else { "desktop-" };
const SHOW_WINDOW_JS : &str = include_str!("../.javascript/show_window.js");


const CORETOOLS_DIR: &str = "TauriCoreTools";



#[cfg(feature = "templates")]
#[derive(Clone, Debug)]
pub struct Templates(Arc<Handlebars<'static>>);


struct CoreToolsBase {
    #[cfg(feature = "templates")]
    templates: Option<Templates>,

    #[cfg(feature = "themes")]
    themes: Option<crate::themes::Themes>,

    configs: Vec<WindowConfig>,
    main: Option<WebviewWindow>,
    windows: HashMap<String, WebviewWindow>,
}

type CoreToolsManager = OnceLock<Arc<RwLock<CoreToolsBase>>>;






#[derive(Encode, Decode, PartialEq, Debug)]
struct WindowState {
    label: String,
    pos: Option<(i32, i32)>,
    size: Option<(u32, u32)>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct WindowStatesData(Vec<WindowState>);



type WindowStatesManagerState = Arc<Mutex<WindowStatesManager>>;



fn create_empty_states(full_path: &str, states: Vec<WindowState>) -> Result<()> {
    let data = WindowStatesData(states);
    let encoded: Vec<u8> = bincode::encode_to_vec(&data, bincode::config::standard()).unwrap();
    std::fs::write(&full_path, encoded)?;
    Ok(())
}



#[derive(Debug)]
struct WindowStatesManager {
    store_path: PathBuf,
    count: usize,
    states: WindowStatesData,
}

impl WindowStatesManager {

    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            store_path: data_dir,
            count: 0,
            states: WindowStatesData(vec![]),
        }
    }

    pub fn load(&mut self) -> Result<()> {
        let full_path = self.store_path.to_str().unwrap();
        let file = std::fs::read(full_path)?;
        let (decoded, count): (WindowStatesData, usize) = bincode::decode_from_slice(&file[..], bincode::config::standard()).unwrap();
        self.count = count;
        self.states = decoded;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let full_path = self.store_path.to_str().unwrap();
        let encoded: Vec<u8> = bincode::encode_to_vec(&self.states, bincode::config::standard()).unwrap();
        std::fs::write(&full_path, encoded)?;
        Ok(())
    }

    pub fn get(&self, label: &str) -> Result<&WindowState> {
        let states = self.states.0.iter().filter(|v|{ v.label == label }).collect::<Vec<&WindowState>>();
        if states.len() == 0 {
            return Err(Error::WindowStatesNotFound(label.to_string()));
        }
        if states.len() > 1 {
            return Err(Error::WindowStatesToMany(label.to_string()));
        }
        let state = states.get(0).unwrap();
        Ok(state)
    }

    pub fn check(&mut self, label: &str, new: WindowState) -> Result<()> {
        let mut states = self.states.0.iter().filter(|item| { item.label == label }).collect::<Vec<&WindowState>>();
        if states.len() == 0 {
            self.states.0.push(WindowState {
                label: label.to_string(),
                pos: new.pos,
                size: new.size
            });
        }
        Ok(())
    }

    pub fn set(&mut self, label: &str, new: WindowState) -> Result<()> {
        let mut states = self.states.0.iter_mut().filter(|item| { item.label == label }).collect::<Vec<&mut WindowState>>();
        if states.len() > 1 {
            return Err(Error::WindowStatesToMany(label.to_string()));
        }
        for state in states {
            if let Some(pos) = new.pos {
                state.pos = Some(pos);
            }
            if let Some(size) = new.size {
                state.size = Some(size);
            }
        }
        Ok(())
    }

}















fn init_core_tools_plugin<R: Runtime>() -> TauriPlugin<R> {

    let config = bincode::config::standard();

    PluginBuilder::<R>::new("tauriextension")
        .invoke_handler(tauri::generate_handler![])
        .setup(move |app, _api| {
            
            let data_dir = app.path().app_data_dir()?.join("WINDOW_STATES.bin");

            create_empty_states(data_dir.to_str().unwrap(), vec![]);

            app.manage(Arc::new(Mutex::new(WindowStatesManager::new(data_dir))));

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("actions", |ctx, req, res| {

            let path = req.uri().path();

            let app = ctx.app_handle();
            
            match path {
                "/show_window" => {
                    let label = ctx.webview_label();
                    let win = app.get_webview_window(label).unwrap();
                    win.show().unwrap();
                }
                _ => {}
            }

            res.respond(response(StatusCode::OK, "text/plain", "result".as_bytes().to_vec()));

        })
        /* TODO : Fix this
        .on_event(|app_handle, event| {

            let handle = app_handle.state::<WindowStatesManagerState>();

            match event {
                tauri::RunEvent::Ready => {

                    let mut states = handle.lock().unwrap();

                    states.load();

                }
                tauri::RunEvent::WindowEvent { label, event, .. } => {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            let mut states = handle.lock().unwrap();
                            states.save();
                            api.prevent_close();
                            std::process::exit(0);
                        }
                        tauri::WindowEvent::Moved(pos ) => {

                            let mut states = handle.lock().unwrap();
                            states.set(label, WindowState {
                                label: label.to_string(),
                                size: None,
                                pos : Some((pos.x, pos.y))
                            }).unwrap();

                        },
                        tauri::WindowEvent::Resized(size) => {

                            let mut states = handle.lock().unwrap();
                            states.set(label, WindowState {
                                label: label.to_string(),
                                size: Some((size.width, size.height)),
                                pos : None
                            }).unwrap();

                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        */
        .build()

}







pub trait CoreToolsBuilderExt<R: Runtime> {

    fn init_core_tools(self) -> Self;

    fn set_window_delay(self, timeout: usize) -> Self;

    fn use_router(self, uri_sheme: &str, routes: Routes<R>) -> Self;

    #[cfg(feature = "themes")]
    fn use_themes(self, default_theme: &str, internal_dir: &str) -> Self;

    #[cfg(feature = "templates")]
    fn use_templates(self, directory: &str, extension: &str) -> Self;

}

impl<R: Runtime> CoreToolsBuilderExt<R> for Builder<R> {

    fn init_core_tools(self) -> Self {

        CTM.get_or_init(|| {
            Arc::new(RwLock::new(CoreToolsBase {

                #[cfg(feature = "templates")]
                templates : None,

                #[cfg(feature = "themes")]
                themes: None,

                configs: Vec::new(),
                main: None,
                windows: HashMap::new(),
            }))
        });

        WINDOW_TIMEOUT.set(200).unwrap();

        self.plugin(init_core_tools_plugin())

    }

    fn set_window_delay(self, timeout: usize) -> Self {
        WINDOW_TIMEOUT.set(timeout).unwrap();
        self
    }

    /// ### Router with internal `uri scheme protocol`
    fn use_router(self, uri_sheme: &str, routes: Routes<R>) -> Self {
        let builder = routes.build();
        let router = std::sync::Arc::new(crate::router::Router::register(builder)); // Arc for thread safety

        self.register_asynchronous_uri_scheme_protocol(uri_sheme, move |ctx, req, res| {

            let app_handle = ctx.app_handle().clone();

            let cloned_router = router.clone();

            tauri::async_runtime::spawn(async move {
                let url = req.uri().clone().to_string();
                let raw_uri = percent_decode_str(&url).decode_utf8().unwrap();
                let uri = url::Url::options().parse(&raw_uri).unwrap();

                let path = crate::router::normalize_path(uri.path());

                if let Some(result) = &cloned_router.verify(&path, req, &app_handle, uri.query()) {
                    res.respond(result.to_owned())
                } else {
                    res.respond(crate::router::not_found(&path))
                }
            });
        })

    }

    #[cfg(feature = "themes")]
    /// Allow to use themes
    fn use_themes(self, default_theme: &str, internal_dir: &str) -> Self {

        if let Some(data) = CTM.get() {

            if let Ok(mut shared) = data.write() {

                shared.themes = Some(crate::themes::Themes::new(default_theme.to_string(), internal_dir.to_string()));

                shared.themes.as_mut().unwrap().reload().unwrap();

            }

        }

        self
    }

    #[cfg(feature = "templates")]
    fn use_templates(self, directory: &str, extension: &str) -> Self {

        let mut file_type = extension;
        if !file_type.contains(".") {
            file_type = file_type.trim_start_matches(".");
        }

        if let Some(data) = CTM.get() {

            if let Ok(mut shared) = data.write() {

                let mut reg = Handlebars::new();

                #[cfg(debug_assertions)]
                reg.set_dev_mode(true);

                register_templates(&mut reg, directory, file_type).unwrap();

                shared.templates = Some(Templates(Arc::new(reg)));

            }

        }

        self

    }

}







#[allow(dead_code)]
pub trait CoreToolsAppExt<R: Runtime> {

    #[cfg(not(all(target_os = "android", target_os = "ios")))]
    fn create_window(&self, label : &str) -> crate::Result<WebviewWindow<R>>;

    #[cfg(feature = "themes")]
    fn list_themes(&self) -> ThemeList;
    #[cfg(feature = "themes")]
    fn apply_theme_to_all(&self, theme_name: &str) -> Result<()>;

    #[cfg(feature = "templates")]
    fn template<T>(&self, path: &str, data: &T) -> String
    where
        T: Serialize;
}

impl<T, R: Runtime> CoreToolsAppExt<R> for T
where
    T: Manager<R>,
{
    #[cfg(not(all(target_os = "android", target_os = "ios")))]
    fn create_window(&self, label : &str) -> crate::Result<WebviewWindow<R>> {

        let handle = self.app_handle().state::<WindowStatesManagerState>();

        let win_label = label;

        let mut states = handle.lock().unwrap();

        let mut config = self
            .config()
            .app
            .windows
            .iter()
            .find(|c| c.label.contains(&(LABEL_PREFIX.to_owned() + label)))
            .ok_or(Error::WindowConfigNotFound(label.to_string()))?;

        let win = WebviewWindowBuilder::from_config(self.app_handle(), &config)?.build()?;

        /* IMPORTANT
        let w_pos = win.outer_position()?;
        let w_size = win.inner_size()?;

        states.check(&win_label, WindowState {
            label   : win_label.to_string(),
            pos     : Some((w_pos.x.into(), w_pos.y.into())),
            size    : Some((w_size.width.into(), w_size.height.into()))
        });

        let win_state = states.get(&win_label)?;

        if let Some((width, height)) = win_state.size {
            win.set_size(tauri::PhysicalSize::new(width, height));
        }

        if let Some((x, y)) = win_state.size {
            win.set_position(tauri::PhysicalPosition::new(x, y));
        }
        */

        #[cfg(feature = "themes")]
        win.init_theme()?;

        //if config.visible == false { win.show_window()?; }

        Ok(win)
    }

    #[cfg(feature = "themes")]
    /// List all available themes
    fn list_themes(&self) -> ThemeList {
        let themes = CTM.get().unwrap().read().unwrap().themes.clone().unwrap();
        themes.list()
    }

    #[cfg(feature = "themes")]
    /// Apply theme to all windows
    fn apply_theme_to_all(&self, theme_name: &str) -> Result<()> {
        for (_, win) in self.webview_windows() {
            win.apply_theme(theme_name)?
        }
        Ok(())
    }



    #[cfg(feature = "templates")]
    fn template<T2>(&self, path: &str, data: &T2) -> String
    where
        T2: Serialize,
    {
        let templates = &CTM.get().unwrap().read().unwrap().templates;
        templates.clone().unwrap().0.render(path, data).expect("templates:")
    }
}









pub trait EnhanceWebviewWindow<R: Runtime> {
    fn show_window(&self) -> crate::Result<()>;

    #[cfg(feature = "themes")]
    fn init_theme(&self) -> Result<()>;
    #[cfg(feature = "themes")]
    fn apply_theme(&self, theme_name: &str) -> Result<()>;
}

impl <R: Runtime> EnhanceWebviewWindow<R> for WebviewWindow<R> {

    fn show_window(&self) -> crate::Result<()> {
        let timeout = format!("{}", WINDOW_TIMEOUT.get().unwrap());
        self.eval(SHOW_WINDOW_JS.replace("___CUSTOM___DELAY___",  &timeout))?;
        Ok(())
    }

    #[cfg(feature = "themes")]
    /// Init theme
    fn init_theme(&self) -> Result<()> {
        let themes = &CTM.get().unwrap().read().unwrap().themes.clone().unwrap();
        let theme_str = themes.parse_to_string(&themes.default).unwrap();
        self.eval(&format!(r#"(function (){{
            //window.addEventListener('DOMContentLoaded', () => {{
                {}
            //}})
        }})();"#, theme_str))?;
        Ok(())
    }

    #[cfg(feature = "themes")]
    /// Apply theme to window
    fn apply_theme(&self, theme_name: &str) -> Result<()> {
        let themes = &CTM.get().unwrap().read().unwrap().themes.clone().unwrap();
        let theme_str = themes.parse_to_string(theme_name).unwrap();
        self.eval(&format!(r#"(function (){{
            {}
        }})();"#, theme_str))?;
        Ok(())
    }

}















pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  TauriError(#[from] tauri::Error),

  #[cfg(feature = "templates")]
  #[error(transparent)]
  HandlebarsTemplateError(#[from] handlebars::TemplateError),

  #[error("WindowConfig with label: {0} not found.")]
  WindowConfigNotFound(String),

  #[error("to many states for label: {0}")]
  WindowStatesToMany(String),

  #[error("state for label: {0} not found.")]
  WindowStatesNotFound(String),

  #[cfg(mobile)]
  #[error(transparent)]
  PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}





//
// Private functions
//

#[cfg(feature = "templates")]
fn register_templates(handlebars: &mut Handlebars, dir: &str, ext: &str) -> Result<()> {
    for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let fullpath = entry.path();
            if let Some(extension) = fullpath.extension() {
                if extension == ext {
                    let path = fullpath.to_str().unwrap();
                    let tmpl = path.trim_start_matches(dir).replace("\\", "/");
                    let name = tmpl.trim_start_matches("/");
                    handlebars.register_template_file(name, path)?;
                }
            }
        }
    }
    Ok(())
}

fn read_dir_recursive(dir: PathBuf, cb: &dyn Fn(&std::fs::DirEntry)) -> Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                read_dir_recursive(path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}
