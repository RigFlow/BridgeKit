# Apple APNs FFI

BridgeKit includes native APNs integration points for push authorization,
registration, and unregister. They are compiled only for macOS/iOS and only when
the `apple-apns-ffi` feature is enabled.

The repository includes a Swift implementation of this ABI in
`native/apple/BridgeKitStoreKit` alongside the StoreKit FFI. See
[`apple-storekit-swift-package.md`](apple-storekit-swift-package.md) for linking
instructions.

```toml
[dependencies]
bridgekit = { git = "https://github.com/RigFlow/BridgeKit", features = ["apple-apns-ffi"] }
```

With that feature enabled:

- `NativeApplePushClient::request_authorization` serializes an
  `ApplePushAuthorizationRequest`, calls a native C ABI symbol, and parses
  `ApplePushAuthorization`.
- `NativeApplePushClient::register` serializes an `ApplePushRegistrationRequest`,
  calls a native C ABI symbol, and parses `ApplePushRegistration`.
- `NativeApplePushClient::unregister` calls a native C ABI symbol.

## Required native symbols

The consuming Apple app or native support library must export:

```c
char *bridgekit_apns_request_authorization_json(const char *request_json);
char *bridgekit_apns_register_json(const char *request_json);
void bridgekit_apns_unregister(void);
void bridgekit_apns_forward_device_token_hex(const char *token_hex);
void bridgekit_string_free(char *value);
```

Ownership contract:

- `request_json` is a borrowed UTF-8 JSON string owned by Rust.
- APNs JSON functions return newly allocated UTF-8 JSON strings.
- Rust calls `bridgekit_string_free` exactly once for non-null returned strings.
- Returning null is treated as `ProviderUnavailable`.

## Authorization request JSON

The request is `ApplePushAuthorizationRequest` with camelCase fields:

```json
{
  "alert": true,
  "badge": true,
  "sound": true,
  "provisional": false,
  "metadata": {
    "optional": true
  }
}
```

## Authorization response JSON

Return an `ApplePushAuthorization` value without `platform`; BridgeKit adds the
platform when mapping into shared push types:

```json
{
  "status": "authorized",
  "metadata": {
    "source": "apns"
  }
}
```

Supported `status` values:

- `not_determined`
- `denied`
- `authorized`
- `provisional`
- `ephemeral`
- `unsupported`

## Registration request JSON

The request is `ApplePushRegistrationRequest` with camelCase fields:

```json
{
  "environment": "production",
  "metadata": {
    "optional": true
  }
}
```

## Registration response JSON

Return an `ApplePushRegistration` value:

```json
{
  "deviceToken": "apns-device-token",
  "environment": "production",
  "expiresAtMs": null,
  "raw": {
    "source": "apns"
  }
}
```

## Device token forwarding

`bridgekit_apns_register_json` calls `registerForRemoteNotifications` and waits
for a device token. The consuming app must forward the token from its app
delegate:

```swift
func application(
    _ application: UIApplication,
    didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
) {
    let tokenHex = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
    bridgekitApnsForwardDeviceTokenHex(tokenHex)
}
```

On macOS, forward the token from the equivalent
`NSApplicationDelegate` callback.

## APNs implementation notes

Authorization should use `UNUserNotificationCenter.requestAuthorization`.

Registration should call:

- `UIApplication.registerForRemoteNotifications()` on iOS
- `NSApplication.registerForRemoteNotifications()` on macOS

Unregister should call `UIApplication.unregisterForRemoteNotifications()` on iOS
and clear any cached token state.

The exported C ABI is synchronous from Rust's perspective. If the Swift
implementation performs async notification work internally, ensure it does not
block the app's main actor indefinitely.

## Apple Developer setup

To validate this on Apple hardware:

- enable Push Notifications for the app identifier
- add the Push Notifications capability to the consuming Tauri/Xcode target
- configure APNs keys/certificates in Apple Developer
- forward device tokens from the app delegate into
  `bridgekit_apns_forward_device_token_hex`
