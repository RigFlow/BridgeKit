use crate::apple::{ApplePushClient, ApplePushProvider, AppleStoreKitClient, AppleStoreProvider};
use crate::iap::{
    Product, ProductRequest, Purchase, PurchaseRequest, ReceiptValidationRequest,
    ReceiptValidationResult, StoreProvider,
};
use crate::microsoft::{
    MicrosoftPushClient, MicrosoftPushProvider, MicrosoftStoreClient, MicrosoftStoreProvider,
};
use crate::push::{
    PushAuthorization, PushAuthorizationRequest, PushProvider, PushRegistration,
    PushRegistrationRequest,
};
use crate::unsupported::{UnsupportedPushProvider, UnsupportedStoreProvider};
use crate::{Platform, Result};
use std::sync::Arc;

#[derive(Clone)]
pub struct BridgeKit {
    store: Arc<dyn StoreProvider>,
    push: Arc<dyn PushProvider>,
}

impl BridgeKit {
    #[must_use]
    pub fn builder() -> BridgeKitBuilder {
        BridgeKitBuilder::default()
    }

    #[must_use]
    pub fn native() -> Self {
        Self::builder().build()
    }

    #[must_use]
    pub fn apple(storekit: Arc<dyn AppleStoreKitClient>, apns: Arc<dyn ApplePushClient>) -> Self {
        Self::with_providers(
            Arc::new(AppleStoreProvider::new(storekit)),
            Arc::new(ApplePushProvider::new(apns)),
        )
    }

    #[must_use]
    pub fn microsoft(
        store: Arc<dyn MicrosoftStoreClient>,
        wns: Arc<dyn MicrosoftPushClient>,
    ) -> Self {
        Self::with_providers(
            Arc::new(MicrosoftStoreProvider::new(store)),
            Arc::new(MicrosoftPushProvider::new(wns)),
        )
    }

    #[must_use]
    pub fn with_providers(store: Arc<dyn StoreProvider>, push: Arc<dyn PushProvider>) -> Self {
        Self { store, push }
    }

    pub async fn products(&self, request: ProductRequest) -> Result<Vec<Product>> {
        self.store.products(request).await
    }

    pub async fn purchase(&self, request: PurchaseRequest) -> Result<Purchase> {
        self.store.purchase(request).await
    }

    pub async fn restore_purchases(&self) -> Result<Vec<Purchase>> {
        self.store.restore_purchases().await
    }

    pub async fn validate_receipt(
        &self,
        request: ReceiptValidationRequest,
    ) -> Result<ReceiptValidationResult> {
        self.store.validate_receipt(request).await
    }

    pub async fn request_push_authorization(
        &self,
        request: PushAuthorizationRequest,
    ) -> Result<PushAuthorization> {
        self.push.request_authorization(request).await
    }

    pub async fn register_push(
        &self,
        request: PushRegistrationRequest,
    ) -> Result<PushRegistration> {
        self.push.register(request).await
    }

    pub async fn unregister_push(&self) -> Result<()> {
        self.push.unregister().await
    }
}

#[derive(Default)]
pub struct BridgeKitBuilder {
    store: Option<Arc<dyn StoreProvider>>,
    push: Option<Arc<dyn PushProvider>>,
}

impl BridgeKitBuilder {
    #[must_use]
    pub fn store_provider(mut self, provider: Arc<dyn StoreProvider>) -> Self {
        self.store = Some(provider);
        self
    }

    #[must_use]
    pub fn push_provider(mut self, provider: Arc<dyn PushProvider>) -> Self {
        self.push = Some(provider);
        self
    }

    #[must_use]
    pub fn build(self) -> BridgeKit {
        let platform = Platform::current();
        BridgeKit {
            store: self
                .store
                .unwrap_or_else(|| Arc::new(UnsupportedStoreProvider::new(platform))),
            push: self
                .push
                .unwrap_or_else(|| Arc::new(UnsupportedPushProvider::new(platform))),
        }
    }
}
