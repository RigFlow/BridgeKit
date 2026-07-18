# Apple StoreKit FFI

BridgeKit includes native StoreKit integration points for product lookup,
purchase, and restore purchases. They are compiled only for macOS/iOS and only
when the `apple-storekit-ffi` feature is enabled.

The repository includes a Swift implementation of this ABI at
`native/apple/BridgeKitStoreKit`. See
[`apple-storekit-swift-package.md`](apple-storekit-swift-package.md) for linking
instructions.

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["apple-storekit-ffi"] }
```

With that feature enabled:

- `NativeAppleStoreKitClient::products` serializes an `AppleProductRequest`,
  calls a native C ABI symbol, and parses `Vec<AppleProduct>`.
- `NativeAppleStoreKitClient::purchase` serializes an `ApplePurchaseRequest`,
  calls a native C ABI symbol, and parses `AppleTransaction`.
- `NativeAppleStoreKitClient::restore_purchases` calls a native C ABI symbol and
  parses `Vec<AppleTransaction>`.
- `NativeAppleStoreKitClient::validate_receipt` serializes an
  `AppleReceiptValidationRequest`, calls a native C ABI symbol, and parses
  `AppleReceiptValidationResult`.

Other Apple native operations still return `ProviderUnavailable` until their
native bindings are implemented.

## Required native symbols

The consuming Apple app or native support library must export:

```c
char *bridgekit_storekit_products_json(const char *request_json);
char *bridgekit_storekit_purchase_json(const char *request_json);
char *bridgekit_storekit_restore_purchases_json(void);
char *bridgekit_storekit_validate_receipt_json(const char *request_json);
void bridgekit_string_free(char *value);
```

Ownership contract:

- `request_json` is a borrowed UTF-8 JSON string owned by Rust.
- StoreKit functions return newly allocated UTF-8 JSON strings.
- Rust calls `bridgekit_string_free` exactly once for non-null returned strings.
- Returning null is treated as `ProviderUnavailable`.

## Product request JSON

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

## Product response JSON

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

## Purchase request JSON

The request is `ApplePurchaseRequest` with camelCase fields:

```json
{
  "productId": "pro.monthly",
  "quantity": 1,
  "appAccountToken": "4b5f5e4f-1e59-4326-90cc-13485c5fdbd4",
  "metadata": {
    "optional": true
  }
}
```

`appAccountToken` should be a UUID string when the Swift implementation should
pass it through to StoreKit's `Product.PurchaseOption.appAccountToken`.

## Purchase response JSON

Return an `AppleTransaction` value:

```json
{
  "transactionId": "100000000000001",
  "productId": "pro.monthly",
  "state": "purchased",
  "signedTransactionJws": "signed-transaction-jws",
  "originalTransactionId": "100000000000000",
  "purchasedAtMs": 1700000000000,
  "expiresAtMs": 1702592000000,
  "raw": {
    "source": "storekit",
    "verification": "verified"
  }
}
```

User-cancelled purchases should return `state: "cancelled"`. Pending StoreKit
purchases are mapped to `state: "deferred"`.

## Restore purchases response JSON

`bridgekit_storekit_restore_purchases_json` takes no request body. Return an
array of `AppleTransaction` values representing the user's current StoreKit
entitlements:

```json
[
  {
    "transactionId": "100000000000002",
    "productId": "pro.monthly",
    "state": "restored",
    "signedTransactionJws": "signed-transaction-jws",
    "originalTransactionId": "100000000000000",
    "purchasedAtMs": 1700000000000,
    "expiresAtMs": 1702592000000,
    "raw": {
      "source": "storekit",
      "verification": "verified"
    }
  }
]
```

Verified current entitlements should use `state: "restored"`. Unverified
entitlements should use `state: "failed"`.

## Receipt validation request JSON

The request is `AppleReceiptValidationRequest` with camelCase fields:

```json
{
  "receipt": "signed-transaction-jws",
  "transactionId": "100000000000001",
  "productId": "pro.monthly",
  "environment": "sandbox",
  "metadata": {
    "optional": true
  }
}
```

`receipt` should contain a StoreKit 2 signed transaction JWS when validating on
device. `transactionId` and `productId` are optional filters.

## Receipt validation response JSON

Return an `AppleReceiptValidationResult` value:

```json
{
  "isValid": true,
  "productId": "pro.monthly",
  "transactionId": "100000000000001",
  "expiresAtMs": 1702592000000,
  "raw": {
    "source": "storekit",
    "verification": "verified"
  }
}
```

When no matching verified transaction is found, return `isValid: false` with
`raw.verification: "not_found"`.

## StoreKit implementation notes

The native implementation should use StoreKit 2 product lookup:

```swift
let products = try await Product.products(for: productIds)
```

Purchases should use StoreKit 2 purchase:

```swift
let result = try await product.purchase(options: options)
```

Restore purchases should enumerate StoreKit 2 current entitlements:

```swift
for await verificationResult in Transaction.currentEntitlements {
    // map verified entitlements to restored transactions
}
```

Receipt validation should match the request against verified StoreKit 2
transactions from current entitlements and transaction history:

```swift
for await verificationResult in Transaction.currentEntitlements {
    // compare signed transaction JWS or transaction/product identifiers
}
for await verificationResult in Transaction.all {
    // compare signed transaction JWS or transaction/product identifiers
}
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
