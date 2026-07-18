# Microsoft WNS WinRT bindings

BridgeKit includes native Windows Push Notification Services integration points
for authorization, registration, and unregister. They are compiled only for
Windows and only when the `microsoft-wns-ffi` feature is enabled.

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["microsoft-wns-ffi"] }
```

With that feature enabled on Windows:

- `NativeMicrosoftPushClient::request_authorization` returns the current Windows
  notification capability posture
- `NativeMicrosoftPushClient::register` calls
  `PushNotificationChannelManager.CreatePushNotificationChannelForApplicationAsync`
- `NativeMicrosoftPushClient::unregister` clears local registration state

The implementation lives in Rust under `bridgekit::winrt_wns` using the `windows`
crate.

## Required Windows setup

To validate this on Windows hardware:

- enable toast/push notification capabilities in the app manifest
- configure package identity for the Microsoft Store app
- configure WNS credentials in Partner Center for server-side delivery
- test on a signed Store build or sideloaded package with the correct identity

## Authorization

Windows does not expose an iOS-style runtime permission prompt for WNS. The
current implementation returns `status: "authorized"` and records the requested
alert/badge/sound options in `metadata`, along with a note that Windows relies
on manifest capabilities.

## Registration

`register` creates a WNS channel and returns:

- `channelUri` as the push token BridgeKit exposes through `PushRegistration.token`
- optional `environment`
- optional `expiresAtMs` from the channel expiration time
- `raw.source = "wns"`

## Unregister

Windows does not provide a full remote unregister equivalent for every app type.
BridgeKit's unregister call is currently a local no-op that keeps the API
symmetric with Apple. Host apps can add their own channel invalidation logic if
needed.

## Server-side delivery

BridgeKit only handles client-side channel creation. Sending notifications still
requires your backend to call WNS with the channel URI and your app's WNS
credentials from Partner Center.
