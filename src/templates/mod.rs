use tauri::AppHandle;
use tauri::Manager;
use tauri::Builder;
use tauri::Runtime;
use tauri::State;

use serde::Serialize;

use std::sync::OnceLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use handlebars::Handlebars;
use walkdir::WalkDir;




#[derive(Clone, Debug)]
pub struct Templates(Arc<Handlebars<'static>>);

pub type Template = Arc<Handlebars<'static>>;


static TEMPLATES: OnceLock<Templates> = OnceLock::new();

#[allow(dead_code)]
pub trait BuilderExtTemplates<R: Runtime> {
    fn use_template_engine(self, directory: &str, extension: &str) -> Self;
}

impl<R: Runtime> BuilderExtTemplates<R> for Builder<R> {

    fn use_template_engine(self, directory: &str, extension: &str) -> Self {
    
        let mut file_type = extension;
        if !file_type.contains(".") {
            file_type = file_type.trim_start_matches(".");
        }

        let mut reg = Handlebars::new();

        #[cfg(debug_assertions)]
        reg.set_dev_mode(true);

        register_templates(&mut reg, directory, file_type).unwrap();
        
        TEMPLATES.set(Templates(Arc::new(reg))).unwrap();

        self

    }    

}


#[allow(dead_code)]
pub trait AppHandleExtTemplates<R: Runtime> {
    fn template<T>(&self, path: &str, data: &T) -> String
    where
        T: Serialize;
}

impl<R: Runtime> AppHandleExtTemplates<R> for AppHandle<R> {

    fn template<T>(&self, path: &str, data: &T) -> String
    where
        T: Serialize,
    {

        let templates = TEMPLATES.get().expect("templates:");

        templates.0.render(path, data).expect("templates:")

    }    

}




//
// Private functions
//

fn register_templates(handlebars: &mut Handlebars, dir: &str, ext: &str) -> Result<(), Box<dyn std::error::Error>> { 
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