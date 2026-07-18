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
