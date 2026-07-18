# Apple StoreKit Swift package

`native/apple/BridgeKitStoreKit` is a Swift package that implements BridgeKit's
Apple native slices:

- StoreKit 2 product lookup, purchase, restore purchases, and receipt validation
- APNs authorization, registration, and unregister

It exports the C ABIs expected by the Rust `apple-storekit-ffi` and
`apple-apns-ffi` features:

```c
char *bridgekit_storekit_products_json(const char *request_json);
char *bridgekit_storekit_purchase_json(const char *request_json);
char *bridgekit_storekit_restore_purchases_json(void);
char *bridgekit_storekit_validate_receipt_json(const char *request_json);
char *bridgekit_apns_request_authorization_json(const char *request_json);
char *bridgekit_apns_register_json(const char *request_json);
void bridgekit_apns_unregister(void);
void bridgekit_apns_forward_device_token_hex(const char *token_hex);
void bridgekit_apple_bootstrap(void);
void bridgekit_string_free(char *value);
```

## What is implemented

`bridgekit_storekit_products_json`:

1. decodes `AppleProductRequest` JSON from Rust
2. calls StoreKit 2 `Product.products(for:)`
3. maps StoreKit products into BridgeKit `AppleProduct` JSON
4. returns a newly allocated UTF-8 C string

`bridgekit_storekit_purchase_json`:

1. decodes `ApplePurchaseRequest` JSON from Rust
2. loads the requested StoreKit product
3. calls `product.purchase(options:)`
4. maps StoreKit purchase results into BridgeKit `AppleTransaction` JSON
5. returns a newly allocated UTF-8 C string

`bridgekit_storekit_restore_purchases_json`:

1. enumerates StoreKit 2 `Transaction.currentEntitlements`
2. maps verified entitlements into BridgeKit `AppleTransaction` JSON with
   `state: "restored"`
3. returns a newly allocated UTF-8 C string containing a JSON array

`bridgekit_storekit_validate_receipt_json`:

1. decodes `AppleReceiptValidationRequest` JSON from Rust
2. searches verified StoreKit 2 transactions in current entitlements and
   transaction history
3. maps a matching verified transaction into BridgeKit
   `AppleReceiptValidationResult` JSON
4. returns a newly allocated UTF-8 C string

`bridgekit_apns_request_authorization_json`:

1. decodes `ApplePushAuthorizationRequest` JSON from Rust
2. calls `UNUserNotificationCenter.requestAuthorization`
3. maps the resulting authorization status into BridgeKit
   `ApplePushAuthorization` JSON

`bridgekit_apns_register_json`:

1. decodes `ApplePushRegistrationRequest` JSON from Rust
2. calls `registerForRemoteNotifications`
3. waits for a device token forwarded through
   `bridgekit_apns_forward_device_token_hex`
4. returns a newly allocated UTF-8 C string containing
   `ApplePushRegistration` JSON

`bridgekit_apns_unregister` clears cached token state and unregisters from
remote notifications on iOS.

`bridgekit_apple_bootstrap` installs a BridgeKit app delegate that forwards APNs
device tokens to `bridgekit_apns_forward_device_token_hex`. Tauri apps should
call `bridgekit::bootstrap_apple_runtime()` during startup.

`bridgekit_string_free` releases strings returned by the package.

## Link from a Tauri Apple app

Enable the Rust FFI features in the Tauri app's `src-tauri/Cargo.toml`:

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["apple-storekit-ffi", "apple-apns-ffi"] }
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

## Purchase response mapping

The Swift package maps StoreKit purchase results to BridgeKit transaction
states:

| StoreKit result | BridgeKit state |
| --- | --- |
| verified success | `purchased` |
| unverified success | `failed` |
| user cancelled | `cancelled` |
| pending | `deferred` |
| unknown result | `failed` |

For verified transactions, the package calls `transaction.finish()` before
returning the response.

## Restore response mapping

The Swift package maps StoreKit current entitlements to BridgeKit transaction
states:

| StoreKit entitlement | BridgeKit state |
| --- | --- |
| verified entitlement | `restored` |
| unverified entitlement | `failed` |

Restore does not call `transaction.finish()` because it reads existing
entitlements rather than completing a new purchase flow.

## Receipt validation mapping

The Swift package validates on-device by matching the request against verified
StoreKit 2 transactions:

| Match result | `isValid` | `raw.verification` |
| --- | --- | --- |
| verified transaction found | `true` | `verified` |
| matching unverified transaction | `false` | `unverified` |
| no match | `false` | `not_found` |

## APNs mapping

Authorization statuses map to BridgeKit `status` values:

| `UNAuthorizationStatus` | BridgeKit status |
| --- | --- |
| `.notDetermined` | `not_determined` |
| `.denied` | `denied` |
| `.authorized` | `authorized` |
| `.provisional` | `provisional` |
| `.ephemeral` | `ephemeral` |

Device tokens are returned as lowercase hex strings in `deviceToken`.

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

Runtime purchase testing additionally requires products to be cleared for
sandbox testing and a StoreKit flow that can present App Store purchase UI.

Runtime APNs registration additionally requires:

- Push Notifications capability
- app delegate forwarding of device tokens to
  `bridgekit_apns_forward_device_token_hex`
