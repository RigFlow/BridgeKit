# Tauri plugin

The `tauri-plugin-bridgekit` crate registers BridgeKit as managed Tauri state
and exposes standard commands for purchases, receipt validation, and push
notifications.

## Install

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit" }
tauri-plugin-bridgekit = { git = "https://github.com/RigFlow/BridgeKit" }
```

## Rust setup

Create a `BridgeKit` instance with native, Apple, Microsoft, or mock providers,
then register the plugin:

```rust
use bridgekit::BridgeKit;

fn main() {
    let bridgekit = BridgeKit::native();

    tauri::Builder::default()
        .plugin(tauri_plugin_bridgekit::init(bridgekit))
        .run(tauri::generate_context!())
        .expect("failed to run Tauri app");
}
```

For Apple store builds:

```rust
let bridgekit = BridgeKit::apple(storekit_client, apns_client);
```

For Microsoft Store builds:

```rust
let bridgekit = BridgeKit::microsoft(store_client, wns_client);
```

## Commands

The plugin exposes:

- `bridgekit_products`
- `bridgekit_purchase`
- `bridgekit_restore_purchases`
- `bridgekit_validate_receipt`
- `bridgekit_request_push_authorization`
- `bridgekit_register_push`
- `bridgekit_unregister_push`

## Frontend invocation

Use the TypeScript SDK for typed frontend calls:

```ts
import { purchase, registerPush } from "@bridgekit/tauri";

const transaction = await purchase({ productId: "pro.monthly" });
const registration = await registerPush({ environment: "production" });
```

See [`typescript-sdk.md`](typescript-sdk.md) for the full SDK API.

Tauri plugin commands are invoked through the plugin namespace:

```ts
import { invoke } from "@tauri-apps/api/core";

const products = await invoke("plugin:bridgekit|bridgekit_products", {
  request: {
    productIds: ["pro.monthly"],
    storefront: "apple_app_store",
  },
});

const purchase = await invoke("plugin:bridgekit|bridgekit_purchase", {
  request: {
    productId: "pro.monthly",
  },
});

const pushRegistration = await invoke("plugin:bridgekit|bridgekit_register_push", {
  request: {
    environment: "production",
  },
});
```

The command payloads use the same camelCase fields as the Rust BridgeKit
request/response structs.
