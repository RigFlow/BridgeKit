fn main() {
    let bridgekit = bridgekit::BridgeKit::native();

    tauri::Builder::default()
        .plugin(tauri_plugin_bridgekit::init(bridgekit))
        .setup(configure_store_context)
        .run(tauri::generate_context!())
        .expect("failed to run BridgeKit Windows Tauri app");
}

#[cfg(windows)]
fn configure_store_context(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;

    let window = app
        .get_webview_window("main")
        .ok_or("main window was not created")?;
    let hwnd = window.hwnd()?;
    bridgekit::set_store_window_handle(hwnd.0 as isize);
    Ok(())
}

#[cfg(not(windows))]
fn configure_store_context(_app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
