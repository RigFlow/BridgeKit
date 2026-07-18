# Hardware test checklist

Use this checklist when validating BridgeKit on real Apple and Windows hardware.
Mock providers and unit tests cover contracts, but Store and push flows require
signed builds and platform accounts.

## Apple (macOS / iOS)

### Prerequisites

- [ ] App Store Connect app record with matching bundle identifier
- [ ] In-App Purchase products configured and cleared for sandbox testing
- [ ] Push Notifications capability enabled
- [ ] Signed build with development or distribution certificate
- [ ] Sandbox Apple ID for purchase testing
- [ ] `native/apple/BridgeKitStoreKit` linked into the Apple Tauri target
- [ ] `bridgekit::bootstrap_apple_runtime()` called during app startup

### StoreKit

- [ ] Product lookup returns configured product IDs
- [ ] Purchase completes and returns `state: purchased`
- [ ] Cancelled purchase returns `state: cancelled`
- [ ] Restore purchases returns active entitlements
- [ ] On-device `validateReceipt` matches a known transaction

### APNs

- [ ] Push authorization returns `authorized` or `provisional`
- [ ] App delegate forwards device token to `bridgekit_apns_forward_device_token_hex`
- [ ] `register_push` returns a lowercase hex device token
- [ ] Backend can deliver a test notification to the device token

## Windows

### Prerequisites

- [ ] Partner Center app with matching package identity
- [ ] Microsoft Store product IDs configured
- [ ] Signed package with Store IAP capability
- [ ] Sandbox or test account for purchase validation
- [ ] `bridgekit::set_store_window_handle(hwnd)` called during startup
- [ ] `bridgekit::set_store_context_for_current_windows_user()` called for Desktop Bridge hosts

### Microsoft Store

- [ ] Product lookup returns configured Store IDs
- [ ] Purchase dialog appears from the main window
- [ ] Successful purchase returns `state: purchased`
- [ ] Restore purchases returns owned add-ons
- [ ] On-device `validateReceipt` matches license token or transaction ID
- [ ] Subscription products include `subscriptionPeriod` and `trialPeriod`

### WNS

- [ ] Push authorization reflects `ToastNotificationManager.Setting`
- [ ] `register_push` returns a channel URI and `expiresAtMs`
- [ ] Repeated registration reuses the cached channel before expiry
- [ ] Channel refresh occurs when expiry is within 24 hours
- [ ] Backend can deliver a test toast to the channel URI

## Server-side validation

### Apple App Store Server API

- [ ] App Store Connect API key (.p8) configured on backend
- [ ] `bridgekit::server::apple::generate_app_store_jwt` produces a valid token
- [ ] `get_transaction_info` returns `signedTransactionInfo` for a sandbox transaction

### Microsoft collections API

- [ ] Azure AD / Microsoft identity token available on backend
- [ ] `MicrosoftCollectionsClient::query_user_collections` returns owned products
- [ ] Backend entitlement table updates from collections results

## Example apps

- [ ] `examples/mock-tauri-app` runs on Linux/macOS/Windows with mock providers
- [ ] `examples/apple-tauri-app` builds on macOS with Swift package linking
- [ ] `examples/windows-tauri-app` builds on Windows with WinRT providers

## Regression checks before release

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `npm run typecheck` in `packages/bridgekit-js`
- [ ] Example app smoke test on target hardware
