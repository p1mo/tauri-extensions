use tauri::AppHandle;
use tauri::{
    Manager,
    Runtime,
};

use tauri::plugin::{
    Builder as PluginBuilder,
    TauriPlugin
};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use handlebars::Handlebars;
use walkdir::WalkDir;

// struct RouterState<'a, R: Runtime>(router::Router<'a, R>);

#[derive(Clone, Debug)]
pub struct Templates(Arc<Handlebars<'static>>);

pub type Templ = Arc<Handlebars<'static>>;


pub fn init<R: Runtime>(directory: &str, extension: &str) -> TauriPlugin<R> {

    let templates = directory.to_string();

    let mut file_type = extension.to_string();
    if !file_type.contains(".") {
        file_type = format!(".{}", file_type);
    }

    PluginBuilder::<R>::new("templates")
        .setup(move |app, _api| {

            let templates_path = std::path::Path::new(templates.as_str());

            let mut reg = Handlebars::new();

            #[cfg(debug_assertions)]
            reg.set_dev_mode(true);

            register_templates(&mut reg, templates_path.to_str().unwrap()).unwrap();

            app.manage(Arc::new(reg));

            println!("Initialized templates plugin");

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("templates", move |app, request, responder| {})
    .build()

}


//
// Private functions
//

fn register_templates(handlebars: &mut Handlebars, dir: &str) -> Result<(), Box<dyn std::error::Error>> { 
    for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let fullpath = entry.path();
            if let Some(extension) = fullpath.extension() {
                if extension == "hbs" {
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

fn read_dir_recursive(dir: PathBuf, cb: &dyn Fn(&std::fs::DirEntry)) -> std::io::Result<()> {
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