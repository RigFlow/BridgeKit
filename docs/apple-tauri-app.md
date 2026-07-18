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

## Link the Swift package

The Rust `apple-*-ffi` features expect the Swift symbols to be linked into the
Apple Tauri binary. From Xcode or your macOS/iOS build pipeline:

1. add `native/apple/BridgeKitStoreKit` as a local Swift Package dependency
2. link the `BridgeKitStoreKit` dynamic library into the Tauri Apple target

See [`apple-storekit-swift-package.md`](apple-storekit-swift-package.md) for the
exported symbols and JSON contracts.

## Forward APNs device tokens

`bridgekit_apns_register_json` waits for a device token forwarded from the app
delegate. In the consuming macOS/iOS app delegate:

```swift
func application(
    _ application: UIApplication,
    didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
) {
    let tokenHex = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
    bridgekitApnsForwardDeviceTokenHex(tokenHex)
}
```

Use the equivalent `NSApplicationDelegate` callback on macOS.

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
