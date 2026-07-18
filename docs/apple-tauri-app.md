# Apple Tauri example

The `examples/apple-tauri-app` project demonstrates BridgeKit against native
StoreKit and APNs bindings on macOS/iOS.

## Stack

- `BridgeKit::native()` with `apple-storekit-ffi` and `apple-apns-ffi`
- `tauri-plugin-bridgekit`
- `@bridgekit/tauri` TypeScript SDK
- `native/apple/BridgeKitStoreKit` Swift package for the StoreKit/APNs C ABI

## Configure product IDs

Update `src/main.ts` so `PRODUCT_ID` matches an In-App Purchase configured in
App Store Connect for the app's bundle identifier.

## Swift package build integration

The example's `src-tauri/build.rs` builds and links
`native/apple/BridgeKitStoreKit` automatically on macOS:

1. runs `swift build` for the active Cargo profile
2. links `libBridgeKitStoreKit.dylib` from `.build/<profile>`
3. sets an rpath so the dynamic library can be resolved at runtime

If you need to build the Swift package manually:

```sh
./examples/apple-tauri-app/scripts/link-storekit.sh debug
```

## Bootstrap APNs forwarding

`bridgekit_apns_register_json` waits for a device token forwarded from the app
delegate. The example calls `bridgekit::bootstrap_apple_runtime()` during Tauri
startup, which installs a BridgeKit `NSApplicationDelegate` (or
`UIApplicationDelegate` on iOS) that forwards tokens to
`bridgekit_apns_forward_device_token_hex`.

If you integrate BridgeKit into a custom Apple host, call
`bridgekit::bootstrap_apple_runtime()` before registering for push notifications,
or forward tokens manually from your own delegate:

```swift
func application(
    _ application: NSApplication,
    didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
) {
    let tokenHex = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
    bridgekitApnsForwardDeviceTokenHex(tokenHex)
}
```

See [`apple-storekit-swift-package.md`](apple-storekit-swift-package.md) for the
exported symbols and JSON contracts.

## Required capabilities

- In-App Purchase
- Push Notifications
- signed build with the App Store Connect bundle identifier
- sandbox account or StoreKit testing configuration

## Run

```sh
npm install
cd examples/apple-tauri-app
npm run tauri dev
```

On-device StoreKit and APNs flows require Apple hardware and a signed Store build.

For a full validation pass, see [`hardware-test-checklist.md`](hardware-test-checklist.md).
