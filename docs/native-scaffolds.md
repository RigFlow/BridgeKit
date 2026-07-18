# Native implementation scaffolds

BridgeKit now includes platform-gated native scaffold clients. They are wired
into `BridgeKit::native()` on store platforms, but intentionally return
`ProviderUnavailable` until real native framework bindings are implemented.

This gives apps and downstream crates stable type names and wiring points while
keeping Linux CI and local mock development buildable.

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

StoreKit product lookup and purchase have feature-gated FFI integration points
behind `apple-storekit-ffi`. See
[`apple-storekit-ffi.md`](apple-storekit-ffi.md) for the ABI and JSON contract.

The Swift package at `native/apple/BridgeKitStoreKit` implements that ABI. See
[`apple-storekit-swift-package.md`](apple-storekit-swift-package.md) for Apple
linking instructions.

Other StoreKit and APNs operations return
`BridgeKitError::ProviderUnavailable` with an operation-specific message until
their native bindings are implemented.

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
- `NativeAppleStoreKitClient::restore_purchases`
- `NativeAppleStoreKitClient::validate_receipt`
- `NativeApplePushClient::request_authorization`
- `NativeApplePushClient::register`
- `NativeApplePushClient::unregister`

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

Until native bindings are implemented, Microsoft Store and WNS operations return
`BridgeKitError::ProviderUnavailable` with an operation-specific message.

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

Fill in:

- `NativeMicrosoftStoreClient::products`
- `NativeMicrosoftStoreClient::purchase`
- `NativeMicrosoftStoreClient::restore_purchases`
- `NativeMicrosoftStoreClient::validate_license`
- `NativeMicrosoftPushClient::request_authorization`
- `NativeMicrosoftPushClient::register`
- `NativeMicrosoftPushClient::unregister`

The native layer should normalize Microsoft Store/WNS outputs into the existing
`bridgekit::microsoft` request and response structs before returning them.
