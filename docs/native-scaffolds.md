# Native implementation scaffolds

BridgeKit includes platform-gated native clients wired into
`BridgeKit::native()` on Apple and Windows targets. Enable the corresponding FFI
features to activate the native bindings.

## Apple

Available only on macOS and iOS:

```rust
#[cfg(any(target_os = "macos", target_os = "ios"))]
use bridgekit::apple::native::{NativeApplePushClient, NativeAppleStoreKitClient};
```

Scaffold clients:

- `NativeAppleStoreKitClient`
- `NativeApplePushClient`

`BridgeKit::native()` automatically uses those clients on macOS/iOS:

```rust
let bridgekit = bridgekit::BridgeKit::native();
```

StoreKit product lookup, purchase, restore purchases, and receipt validation have
feature-gated FFI integration points behind `apple-storekit-ffi`. See
[`apple-storekit-ffi.md`](apple-storekit-ffi.md) for the ABI and JSON contract.

APNs authorization, registration, and unregister have feature-gated FFI
integration points behind `apple-apns-ffi`. See
[`apple-apns-ffi.md`](apple-apns-ffi.md) for the ABI and JSON contract.

The Swift package at `native/apple/BridgeKitStoreKit` implements those ABIs.
See [`apple-storekit-swift-package.md`](apple-storekit-swift-package.md) for Apple
linking instructions.

Without the FFI features enabled, Apple native operations return
`BridgeKitError::ProviderUnavailable` with an operation-specific message.

### Required Apple setup

Real Apple implementations will need:

- Apple Developer account and app identifier
- App Store Connect app record
- In-App Purchase capability
- product IDs configured in App Store Connect
- Push Notifications capability
- APNs environment handling for `sandbox` and `production`
- App Store Server API or signed transaction JWS validation strategy
- macOS/iOS entitlement updates in the consuming Tauri app

### Implementation points

Fill in:

- `NativeAppleStoreKitClient::products` via the `apple-storekit-ffi` symbols
- `NativeAppleStoreKitClient::purchase` via the `apple-storekit-ffi` symbols
- `NativeAppleStoreKitClient::restore_purchases` via the `apple-storekit-ffi`
  symbols
- `NativeAppleStoreKitClient::validate_receipt` via the `apple-storekit-ffi`
  symbols
- `NativeApplePushClient::request_authorization` via the `apple-apns-ffi`
  symbols
- `NativeApplePushClient::register` via the `apple-apns-ffi` symbols
- `NativeApplePushClient::unregister` via the `apple-apns-ffi` symbols

The native layer should normalize StoreKit/APNs outputs into the existing
`bridgekit::apple` request and response structs before returning them.

## Microsoft

Available only on Windows:

```rust
#[cfg(target_os = "windows")]
use bridgekit::microsoft::native::{
    NativeMicrosoftPushClient,
    NativeMicrosoftStoreClient,
};
```

Scaffold clients:

- `NativeMicrosoftStoreClient`
- `NativeMicrosoftPushClient`

`BridgeKit::native()` automatically uses those clients on Windows:

```rust
let bridgekit = bridgekit::BridgeKit::native();
```

On Windows, enable `microsoft-store-ffi` and/or `microsoft-wns-ffi` to activate the
WinRT bindings. Without those features, Microsoft Store and WNS operations return
`BridgeKitError::ProviderUnavailable` with an operation-specific message.

Microsoft Store and WNS WinRT bindings are available on Windows behind
`microsoft-store-ffi` and `microsoft-wns-ffi`. See
[`microsoft-store-ffi.md`](microsoft-store-ffi.md) and
[`microsoft-wns-ffi.md`](microsoft-wns-ffi.md).
### Required Microsoft setup

Real Microsoft implementations will need:

- Partner Center app record
- package identity matching the Store app
- Microsoft Store product IDs
- Store purchase/license API access
- collection/subscription validation strategy
- WNS package identity and credentials
- WNS channel URI registration and refresh handling
- Windows app manifest/capability updates in the consuming Tauri app

### Implementation points

The WinRT bindings are implemented in:

- `bridgekit::winrt_store` for Microsoft Store operations
- `bridgekit::winrt_wns` for WNS operations

Enable these features in the consuming Tauri app's `Cargo.toml`:

```toml
bridgekit = { features = ["microsoft-store-ffi", "microsoft-wns-ffi"] }
```

The native layer should normalize Microsoft Store/WNS outputs into the existing
`bridgekit::microsoft` request and response structs before returning them.
