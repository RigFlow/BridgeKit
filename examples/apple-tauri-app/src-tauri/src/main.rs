fn main() {
    let bridgekit = bridgekit::BridgeKit::native();

    tauri::Builder::default()
        .plugin(tauri_plugin_bridgekit::init(bridgekit))
        .setup(bootstrap_apple_runtime)
        .run(tauri::generate_context!())
        .expect("failed to run BridgeKit Apple Tauri app");
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn bootstrap_apple_runtime(_app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    bridgekit::bootstrap_apple_runtime();
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn bootstrap_apple_runtime(_app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
