//! BridgeKit is a small Rust bridge layer for Tauri apps that need store
//! purchases and push notifications on Apple and Microsoft distribution
//! channels.
//!
//! The crate keeps the API surface platform-neutral so Tauri commands can
//! serialize the same request/response types on macOS/iOS and Windows. Native
//! providers can be linked behind the [`StoreProvider`] and [`PushProvider`]
//! traits, while tests and development builds can use the mock providers.

pub mod apple;
mod bridge;
mod error;
pub mod iap;
pub mod microsoft;
pub mod mock;
mod platform;
pub mod push;
#[cfg(any(feature = "server-apple", feature = "server-microsoft"))]
pub mod server;
mod unsupported;
#[cfg(windows)]
mod winrt_store;
#[cfg(windows)]
mod winrt_wns;

pub use bridge::{BridgeKit, BridgeKitBuilder};
pub use error::{BridgeKitError, Result};
pub use iap::{
    Product, ProductRequest, Purchase, PurchaseRequest, ReceiptValidationRequest,
    ReceiptValidationResult, StoreProvider, SubscriptionInfo, TransactionState,
};
pub use platform::{Capability, Platform, Storefront};
pub use push::{
    NotificationPayload, PushAuthorization, PushAuthorizationRequest, PushAuthorizationStatus,
    PushProvider, PushRegistration, PushRegistrationRequest,
};

/// Associates a Win32 owner window with Microsoft Store purchase UI.
///
/// Desktop and Tauri apps should call this during startup before making Store
/// requests. Pass the main webview `HWND` as an `isize`.
#[cfg(windows)]
pub fn set_store_window_handle(hwnd: isize) {
    winrt_store::set_store_window_handle(hwnd);
}

/// Resolves the current Windows user and uses `StoreContext::GetForUser` for
/// subsequent Microsoft Store requests.
///
/// Desktop Bridge and multi-user hosts should call this during startup in
/// addition to [`set_store_window_handle`].
#[cfg(windows)]
pub fn set_store_context_for_current_windows_user() -> Result<()> {
    winrt_store::set_store_context_for_current_windows_user()
}

/// Installs the BridgeKit Apple app delegate hooks required for APNs token
/// forwarding.
///
/// Tauri Apple apps should call this during startup before registering for push
/// notifications.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn bootstrap_apple_runtime() {
    unsafe extern "C" {
        fn bridgekit_apple_bootstrap();
    }

    unsafe {
        bridgekit_apple_bootstrap();
    }
}
