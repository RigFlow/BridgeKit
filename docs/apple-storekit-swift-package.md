# Apple StoreKit Swift package

`native/apple/BridgeKitStoreKit` is a Swift package that implements the first
BridgeKit Apple native slice: StoreKit 2 product lookup.

It exports the C ABI expected by the Rust `apple-storekit-ffi` feature:

```c
char *bridgekit_storekit_products_json(const char *request_json);
void bridgekit_string_free(char *value);
```

## What is implemented

`bridgekit_storekit_products_json`:

1. decodes `AppleProductRequest` JSON from Rust
2. calls StoreKit 2 `Product.products(for:)`
3. maps StoreKit products into BridgeKit `AppleProduct` JSON
4. returns a newly allocated UTF-8 C string

`bridgekit_string_free` releases strings returned by the package.

The current implementation only covers product lookup. Purchase, restore,
receipt validation, and APNs registration still return `ProviderUnavailable`
from the Rust scaffolds.

## Link from a Tauri Apple app

Enable the Rust FFI feature in the Tauri app's `src-tauri/Cargo.toml`:

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["apple-storekit-ffi"] }
```

Then link `native/apple/BridgeKitStoreKit` into the Xcode project or Swift
package used by the macOS/iOS Tauri target so the exported symbols are available
to the Rust binary.

The exact linking step depends on the app's Apple build setup:

- add `BridgeKitStoreKit` as a local Swift Package dependency in Xcode, or
- build it as a dynamic library and link it from the Tauri Apple target.

## Product response mapping

The Swift package maps StoreKit values to BridgeKit fields:

| BridgeKit JSON field | StoreKit source |
| --- | --- |
| `id` | `Product.id` |
| `localizedTitle` | `Product.displayName` |
| `localizedDescription` | `Product.description` |
| `displayPrice` | `Product.displayPrice` |
| `currencyCode` | `Product.priceFormatStyle.currencyCode` |
| `subscriptionPeriod` | `Product.subscription?.subscriptionPeriod` as ISO 8601 |
| `introductoryPrice` | `Product.subscription?.introductoryOffer?.displayPrice` |
| `trialPeriod` | `Product.subscription?.introductoryOffer?.period` as ISO 8601 |
| `raw.source` | `storekit` |
| `raw.type` | StoreKit product type |

## Validate on Apple hardware

From `native/apple/BridgeKitStoreKit` on macOS:

```sh
swift test
swift build
```

Runtime product lookup also requires:

- an App Store Connect app record
- In-App Purchase capability
- product IDs matching requests from the Tauri app
- a signed app with the matching bundle identifier
- a sandbox account or StoreKit testing configuration
