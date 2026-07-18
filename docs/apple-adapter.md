# Apple adapter

BridgeKit's Apple adapter lives in `bridgekit::apple`. It implements the common
`StoreProvider` and `PushProvider` traits by delegating native calls to two
Apple-specific client traits:

- `AppleStoreKitClient`
- `ApplePushClient`

This keeps the public Tauri command API stable while allowing the platform code
to live in the layer that can actually talk to Apple frameworks, such as a Tauri
plugin, Swift package, or macOS/iOS app delegate integration.

BridgeKit also exposes macOS/iOS-gated native scaffold clients under
`bridgekit::apple::native`. See [`native-scaffolds.md`](native-scaffolds.md) for
the current placeholder wiring and required Apple capabilities.

StoreKit product lookup has a feature-gated native FFI path. See
[`apple-storekit-ffi.md`](apple-storekit-ffi.md) for the required symbols and
JSON contract.

## Wiring

```rust
use bridgekit::apple::{ApplePushClient, AppleStoreKitClient};
use bridgekit::BridgeKit;
use std::sync::Arc;

fn create_bridgekit(
    storekit: Arc<dyn AppleStoreKitClient>,
    apns: Arc<dyn ApplePushClient>,
) -> BridgeKit {
    BridgeKit::apple(storekit, apns)
}
```

Use `BridgeKit::apple(...)` when the host app has concrete StoreKit/APNs
clients. Use `BridgeKit::native()` only when the app wants BridgeKit to select
defaults; today it intentionally returns explicit unsupported errors until
first-party native clients are linked.

## StoreKit client responsibilities

Implement `AppleStoreKitClient` with native StoreKit calls:

| BridgeKit method | Apple client method | Recommended Apple API |
| --- | --- | --- |
| `products` | `AppleStoreKitClient::products` | StoreKit 2 `Product.products(for:)` |
| `purchase` | `AppleStoreKitClient::purchase` | StoreKit 2 `Product.purchase(...)` |
| `restore_purchases` | `AppleStoreKitClient::restore_purchases` | StoreKit 2 current entitlements / transaction updates |
| `validate_receipt` | `AppleStoreKitClient::validate_receipt` | App Store Server API or signed transaction JWS validation |

The client should map Apple responses into:

- `AppleProduct`
- `AppleTransaction`
- `AppleReceiptValidationResult`

BridgeKit maps those into shared `Product`, `Purchase`, and
`ReceiptValidationResult` values for Tauri commands.

## APNs client responsibilities

Implement `ApplePushClient` with native notification APIs:

| BridgeKit method | Apple client method | Recommended Apple API |
| --- | --- | --- |
| `request_push_authorization` | `ApplePushClient::request_authorization` | `UNUserNotificationCenter.requestAuthorization` |
| `register_push` | `ApplePushClient::register` | `UIApplication.registerForRemoteNotifications` or `NSApplication.registerForRemoteNotifications` |
| `unregister_push` | `ApplePushClient::unregister` | app/platform unregister equivalent |

The APNs device token should be returned as `ApplePushRegistration.device_token`.
BridgeKit exposes it as `PushRegistration.token` with `platform: "apple"`.

## Environment handling

BridgeKit accepts `sandbox` and `production` in
`PushRegistrationRequest.environment`. The Apple adapter validates that string
and passes an `AppleEnvironment` to the native APNs client.

Receipt validation can also be configured with an environment by constructing
`AppleStoreProvider::new(client).with_environment(...)` directly and passing it
to `BridgeKit::with_providers(...)`.

## Tauri integration shape

For a Tauri app, keep platform calls in the native layer and keep frontend calls
going through BridgeKit commands:

```rust
#[tauri::command]
async fn iap_purchase(
    state: tauri::State<'_, AppState>,
    request: bridgekit::PurchaseRequest,
) -> Result<bridgekit::Purchase, String> {
    state
        .bridgekit
        .purchase(request)
        .await
        .map_err(|error| error.to_string())
}
```

The app's Apple client implementation can be backed by Swift/Objective-C code,
an FFI boundary, or a Tauri plugin command dispatcher. The important boundary is
that all native outcomes are normalized into the `bridgekit::apple` structs
before they reach shared application code.
