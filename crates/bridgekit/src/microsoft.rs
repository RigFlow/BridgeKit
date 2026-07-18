use crate::iap::{
    Product, ProductRequest, Purchase, PurchaseRequest, ReceiptValidationRequest,
    ReceiptValidationResult, StoreProvider, SubscriptionInfo, TransactionState,
};
use crate::push::{
    PushAuthorization, PushAuthorizationRequest, PushAuthorizationStatus, PushProvider,
    PushRegistration, PushRegistrationRequest,
};
use crate::{BridgeKitError, Platform, Result, Storefront};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MicrosoftStoreEnvironment {
    Sandbox,
    Production,
}

impl MicrosoftStoreEnvironment {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sandbox => "sandbox",
            Self::Production => "production",
        }
    }
}

impl TryFrom<&str> for MicrosoftStoreEnvironment {
    type Error = BridgeKitError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "sandbox" | "Sandbox" | "SANDBOX" => Ok(Self::Sandbox),
            "production" | "Production" | "PRODUCTION" => Ok(Self::Production),
            other => Err(BridgeKitError::InvalidRequest(format!(
                "unknown Microsoft Store environment `{other}`"
            ))),
        }
    }
}

impl std::fmt::Display for MicrosoftStoreEnvironment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftProductRequest {
    pub product_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<ProductRequest> for MicrosoftProductRequest {
    fn from(request: ProductRequest) -> Self {
        Self {
            product_ids: request.product_ids,
            user_id: request.app_user_id,
            metadata: request.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MicrosoftProductKind {
    Consumable,
    Durable,
    Subscription,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftProduct {
    pub store_id: String,
    pub title: String,
    pub description: String,
    pub display_price: String,
    pub currency_code: String,
    pub kind: MicrosoftProductKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_period: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial_period: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

impl From<MicrosoftProduct> for Product {
    fn from(product: MicrosoftProduct) -> Self {
        let subscription = match product.kind {
            MicrosoftProductKind::Subscription => {
                product.subscription_period.map(|period| SubscriptionInfo {
                    period,
                    introductory_price: None,
                    trial_period: product.trial_period,
                })
            }
            MicrosoftProductKind::Consumable | MicrosoftProductKind::Durable => None,
        };

        Self {
            id: product.store_id,
            title: product.title,
            description: product.description,
            price: product.display_price,
            currency_code: product.currency_code,
            storefront: Storefront::MicrosoftStore,
            subscription,
            metadata: product.raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftPurchaseRequest {
    pub product_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<PurchaseRequest> for MicrosoftPurchaseRequest {
    fn from(request: PurchaseRequest) -> Self {
        Self {
            product_id: request.product_id,
            quantity: request.quantity,
            user_id: request.app_account_token,
            metadata: request.metadata,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MicrosoftTransactionState {
    Pending,
    Purchased,
    Restored,
    Failed,
    Cancelled,
}

impl From<MicrosoftTransactionState> for TransactionState {
    fn from(state: MicrosoftTransactionState) -> Self {
        match state {
            MicrosoftTransactionState::Pending => Self::Pending,
            MicrosoftTransactionState::Purchased => Self::Purchased,
            MicrosoftTransactionState::Restored => Self::Restored,
            MicrosoftTransactionState::Failed => Self::Failed,
            MicrosoftTransactionState::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftPurchase {
    pub transaction_id: String,
    pub product_id: String,
    pub state: MicrosoftTransactionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchased_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

impl From<MicrosoftPurchase> for Purchase {
    fn from(purchase: MicrosoftPurchase) -> Self {
        Self {
            transaction_id: purchase.transaction_id,
            product_id: purchase.product_id,
            state: purchase.state.into(),
            storefront: Storefront::MicrosoftStore,
            receipt: purchase.license_token.or(purchase.collection_id),
            original_transaction_id: None,
            purchased_at_ms: purchase.purchased_at_ms,
            expires_at_ms: purchase.expires_at_ms,
            metadata: purchase.raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftLicenseValidationRequest {
    pub receipt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<MicrosoftStoreEnvironment>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftLicenseValidationResult {
    pub is_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

impl From<MicrosoftLicenseValidationResult> for ReceiptValidationResult {
    fn from(result: MicrosoftLicenseValidationResult) -> Self {
        Self {
            is_valid: result.is_valid,
            product_id: result.product_id,
            transaction_id: result.transaction_id,
            expires_at_ms: result.expires_at_ms,
            raw: result.raw,
        }
    }
}

#[async_trait]
pub trait MicrosoftStoreClient: Send + Sync {
    async fn products(&self, request: MicrosoftProductRequest) -> Result<Vec<MicrosoftProduct>>;

    async fn purchase(&self, request: MicrosoftPurchaseRequest) -> Result<MicrosoftPurchase>;

    async fn restore_purchases(&self) -> Result<Vec<MicrosoftPurchase>>;

    async fn validate_license(
        &self,
        request: MicrosoftLicenseValidationRequest,
    ) -> Result<MicrosoftLicenseValidationResult>;
}

#[derive(Clone)]
pub struct MicrosoftStoreProvider {
    client: Arc<dyn MicrosoftStoreClient>,
    environment: Option<MicrosoftStoreEnvironment>,
}

impl MicrosoftStoreProvider {
    #[must_use]
    pub fn new(client: Arc<dyn MicrosoftStoreClient>) -> Self {
        Self {
            client,
            environment: None,
        }
    }

    #[must_use]
    pub fn with_environment(mut self, environment: MicrosoftStoreEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }
}

#[async_trait]
impl StoreProvider for MicrosoftStoreProvider {
    async fn products(&self, request: ProductRequest) -> Result<Vec<Product>> {
        if request.product_ids.is_empty() {
            return Err(BridgeKitError::InvalidRequest(
                "Microsoft Store product lookup requires at least one product id".into(),
            ));
        }

        let products = self.client.products(request.into()).await?;
        Ok(products.into_iter().map(Into::into).collect())
    }

    async fn purchase(&self, request: PurchaseRequest) -> Result<Purchase> {
        if request.product_id.is_empty() {
            return Err(BridgeKitError::InvalidRequest(
                "Microsoft Store purchase requires a product id".into(),
            ));
        }

        self.client.purchase(request.into()).await.map(Into::into)
    }

    async fn restore_purchases(&self) -> Result<Vec<Purchase>> {
        let purchases = self.client.restore_purchases().await?;
        Ok(purchases.into_iter().map(Into::into).collect())
    }

    async fn validate_receipt(
        &self,
        request: ReceiptValidationRequest,
    ) -> Result<ReceiptValidationResult> {
        if request.storefront != Storefront::MicrosoftStore {
            return Err(BridgeKitError::InvalidRequest(
                "Microsoft license validation requires the Microsoft Store storefront".into(),
            ));
        }

        self.client
            .validate_license(MicrosoftLicenseValidationRequest {
                receipt: request.receipt,
                transaction_id: request.transaction_id,
                product_id: request.product_id,
                environment: self.environment,
                metadata: request.metadata,
            })
            .await
            .map(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftPushAuthorizationRequest {
    pub alert: bool,
    pub badge: bool,
    pub sound: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<PushAuthorizationRequest> for MicrosoftPushAuthorizationRequest {
    fn from(request: PushAuthorizationRequest) -> Self {
        Self {
            alert: request.alert,
            badge: request.badge,
            sound: request.sound,
            metadata: request.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftPushAuthorization {
    pub status: PushAuthorizationStatus,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<MicrosoftPushAuthorization> for PushAuthorization {
    fn from(authorization: MicrosoftPushAuthorization) -> Self {
        Self {
            status: authorization.status,
            platform: Platform::Microsoft,
            metadata: authorization.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftPushRegistrationRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<MicrosoftStoreEnvironment>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftPushRegistration {
    pub channel_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<MicrosoftStoreEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

impl From<MicrosoftPushRegistration> for PushRegistration {
    fn from(registration: MicrosoftPushRegistration) -> Self {
        Self {
            token: registration.channel_uri,
            platform: Platform::Microsoft,
            environment: registration
                .environment
                .map(|environment| environment.to_string()),
            expires_at_ms: registration.expires_at_ms,
            metadata: registration.raw,
        }
    }
}

#[async_trait]
pub trait MicrosoftPushClient: Send + Sync {
    async fn request_authorization(
        &self,
        request: MicrosoftPushAuthorizationRequest,
    ) -> Result<MicrosoftPushAuthorization>;

    async fn register(
        &self,
        request: MicrosoftPushRegistrationRequest,
    ) -> Result<MicrosoftPushRegistration>;

    async fn unregister(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct MicrosoftPushProvider {
    client: Arc<dyn MicrosoftPushClient>,
    environment: Option<MicrosoftStoreEnvironment>,
}

impl MicrosoftPushProvider {
    #[must_use]
    pub fn new(client: Arc<dyn MicrosoftPushClient>) -> Self {
        Self {
            client,
            environment: None,
        }
    }

    #[must_use]
    pub fn with_environment(mut self, environment: MicrosoftStoreEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }
}

#[async_trait]
impl PushProvider for MicrosoftPushProvider {
    async fn request_authorization(
        &self,
        request: PushAuthorizationRequest,
    ) -> Result<PushAuthorization> {
        self.client
            .request_authorization(request.into())
            .await
            .map(Into::into)
    }

    async fn register(&self, request: PushRegistrationRequest) -> Result<PushRegistration> {
        let requested_environment = request
            .environment
            .as_deref()
            .map(MicrosoftStoreEnvironment::try_from)
            .transpose()?;

        self.client
            .register(MicrosoftPushRegistrationRequest {
                environment: requested_environment.or(self.environment),
                metadata: request.metadata,
            })
            .await
            .map(Into::into)
    }

    async fn unregister(&self) -> Result<()> {
        self.client.unregister().await
    }
}

#[cfg(target_os = "windows")]
pub mod native {
    use super::*;

    #[derive(Debug, Clone, Default)]
    pub struct NativeMicrosoftStoreClient;

    impl NativeMicrosoftStoreClient {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl MicrosoftStoreClient for NativeMicrosoftStoreClient {
        async fn products(&self, request: MicrosoftProductRequest) -> Result<Vec<MicrosoftProduct>> {
            store_products(request).await
        }

        async fn purchase(&self, request: MicrosoftPurchaseRequest) -> Result<MicrosoftPurchase> {
            store_purchase(request).await
        }

        async fn restore_purchases(&self) -> Result<Vec<MicrosoftPurchase>> {
            store_restore_purchases().await
        }

        async fn validate_license(
            &self,
            request: MicrosoftLicenseValidationRequest,
        ) -> Result<MicrosoftLicenseValidationResult> {
            store_validate_license(request).await
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct NativeMicrosoftPushClient;

    impl NativeMicrosoftPushClient {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl MicrosoftPushClient for NativeMicrosoftPushClient {
        async fn request_authorization(
            &self,
            request: MicrosoftPushAuthorizationRequest,
        ) -> Result<MicrosoftPushAuthorization> {
            wns_request_authorization(request).await
        }

        async fn register(
            &self,
            request: MicrosoftPushRegistrationRequest,
        ) -> Result<MicrosoftPushRegistration> {
            wns_register(request).await
        }

        async fn unregister(&self) -> Result<()> {
            wns_unregister().await
        }
    }

    #[cfg(not(feature = "microsoft-store-ffi"))]
    async fn store_products(_request: MicrosoftProductRequest) -> Result<Vec<MicrosoftProduct>> {
        Err(BridgeKitError::ProviderUnavailable(
            "Microsoft Store product lookup native bindings are not enabled; enable the \
             `microsoft-store-ffi` feature"
                .into(),
        ))
    }

    #[cfg(feature = "microsoft-store-ffi")]
    async fn store_products(request: MicrosoftProductRequest) -> Result<Vec<MicrosoftProduct>> {
        crate::winrt_store::products(request)
    }

    #[cfg(not(feature = "microsoft-store-ffi"))]
    async fn store_purchase(_request: MicrosoftPurchaseRequest) -> Result<MicrosoftPurchase> {
        Err(BridgeKitError::ProviderUnavailable(
            "Microsoft Store purchase native bindings are not enabled; enable the \
             `microsoft-store-ffi` feature"
                .into(),
        ))
    }

    #[cfg(feature = "microsoft-store-ffi")]
    async fn store_purchase(request: MicrosoftPurchaseRequest) -> Result<MicrosoftPurchase> {
        crate::winrt_store::purchase(request)
    }

    #[cfg(not(feature = "microsoft-store-ffi"))]
    async fn store_restore_purchases() -> Result<Vec<MicrosoftPurchase>> {
        Err(BridgeKitError::ProviderUnavailable(
            "Microsoft Store restore purchases native bindings are not enabled; enable the \
             `microsoft-store-ffi` feature"
                .into(),
        ))
    }

    #[cfg(feature = "microsoft-store-ffi")]
    async fn store_restore_purchases() -> Result<Vec<MicrosoftPurchase>> {
        crate::winrt_store::restore_purchases()
    }

    #[cfg(not(feature = "microsoft-store-ffi"))]
    async fn store_validate_license(
        _request: MicrosoftLicenseValidationRequest,
    ) -> Result<MicrosoftLicenseValidationResult> {
        Err(BridgeKitError::ProviderUnavailable(
            "Microsoft Store license validation native bindings are not enabled; enable the \
             `microsoft-store-ffi` feature"
                .into(),
        ))
    }

    #[cfg(feature = "microsoft-store-ffi")]
    async fn store_validate_license(
        request: MicrosoftLicenseValidationRequest,
    ) -> Result<MicrosoftLicenseValidationResult> {
        crate::winrt_store::validate_license(request)
    }

    #[cfg(not(feature = "microsoft-wns-ffi"))]
    async fn wns_request_authorization(
        _request: MicrosoftPushAuthorizationRequest,
    ) -> Result<MicrosoftPushAuthorization> {
        Err(BridgeKitError::ProviderUnavailable(
            "WNS authorization native bindings are not enabled; enable the `microsoft-wns-ffi` \
             feature"
                .into(),
        ))
    }

    #[cfg(feature = "microsoft-wns-ffi")]
    async fn wns_request_authorization(
        request: MicrosoftPushAuthorizationRequest,
    ) -> Result<MicrosoftPushAuthorization> {
        crate::winrt_wns::request_authorization(request)
    }

    #[cfg(not(feature = "microsoft-wns-ffi"))]
    async fn wns_register(
        _request: MicrosoftPushRegistrationRequest,
    ) -> Result<MicrosoftPushRegistration> {
        Err(BridgeKitError::ProviderUnavailable(
            "WNS registration native bindings are not enabled; enable the `microsoft-wns-ffi` \
             feature"
                .into(),
        ))
    }

    #[cfg(feature = "microsoft-wns-ffi")]
    async fn wns_register(
        request: MicrosoftPushRegistrationRequest,
    ) -> Result<MicrosoftPushRegistration> {
        crate::winrt_wns::register(request)
    }

    #[cfg(not(feature = "microsoft-wns-ffi"))]
    async fn wns_unregister() -> Result<()> {
        Err(BridgeKitError::ProviderUnavailable(
            "WNS unregister native bindings are not enabled; enable the `microsoft-wns-ffi` \
             feature"
                .into(),
        ))
    }

    #[cfg(feature = "microsoft-wns-ffi")]
    async fn wns_unregister() -> Result<()> {
        crate::winrt_wns::unregister()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::push::PushAuthorizationStatus;

    #[test]
    fn microsoft_product_json_matches_native_contract() {
        let json = r#"{
          "storeId": "9NBLGGH4R315",
          "title": "Pro Monthly",
          "description": "Monthly Pro access",
          "displayPrice": "$4.99",
          "currencyCode": "USD",
          "kind": "subscription",
          "subscriptionPeriod": "P1M",
          "trialPeriod": "P7D",
          "raw": {
            "source": "microsoft_store"
          }
        }"#;

        let product: MicrosoftProduct =
            serde_json::from_str(json).expect("Microsoft product JSON should parse");
        assert_eq!(product.store_id, "9NBLGGH4R315");
        assert_eq!(product.kind, MicrosoftProductKind::Subscription);
    }

    #[test]
    fn microsoft_purchase_json_matches_native_contract() {
        let json = r#"{
          "transactionId": "order-1",
          "productId": "9NBLGGH4R315",
          "state": "purchased",
          "licenseToken": "license-token",
          "collectionId": "collection-id",
          "purchasedAtMs": 1700000000000,
          "expiresAtMs": 1702592000000,
          "raw": {
            "source": "microsoft_store"
          }
        }"#;

        let purchase: MicrosoftPurchase =
            serde_json::from_str(json).expect("Microsoft purchase JSON should parse");
        assert_eq!(purchase.transaction_id, "order-1");
        assert_eq!(purchase.state, MicrosoftTransactionState::Purchased);
    }

    #[test]
    fn microsoft_license_validation_json_matches_native_contract() {
        let json = r#"{
          "isValid": true,
          "productId": "9NBLGGH4R315",
          "transactionId": "order-1",
          "expiresAtMs": 1702592000000,
          "raw": {
            "source": "microsoft_store",
            "verification": "verified"
          }
        }"#;

        let result: MicrosoftLicenseValidationResult =
            serde_json::from_str(json).expect("Microsoft license validation JSON should parse");
        assert!(result.is_valid);
        assert_eq!(result.product_id.as_deref(), Some("9NBLGGH4R315"));
    }

    #[test]
    fn microsoft_push_authorization_json_matches_native_contract() {
        let json = r#"{
          "status": "authorized",
          "metadata": {
            "source": "wns"
          }
        }"#;

        let authorization: MicrosoftPushAuthorization =
            serde_json::from_str(json).expect("Microsoft push authorization JSON should parse");
        assert_eq!(authorization.status, PushAuthorizationStatus::Authorized);
    }

    #[test]
    fn microsoft_push_registration_json_matches_native_contract() {
        let json = r#"{
          "channelUri": "https://example.test/channel",
          "environment": "production",
          "raw": {
            "source": "wns"
          }
        }"#;

        let registration: MicrosoftPushRegistration =
            serde_json::from_str(json).expect("Microsoft push registration JSON should parse");
        assert_eq!(registration.channel_uri, "https://example.test/channel");
        assert_eq!(
            registration.environment,
            Some(MicrosoftStoreEnvironment::Production)
        );
    }
}
