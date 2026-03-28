





pub fn tooling_plugin<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::<R>::new("tauri-core-tools")
        .invoke_handler(tauri::generate_handler![show_window])
        .setup(|_app, _api| {
            Ok(())
        })
        .on_event(|app_handle, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, code, .. } => {
                    //api.prevent_exit();
                    std::process::exit(0);
                }
                _ => {}
            }
        })
        .build()
}







