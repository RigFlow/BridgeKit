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
mod unsupported;

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
