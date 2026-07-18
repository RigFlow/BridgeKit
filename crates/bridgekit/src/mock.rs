use crate::iap::{
    Product, ProductRequest, Purchase, PurchaseRequest, ReceiptValidationRequest,
    ReceiptValidationResult, StoreProvider, TransactionState,
};
use crate::push::{
    PushAuthorization, PushAuthorizationRequest, PushAuthorizationStatus, PushProvider,
    PushRegistration, PushRegistrationRequest,
};
use crate::{BridgeKitError, Platform, Result, Storefront};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MockStoreProvider {
    products: Vec<Product>,
    purchases: Arc<Mutex<Vec<Purchase>>>,
    storefront: Storefront,
}

impl MockStoreProvider {
    #[must_use]
    pub fn new(products: Vec<Product>, storefront: Storefront) -> Self {
        Self {
            products,
            purchases: Arc::new(Mutex::new(Vec::new())),
            storefront,
        }
    }

    #[must_use]
    pub fn apple(products: Vec<Product>) -> Self {
        Self::new(products, Storefront::AppleAppStore)
    }

    #[must_use]
    pub fn microsoft(products: Vec<Product>) -> Self {
        Self::new(products, Storefront::MicrosoftStore)
    }

    pub fn record_purchase(&self, purchase: Purchase) -> Result<()> {
        let mut purchases = self
            .purchases
            .lock()
            .map_err(|_| BridgeKitError::ProviderUnavailable("mock store lock poisoned".into()))?;
        purchases.push(purchase);
        Ok(())
    }
}

#[async_trait]
impl StoreProvider for MockStoreProvider {
    async fn products(&self, request: ProductRequest) -> Result<Vec<Product>> {
        let requested = request.product_ids;
        let products = self
            .products
            .iter()
            .filter(|product| requested.is_empty() || requested.contains(&product.id))
            .cloned()
            .collect();
        Ok(products)
    }

    async fn purchase(&self, request: PurchaseRequest) -> Result<Purchase> {
        if !self
            .products
            .iter()
            .any(|product| product.id == request.product_id)
        {
            return Err(BridgeKitError::InvalidRequest(format!(
                "unknown product id `{}`",
                request.product_id
            )));
        }

        let purchase = Purchase {
            transaction_id: format!("mock-{}", request.product_id),
            product_id: request.product_id,
            state: TransactionState::Purchased,
            storefront: self.storefront,
            receipt: Some("mock-receipt".into()),
            original_transaction_id: None,
            purchased_at_ms: None,
            expires_at_ms: None,
            metadata: request.metadata,
        };
        self.record_purchase(purchase.clone())?;
        Ok(purchase)
    }

    async fn restore_purchases(&self) -> Result<Vec<Purchase>> {
        let purchases = self
            .purchases
            .lock()
            .map_err(|_| BridgeKitError::ProviderUnavailable("mock store lock poisoned".into()))?;
        Ok(purchases.clone())
    }

    async fn validate_receipt(
        &self,
        request: ReceiptValidationRequest,
    ) -> Result<ReceiptValidationResult> {
        Ok(ReceiptValidationResult {
            is_valid: request.receipt == "mock-receipt",
            product_id: request.product_id,
            transaction_id: request.transaction_id,
            expires_at_ms: None,
            raw: serde_json::json!({
                "storefront": request.storefront,
                "mock": true
            }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct MockPushProvider {
    platform: Platform,
    token: String,
    authorization_status: PushAuthorizationStatus,
}

impl MockPushProvider {
    #[must_use]
    pub fn new(platform: Platform, token: impl Into<String>) -> Self {
        Self {
            platform,
            token: token.into(),
            authorization_status: PushAuthorizationStatus::Authorized,
        }
    }

    #[must_use]
    pub fn with_authorization_status(mut self, status: PushAuthorizationStatus) -> Self {
        self.authorization_status = status;
        self
    }
}

#[async_trait]
impl PushProvider for MockPushProvider {
    async fn request_authorization(
        &self,
        request: PushAuthorizationRequest,
    ) -> Result<PushAuthorization> {
        Ok(PushAuthorization {
            status: self.authorization_status.clone(),
            platform: self.platform,
            metadata: request.metadata,
        })
    }

    async fn register(&self, request: PushRegistrationRequest) -> Result<PushRegistration> {
        Ok(PushRegistration {
            token: self.token.clone(),
            platform: self.platform,
            environment: request.environment,
            expires_at_ms: None,
            metadata: request.metadata,
        })
    }

    async fn unregister(&self) -> Result<()> {
        Ok(())
    }
}
