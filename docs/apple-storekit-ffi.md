# Apple StoreKit FFI product lookup

BridgeKit includes a first native StoreKit integration point for product lookup.
It is compiled only for macOS/iOS and only when the `apple-storekit-ffi` feature
is enabled.

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["apple-storekit-ffi"] }
```

With that feature enabled, `NativeAppleStoreKitClient::products` serializes an
`AppleProductRequest` to JSON, calls a native C ABI symbol, and parses the
returned JSON into `Vec<AppleProduct>`.

Other Apple native operations still return `ProviderUnavailable` until their
native bindings are implemented.

## Required native symbols

The consuming Apple app or native support library must export:

```c
char *bridgekit_storekit_products_json(const char *request_json);
void bridgekit_string_free(char *value);
```

Ownership contract:

- `request_json` is a borrowed UTF-8 JSON string owned by Rust.
- `bridgekit_storekit_products_json` returns a newly allocated UTF-8 JSON string.
- Rust calls `bridgekit_string_free` exactly once for non-null returned strings.
- Returning null is treated as `ProviderUnavailable`.

## Request JSON

The request is `AppleProductRequest` with camelCase fields:

```json
{
  "productIds": ["pro.monthly"],
  "appAccountToken": "optional-user-or-account-token",
  "metadata": {
    "optional": true
  }
}
```

## Response JSON

Return an array of `AppleProduct` values:

```json
[
  {
    "id": "pro.monthly",
    "localizedTitle": "Pro Monthly",
    "localizedDescription": "Monthly Pro access",
    "displayPrice": "$4.99",
    "currencyCode": "USD",
    "subscriptionPeriod": "P1M",
    "trialPeriod": "P7D",
    "raw": {
      "source": "storekit"
    }
  }
]
```

`subscriptionPeriod`, `introductoryPrice`, and `trialPeriod` are optional.
Periods should use ISO 8601 durations such as `P1M` or `P1Y`.

## StoreKit implementation notes

The native implementation should use StoreKit 2 product lookup:

```swift
let products = try await Product.products(for: productIds)
```

Map StoreKit products into the response JSON above. Include StoreKit-native
values in `raw` when useful for debugging, but avoid secrets or personally
identifying data.

The exported C ABI is synchronous from Rust's perspective. If the Swift
implementation performs async StoreKit work internally, ensure it does not block
the app's main actor indefinitely.

## App Store Connect setup

To validate this on Apple hardware:

- create the app in App Store Connect
- enable In-App Purchase for the app identifier
- create product IDs that match the `productIds` request
- sign the Tauri app with the matching bundle identifier
- test with a sandbox account or StoreKit testing configuration
