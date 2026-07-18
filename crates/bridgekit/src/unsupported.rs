use crate::iap::{
    Product, ProductRequest, Purchase, PurchaseRequest, ReceiptValidationRequest,
    ReceiptValidationResult, StoreProvider,
};
use crate::push::{
    PushAuthorization, PushAuthorizationRequest, PushProvider, PushRegistration,
    PushRegistrationRequest,
};
use crate::{BridgeKitError, Capability, Platform, Result};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct UnsupportedStoreProvider {
    platform: Platform,
}

impl UnsupportedStoreProvider {
    #[must_use]
    pub const fn new(platform: Platform) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl StoreProvider for UnsupportedStoreProvider {
    async fn products(&self, _request: ProductRequest) -> Result<Vec<Product>> {
        Err(BridgeKitError::UnsupportedPlatform {
            capability: Capability::InAppPurchases,
            platform: self.platform,
        })
    }

    async fn purchase(&self, _request: PurchaseRequest) -> Result<Purchase> {
        Err(BridgeKitError::UnsupportedPlatform {
            capability: Capability::InAppPurchases,
            platform: self.platform,
        })
    }

    async fn restore_purchases(&self) -> Result<Vec<Purchase>> {
        Err(BridgeKitError::UnsupportedPlatform {
            capability: Capability::InAppPurchases,
            platform: self.platform,
        })
    }

    async fn validate_receipt(
        &self,
        _request: ReceiptValidationRequest,
    ) -> Result<ReceiptValidationResult> {
        Err(BridgeKitError::UnsupportedPlatform {
            capability: Capability::InAppPurchases,
            platform: self.platform,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UnsupportedPushProvider {
    platform: Platform,
}

impl UnsupportedPushProvider {
    #[must_use]
    pub const fn new(platform: Platform) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl PushProvider for UnsupportedPushProvider {
    async fn request_authorization(
        &self,
        _request: PushAuthorizationRequest,
    ) -> Result<PushAuthorization> {
        Err(BridgeKitError::UnsupportedPlatform {
            capability: Capability::PushNotifications,
            platform: self.platform,
        })
    }

    async fn register(&self, _request: PushRegistrationRequest) -> Result<PushRegistration> {
        Err(BridgeKitError::UnsupportedPlatform {
            capability: Capability::PushNotifications,
            platform: self.platform,
        })
    }

    async fn unregister(&self) -> Result<()> {
        Err(BridgeKitError::UnsupportedPlatform {
            capability: Capability::PushNotifications,
            platform: self.platform,
        })
    }
}
