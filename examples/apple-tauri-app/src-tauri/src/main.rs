fn main() {
    let bridgekit = bridgekit::BridgeKit::native();

    tauri::Builder::default()
        .plugin(tauri_plugin_bridgekit::init(bridgekit))
        .run(tauri::generate_context!())
        .expect("failed to run BridgeKit Apple Tauri app");
}
