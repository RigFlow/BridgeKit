fn main() {
    tauri_build::build();

    #[cfg(target_os = "macos")]
    link_bridgekit_storekit();
}

#[cfg(target_os = "macos")]
fn link_bridgekit_storekit() {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let swift_package = manifest_dir
        .join("../../../native/apple/BridgeKitStoreKit")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../../../native/apple/BridgeKitStoreKit"));

    let build_profile = match env::var("PROFILE").as_deref() {
        Ok("release") => "release",
        _ => "debug",
    };

    rerun_if_swift_sources_changed(&swift_package);

    let output = Command::new("swift")
        .args(["build", "-c", build_profile])
        .current_dir(&swift_package)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "failed to run `swift build` for BridgeKitStoreKit ({error}); \
                 install Xcode command line tools or run scripts/link-storekit.sh"
            );
        });

    if !output.status.success() {
        panic!(
            "swift build failed for BridgeKitStoreKit:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let dylib_dir = swift_package.join(".build").join(build_profile);
    println!("cargo:rustc-link-search=native={}", dylib_dir.display());
    println!("cargo:rustc-link-lib=dylib=BridgeKitStoreKit");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dylib_dir.display());
}

#[cfg(target_os = "macos")]
fn rerun_if_swift_sources_changed(swift_package: &std::path::Path) {
    println!(
        "cargo:rerun-if-changed={}",
        swift_package.join("Package.swift").display()
    );

    let sources = swift_package.join("Sources");
    if let Ok(entries) = std::fs::read_dir(&sources) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
