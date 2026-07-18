# BridgeKit

BridgeKit is a reusable Rust bridge layer for Tauri apps that plan to ship
through Apple App Store and Microsoft Store channels. It gives apps one typed
API for:

- in-app product lookup, purchase, restore, and receipt validation
- push notification authorization, registration, and unregister flows
- Apple App Store / APNs and Microsoft Store / WNS provider implementations
- mock providers for local development and automated tests

The current crate establishes the shared contract and mock/native-provider
facade. Platform-specific StoreKit, APNs, Microsoft Store, and WNS code can be
implemented behind the `StoreProvider` and `PushProvider` traits without
changing app-facing Tauri commands.

## Workspace

```text
crates/bridgekit
```

`bridgekit` is the Rust library intended to be imported by Tauri apps.

## Quick start

Add the crate to a Tauri app:

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit" }
```

Create app state with either native providers or mocks:

```rust
use bridgekit::BridgeKit;

pub struct AppState {
    pub bridgekit: BridgeKit,
}

let state = AppState {
    bridgekit: BridgeKit::native(),
};
```

When an app has concrete Apple StoreKit/APNs clients, wire them directly:

```rust
let bridgekit = BridgeKit::apple(storekit_client, apns_client);
```

When an app has concrete Microsoft Store/WNS clients, wire them directly:

```rust
let bridgekit = BridgeKit::microsoft(store_client, wns_client);
```

Expose the bridge through Tauri commands:

```rust
use bridgekit::{
    ProductRequest, PurchaseRequest, PushAuthorizationRequest, PushRegistrationRequest,
};

#[tauri::command]
async fn iap_products(
    state: tauri::State<'_, AppState>,
    request: ProductRequest,
) -> Result<serde_json::Value, String> {
    state
        .bridgekit
        .products(request)
        .await
        .and_then(|products| serde_json::to_value(products).map_err(Into::into))
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn iap_purchase(
    state: tauri::State<'_, AppState>,
    request: PurchaseRequest,
) -> Result<serde_json::Value, String> {
    state
        .bridgekit
        .purchase(request)
        .await
        .and_then(|purchase| serde_json::to_value(purchase).map_err(Into::into))
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn push_authorize(
    state: tauri::State<'_, AppState>,
    request: PushAuthorizationRequest,
) -> Result<serde_json::Value, String> {
    state
        .bridgekit
        .request_push_authorization(request)
        .await
        .and_then(|authorization| serde_json::to_value(authorization).map_err(Into::into))
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn push_register(
    state: tauri::State<'_, AppState>,
    request: PushRegistrationRequest,
) -> Result<serde_json::Value, String> {
    state
        .bridgekit
        .register_push(request)
        .await
        .and_then(|registration| serde_json::to_value(registration).map_err(Into::into))
        .map_err(|error| error.to_string())
}
```

## Native adapter responsibilities

Implementations should satisfy the public provider traits:

- `StoreProvider`
  - `products`
  - `purchase`
  - `restore_purchases`
  - `validate_receipt`
- `PushProvider`
  - `request_authorization`
  - `register`
  - `unregister`

Recommended platform mapping:

| BridgeKit capability | Apple | Microsoft |
| --- | --- | --- |
| product lookup | StoreKit products | Microsoft Store products |
| purchase | StoreKit transaction | Microsoft Store purchase |
| restore | StoreKit current entitlements / receipt | Microsoft Store license state |
| receipt validation | App Store signed transaction / server validation | Microsoft Store collections/subscriptions validation |
| push registration | APNs device token | WNS channel URI |

On unsupported targets, `BridgeKit::native()` returns explicit
`UnsupportedPlatform` errors. This keeps Linux CI and local development
predictable while Apple and Microsoft native adapters are linked in store
builds.

See [`docs/apple-adapter.md`](docs/apple-adapter.md) for the Apple StoreKit/APNs
adapter boundary that is available under `bridgekit::apple`.

See [`docs/microsoft-adapter.md`](docs/microsoft-adapter.md) for the Microsoft
Store/WNS adapter boundary that is available under `bridgekit::microsoft`.

## Local development

Use mocks to exercise app flows before store credentials and signed builds are
available:

```rust
use bridgekit::mock::{MockPushProvider, MockStoreProvider};
use bridgekit::{BridgeKit, Platform, Storefront};
use std::sync::Arc;

let bridgekit = BridgeKit::with_providers(
    Arc::new(MockStoreProvider::new(vec![], Storefront::AppleAppStore)),
    Arc::new(MockPushProvider::new(Platform::Apple, "mock-apns-token")),
);
```

Run checks:

```sh
cargo test
```
