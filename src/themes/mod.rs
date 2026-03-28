use std::sync::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

use tauri::webview;
use tauri::App;
use tauri::AppHandle;
use tauri::Error;
use tauri::Result;
use tauri::Builder;
use tauri::Manager;
use tauri::Runtime;
use tauri::WebviewUrl;
use tauri::WebviewWindow;

use tauri::plugin::Builder as PluginBuilder;
use tauri::plugin::TauriPlugin;

use tauri::utils::config::WindowConfig;



mod github_adapter;



static THEME_MANAGER: OnceLock<Mutex<Themes>> = OnceLock::new();





type Colors = Vec<ColorItem>;

#[derive(Deserialize, Debug)]
struct ColorItem {
    name: String,
    value: String,
}

pub type ThemeList = HashMap<String, PathBuf>;

#[derive(Debug, Clone)]
pub struct Themes {
    list: ThemeList,
    pub default: String,
    internal_dir: String,
    user_dir: Option<String>,
}

impl Themes {

    pub fn new(default: String, internal_dir: String) -> Self {
        Self {
            list: ThemeList::new(),
            default,
            internal_dir,
            user_dir: None,
        }
    }

    pub fn add(&mut self, theme_name: String, file_path: PathBuf) {
        self.list.insert(theme_name, file_path);
    }
    
    pub fn add_many(&mut self, items: ThemeList) {
        self.list.extend(items);
    }

    pub fn get(&self, theme_name: &str) -> Option<&PathBuf> {
        self.list.get(theme_name)
    }

    pub fn get_many<'a>(&'a self, theme_names: &[String]) -> Vec<Option<&'a PathBuf>> {
        theme_names.iter().map(|name| self.list.get(name)).collect()
    }

    pub fn remove(&mut self, theme_name: &str) {
        self.list.remove(theme_name);
    }

    pub fn remove_many(&mut self, theme_names: &[String]) {
        for name in theme_names {
            self.list.remove(name);
        }
    }

    pub fn reload(&mut self) -> std::io::Result<()> {
        self.list.clear();
        let internal_dir = self.internal_dir.clone();
        self.load_from_directory(&internal_dir)?;
        if let Some(user_dir) = &self.user_dir {
            let user_dir = user_dir.clone();
            self.load_from_directory(&user_dir)?;
        }
    
        Ok(())
    }

    pub fn contains(&self, theme_name: &str) -> bool {
        self.list.contains_key(theme_name)
    }

    pub fn list(&self) -> ThemeList {
        self.list.clone()
    }

    pub fn parse_to_string(&self, theme_name: &str) -> Option<String> {
        let mut theme_str = String::new();
        if self.contains(theme_name) {
            if let Some(path_to_file) = self.get(theme_name) {
                let colors = self.read_file(path_to_file).unwrap();
                theme_str += "const root = document.documentElement;\n";
                for color in colors {
                    theme_str += &format!("root.style.setProperty('{}', '{}');\n", color.name, color.value)
                }
                return Some(theme_str);
            }
        }
        None
    }

    fn read_file(&self, file_path: &PathBuf) -> std::io::Result<Colors> {
        let file_content = std::fs::read_to_string(file_path)?;
        let themes: Colors = serde_json::from_str(&file_content)?;
        Ok(themes)
    }

    fn is_valid_theme(&self, file_path: &std::path::Path) -> bool {
        match std::fs::read_to_string(file_path) {
            Ok(content) => serde_json::from_str::<serde_json::Value>(&content).is_ok(),
            Err(_) => false,
        }
    }
    
    fn load_from_directory(&mut self, dir_path: &str) -> std::io::Result<()> {
        let dir = std::path::Path::new(dir_path);
        if !dir.is_dir() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Directory not found: ".to_owned() + dir_path));
        }
        let themes: ThemeList = std::fs::read_dir(dir)?
            .filter_map(std::io::Result::ok)
            .filter(|entry| entry.path().is_file())
            .filter(|entry| self.is_valid_theme(&entry.path()))
            .filter_map(|entry| {
                entry.path().file_name()?.to_str().map(|file_name| {
                    let theme_name = file_name.trim_end_matches(".json").to_string();
                    (theme_name, entry.path())
                })
            })
            .collect();
        self.list.extend(themes);
        Ok(())
    }

}











#[allow(dead_code)]
pub trait BuilderThemesExt<R: Runtime> {
    fn use_themes(self, default_theme: &str, internal_dir: &str) -> Self;
    fn load_themes(self) -> Self;
}

impl<R: Runtime> BuilderThemesExt<R> for Builder<R> {
    
    /// Allow to use themes
    fn use_themes(self, default_theme: &str, internal_dir: &str) -> Self {
        THEME_MANAGER.set(Mutex::new(Themes::new(default_theme.to_string(), internal_dir.to_string())));
        self.load_themes()
    }
    
    /// Load themes
    fn load_themes(self) -> Self {
        let mut themes = THEME_MANAGER.get().unwrap().lock().unwrap();
        themes.reload().unwrap();
        self
    }

}












//
// Enhance App with a custom create window fn
//
#[allow(dead_code)]
pub trait AppHandleThemesExt<R: Runtime> {
    fn list_themes(&self) -> ThemeList;
    fn apply_theme_to_all(&self, theme_name: &str) -> Result<()>;
}

impl<R: Runtime> AppHandleThemesExt<R> for AppHandle<R> {
    
    /// List all available themes
    fn list_themes(&self) -> ThemeList {
        let themes = THEME_MANAGER.get().unwrap().lock().unwrap();
        themes.list()
    }
    
    /// Apply theme to all windows
    fn apply_theme_to_all(&self, theme_name: &str) -> Result<()> {
        for (_, win) in self.webview_windows() {
            win.apply_theme(theme_name)?          
        }
        Ok(())
    }

}












//
// Enhance App with a custom create window fn
//
#[allow(dead_code)]
pub trait AppThemesExt<R: Runtime> {
    fn apply_theme_to_all(&self, theme_name: &str) -> Result<()>;
}

impl<R: Runtime> AppThemesExt<R> for App<R> {    
    /// Apply theme to all windows
    fn apply_theme_to_all(&self, theme_name: &str) -> Result<()> {
        for (_, win) in self.webview_windows() {
            win.apply_theme(theme_name)?          
        }
        Ok(())
    }
}










pub trait WebviewWindowThemesExt<R: Runtime> {
    fn init_theme(&self) -> Result<()>;
    fn apply_theme(&self, theme_name: &str) -> Result<()>;
}

impl <R: Runtime> WebviewWindowThemesExt<R> for WebviewWindow<R> {

    /// Init theme
    fn init_theme(&self) -> Result<()> {
        let themes = THEME_MANAGER.get().unwrap().lock().unwrap();
        let theme_str = themes.parse_to_string(&themes.default).unwrap();
        self.eval(&format!(r#"(function (){{
            window.addEventListener('DOMContentLoaded', () => {{
                {}
            }})
        }})();"#, theme_str))?;
        Ok(())
    }

    /// Apply theme to window
    fn apply_theme(&self, theme_name: &str) -> Result<()> {
        let themes = THEME_MANAGER.get().unwrap().lock().unwrap();
        let theme_str = themes.parse_to_string(theme_name).unwrap();
        self.eval(&format!(r#"(function (){{
            {}
        }})();"#, theme_str))?;
        Ok(())
    }

}