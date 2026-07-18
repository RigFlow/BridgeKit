# Microsoft adapter

BridgeKit's Microsoft adapter lives in `bridgekit::microsoft`. It implements
the common `StoreProvider` and `PushProvider` traits by delegating native calls
to two Microsoft-specific client traits:

- `MicrosoftStoreClient`
- `MicrosoftPushClient`

This keeps Tauri commands and frontend code independent from the native Windows
API layer. The host app or plugin can implement these traits with WinRT,
Microsoft Store services, or another native bridge that is available in signed
Microsoft Store builds.

BridgeKit also exposes Windows-gated native clients under
`bridgekit::microsoft::native`. See [`native-scaffolds.md`](native-scaffolds.md)
for the current wiring and required Microsoft Store/WNS setup.

Microsoft Store operations are implemented with WinRT bindings behind
`microsoft-store-ffi`. See [`microsoft-store-ffi.md`](microsoft-store-ffi.md).

WNS operations are implemented with WinRT bindings behind `microsoft-wns-ffi`. See
[`microsoft-wns-ffi.md`](microsoft-wns-ffi.md).

## Wiring

```rust
use bridgekit::microsoft::{MicrosoftPushClient, MicrosoftStoreClient};
use bridgekit::BridgeKit;
use std::sync::Arc;

fn create_bridgekit(
    store: Arc<dyn MicrosoftStoreClient>,
    wns: Arc<dyn MicrosoftPushClient>,
) -> BridgeKit {
    BridgeKit::microsoft(store, wns)
}
```

Use `BridgeKit::microsoft(...)` when the host app has concrete Microsoft Store
and WNS clients. `BridgeKit::native()` uses the Windows native clients on
Windows when the corresponding FFI features are enabled.

## Microsoft Store client responsibilities

Implement `MicrosoftStoreClient` with Microsoft Store or Store services calls:

| BridgeKit method | Microsoft client method | Recommended Microsoft API |
| --- | --- | --- |
| `products` | `MicrosoftStoreClient::products` | Store product query / StoreContext product APIs |
| `purchase` | `MicrosoftStoreClient::purchase` | Store purchase APIs |
| `restore_purchases` | `MicrosoftStoreClient::restore_purchases` | Store license and collection state |
| `validate_receipt` | `MicrosoftStoreClient::validate_license` | Store collections/subscriptions validation |

The client should map Microsoft responses into:

- `MicrosoftProduct`
- `MicrosoftPurchase`
- `MicrosoftLicenseValidationResult`

BridgeKit maps those into shared `Product`, `Purchase`, and
`ReceiptValidationResult` values for Tauri commands.

## WNS client responsibilities

Implement `MicrosoftPushClient` with Windows Push Notification Services calls:

| BridgeKit method | Microsoft client method | Recommended Microsoft API |
| --- | --- | --- |
| `request_push_authorization` | `MicrosoftPushClient::request_authorization` | app-level notification permission/status logic |
| `register_push` | `MicrosoftPushClient::register` | WNS push notification channel creation |
| `unregister_push` | `MicrosoftPushClient::unregister` | channel close/unregister logic |

The WNS channel URI should be returned as
`MicrosoftPushRegistration.channel_uri`. BridgeKit exposes it as
`PushRegistration.token` with `platform: "microsoft"`.

## Environment handling

BridgeKit accepts `sandbox` and `production` in
`PushRegistrationRequest.environment`. The Microsoft adapter validates that
string and passes a `MicrosoftStoreEnvironment` to the WNS client.

License validation can also be configured with an environment by constructing
`MicrosoftStoreProvider::new(client).with_environment(...)` directly and
passing it to `BridgeKit::with_providers(...)`.

## Tauri integration shape

For a Tauri app, keep Windows-specific APIs in the native layer and keep
frontend calls going through BridgeKit commands:

```rust
#[tauri::command]
async fn iap_restore(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<bridgekit::Purchase>, String> {
    state
        .bridgekit
        .restore_purchases()
        .await
        .map_err(|error| error.to_string())
}
```

The app's Microsoft client implementation can be backed by WinRT bindings,
Windows App SDK code, an FFI boundary, or a Tauri plugin command dispatcher.
Normalize native outcomes into the `bridgekit::microsoft` structs before they
reach shared application code.
