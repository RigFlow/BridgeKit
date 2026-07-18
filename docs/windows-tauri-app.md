# Windows Tauri example

The `examples/windows-tauri-app` project demonstrates BridgeKit against native
Microsoft Store and WNS WinRT bindings on Windows.

## Stack

- `BridgeKit::native()` with `microsoft-store-ffi` and `microsoft-wns-ffi`
- `tauri-plugin-bridgekit`
- `@bridgekit/tauri` TypeScript SDK
- `bridgekit::set_store_window_handle` for Desktop Bridge purchase UI

## Configure product IDs

Update `src/main.ts` so `PRODUCT_ID` matches a Store add-on ID from Partner
Center for the app's package identity.

## Store owner window

The example calls `bridgekit::set_store_window_handle` during Tauri setup so
`StoreContext` can present purchase UI from desktop and Tauri hosts. This uses
WinRT `IInitializeWithWindow`.

The example also calls `bridgekit::set_store_context_for_current_windows_user`
so Store requests use `StoreContext::GetForUser` for the current Windows user.
This is recommended for Desktop Bridge hosts and multi-user environments.

## WNS channel caching

BridgeKit caches the WNS channel URI and refreshes it automatically when the
channel is within 24 hours of expiry. Repeated `register_push` calls reuse the
cached channel until refresh is needed.

## Required setup

- Partner Center app record with matching package identity
- Microsoft Store product IDs configured for the app
- toast/push notification capabilities in the app manifest
- WNS credentials in Partner Center for server-side delivery
- signed Microsoft Store or sideloaded package for runtime testing

## Run

```sh
npm install
cd examples/windows-tauri-app
npm run tauri dev
```

Store and WNS flows require Windows hardware and a signed package with the
correct Store identity.

For a full validation pass, see [`hardware-test-checklist.md`](hardware-test-checklist.md).
