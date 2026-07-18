# Server-side receipt and license validation

BridgeKit includes on-device validation helpers for development and client-side
entitlement checks, but production apps should validate purchases on a trusted
backend before unlocking paid features.

## Apple App Store

### Client responsibilities

- complete purchases through StoreKit
- send the signed transaction JWS (`Purchase.receipt`) to your backend
- refresh entitlements with `restorePurchases` when appropriate

### Server responsibilities

Validate purchases with one of these approaches:

1. **App Store Server API (recommended)**
   - verify transaction history and subscription status with Apple's server APIs
   - use App Store Connect API keys
   - handle `sandbox` and `production` environments explicitly

2. **Signed transaction JWS verification**
   - verify the JWS signature and payload fields server-side
   - check `productId`, `transactionId`, `expiresDate`, and `revocationDate`

### Suggested backend flow

1. client sends `signedTransactionJws`, `productId`, and `transactionId`
2. backend verifies the JWS or calls App Store Server API
3. backend stores the entitlement for the user account
4. backend returns an app-specific access token or feature flag

### References

- [App Store Server API](https://developer.apple.com/documentation/appstoreserverapi)
- [Validating receipts with the App Store](https://developer.apple.com/documentation/storekit/validating-receipts-with-the-app-store)

## Microsoft Store

### Client responsibilities

- complete purchases through Microsoft Store / `StoreContext`
- send `licenseToken`, `collectionId`, or purchase `receipt` values to your backend
- refresh owned products with `restorePurchases`

### Server responsibilities

Validate add-on ownership with Microsoft Store collections and license APIs:

1. **Collections query**
   - verify that the signed-in user owns the requested Store product
   - use Microsoft Store authentication and collection APIs from your backend

2. **License / subscription state**
   - for subscriptions, track renewal and expiration from Microsoft responses
   - reconcile against the app's internal entitlement table

### Suggested backend flow

1. client sends `licenseToken` or purchase metadata from BridgeKit
2. backend queries Microsoft Store collections or license endpoints for the user
3. backend stores entitlement state keyed by your user account
4. backend returns an app-specific access token or feature flag

### References

- [Enable subscription add-ons for your app](https://learn.microsoft.com/en-us/windows/uwp/monetize/enable-subscription-add-ons-for-your-app)
- [Microsoft Store authentication and analytics](https://learn.microsoft.com/en-us/gaming/gdk/docs/store/commerce/microsoft-store-authentication)

## BridgeKit mapping

| BridgeKit field | Apple meaning | Microsoft meaning |
| --- | --- | --- |
| `Purchase.receipt` | signed transaction JWS | license token / extended purchase JSON |
| `Purchase.transactionId` | StoreKit transaction ID | BridgeKit-generated purchase identifier |
| `Purchase.productId` | App Store product ID | Microsoft Store product ID |
| `ReceiptValidationRequest.storefront` | `apple_app_store` | `microsoft_store` |

On-device `validateReceipt` is useful for quick entitlement checks, but server
validation remains the source of truth for account-based access control.

## Rust server helpers

BridgeKit provides optional backend helpers behind Cargo features:

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["server-apple", "server-microsoft"] }
```

### Apple (`server-apple`)

- `bridgekit::server::apple::generate_app_store_jwt` — signs a short-lived JWT
  for App Store Server API requests
- `bridgekit::server::apple::get_transaction_info` — fetches
  `signedTransactionInfo` for a transaction ID

### Microsoft (`server-microsoft`)

- `bridgekit::server::microsoft::MicrosoftCollectionsClient` — queries the
  Microsoft Store collections API for owned products

These helpers expect your backend to supply App Store Connect credentials or a
Microsoft identity bearer token. They do not replace account management or
entitlement storage in your own database.
