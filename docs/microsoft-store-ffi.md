# Microsoft Store WinRT bindings

BridgeKit includes native Microsoft Store integration points for product lookup,
purchase, restore purchases, and license validation. They are compiled only for
Windows and only when the `microsoft-store-ffi` feature is enabled.

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["microsoft-store-ffi"] }
```

With that feature enabled on Windows:

- `NativeMicrosoftStoreClient::products` calls WinRT `StoreContext.GetStoreProductsAsync`
- `NativeMicrosoftStoreClient::purchase` calls `StoreContext.RequestPurchaseAsync`
- `NativeMicrosoftStoreClient::restore_purchases` calls
  `StoreContext.GetUserCollectionAsync`
- `NativeMicrosoftStoreClient::validate_license` matches receipts and transaction
  identifiers against the user collection and app license

The implementation lives in Rust under `bridgekit::winrt_store` using the
`windows` crate. No separate C ABI package is required.

## Required Windows setup

To validate this on Windows hardware:

- create the app in Partner Center
- configure Microsoft Store product IDs
- use a signed package identity matching the Store app
- enable the Windows Store IAP capability in the app manifest
- test with a sandbox account or Store simulation where available

## Product lookup

`products` maps `MicrosoftProductRequest.product_ids` to Store IDs and queries
all supported product kinds:

- `Consumable`
- `Durable`
- `UnmanagedConsumable`
- `Subscription`

Responses are normalized into `MicrosoftProduct` values with:

- `storeId`
- `title`
- `description`
- `displayPrice`
- `currencyCode`
- `kind`
- `raw.source = "microsoft_store"`

## Purchase

`purchase` calls `RequestPurchaseAsync` for the requested Store ID and maps
`StorePurchaseStatus` to BridgeKit transaction states:

| Store status | BridgeKit state |
| --- | --- |
| `Succeeded` | `purchased` |
| `AlreadyPurchased` | `restored` |
| `NotPurchased` | `cancelled` |
| network/server errors | `failed` |

Purchase responses include `licenseToken` from `ExtendedJsonData` when Store
returns it.

## Restore purchases

`restore_purchases` enumerates the current user collection and maps each owned
product into a `MicrosoftPurchase` with `state: "restored"`.

## License validation

`validate_license` checks, in order:

1. owned products from `GetUserCollectionAsync`
2. the active app license from `GetAppLicenseAsync`

A request is considered valid when:

- `receipt` matches a stored license token, or
- `transactionId` matches a restored purchase transaction ID, or
- the active app license matches the optional `productId` and `receipt` filters

When no match is found, the client returns `isValid: false` with
`raw.verification = "not_found"`.

## Desktop Bridge note

Desktop and Tauri hosts should associate the main window `HWND` with Store
purchase UI before calling purchase APIs:

```rust
bridgekit::set_store_window_handle(hwnd);
```

BridgeKit uses WinRT `IInitializeWithWindow` so `StoreContext` can present
purchase dialogs from desktop processes.

If the Tauri app uses the Desktop Bridge, the host app may also need additional
`StoreContext` initialization for the current user. See Microsoft's
[StoreContext guidance for desktop apps](https://learn.microsoft.com/en-us/windows/uwp/monetize/in-app-purchases-and-trials#use-the-storecontext-class-in-desktop-apps).

## Subscription metadata

Subscription products map `StoreSku.SubscriptionInfo` into:

- `subscriptionPeriod` as an ISO 8601 duration such as `P1M`
- `trialPeriod` when the SKU exposes a trial
