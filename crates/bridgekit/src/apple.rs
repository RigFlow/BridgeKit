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
pub enum AppleEnvironment {
    Sandbox,
    Production,
}

impl AppleEnvironment {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sandbox => "sandbox",
            Self::Production => "production",
        }
    }
}

impl TryFrom<&str> for AppleEnvironment {
    type Error = BridgeKitError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "sandbox" | "Sandbox" | "SANDBOX" => Ok(Self::Sandbox),
            "production" | "Production" | "PRODUCTION" => Ok(Self::Production),
            other => Err(BridgeKitError::InvalidRequest(format!(
                "unknown Apple environment `{other}`"
            ))),
        }
    }
}

impl std::fmt::Display for AppleEnvironment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleProductRequest {
    pub product_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_account_token: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<ProductRequest> for AppleProductRequest {
    fn from(request: ProductRequest) -> Self {
        Self {
            product_ids: request.product_ids,
            app_account_token: request.app_user_id,
            metadata: request.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleProduct {
    pub id: String,
    pub localized_title: String,
    pub localized_description: String,
    pub display_price: String,
    pub currency_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_period: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introductory_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial_period: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

impl From<AppleProduct> for Product {
    fn from(product: AppleProduct) -> Self {
        let subscription = product.subscription_period.map(|period| SubscriptionInfo {
            period,
            introductory_price: product.introductory_price,
            trial_period: product.trial_period,
        });

        Self {
            id: product.id,
            title: product.localized_title,
            description: product.localized_description,
            price: product.display_price,
            currency_code: product.currency_code,
            storefront: Storefront::AppleAppStore,
            subscription,
            metadata: product.raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplePurchaseRequest {
    pub product_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_account_token: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<PurchaseRequest> for ApplePurchaseRequest {
    fn from(request: PurchaseRequest) -> Self {
        Self {
            product_id: request.product_id,
            quantity: request.quantity,
            app_account_token: request.app_account_token,
            metadata: request.metadata,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppleTransactionState {
    Purchasing,
    Purchased,
    Restored,
    Failed,
    Deferred,
    Cancelled,
}

impl From<AppleTransactionState> for TransactionState {
    fn from(state: AppleTransactionState) -> Self {
        match state {
            AppleTransactionState::Purchasing => Self::Pending,
            AppleTransactionState::Purchased => Self::Purchased,
            AppleTransactionState::Restored => Self::Restored,
            AppleTransactionState::Failed => Self::Failed,
            AppleTransactionState::Deferred => Self::Deferred,
            AppleTransactionState::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleTransaction {
    pub transaction_id: String,
    pub product_id: String,
    pub state: AppleTransactionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_transaction_jws: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_store_receipt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_transaction_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchased_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

impl From<AppleTransaction> for Purchase {
    fn from(transaction: AppleTransaction) -> Self {
        Self {
            transaction_id: transaction.transaction_id,
            product_id: transaction.product_id,
            state: transaction.state.into(),
            storefront: Storefront::AppleAppStore,
            receipt: transaction
                .signed_transaction_jws
                .or(transaction.app_store_receipt),
            original_transaction_id: transaction.original_transaction_id,
            purchased_at_ms: transaction.purchased_at_ms,
            expires_at_ms: transaction.expires_at_ms,
            metadata: transaction.raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleReceiptValidationRequest {
    pub receipt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<AppleEnvironment>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleReceiptValidationResult {
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

impl From<AppleReceiptValidationResult> for ReceiptValidationResult {
    fn from(result: AppleReceiptValidationResult) -> Self {
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
pub trait AppleStoreKitClient: Send + Sync {
    async fn products(&self, request: AppleProductRequest) -> Result<Vec<AppleProduct>>;

    async fn purchase(&self, request: ApplePurchaseRequest) -> Result<AppleTransaction>;

    async fn restore_purchases(&self) -> Result<Vec<AppleTransaction>>;

    async fn validate_receipt(
        &self,
        request: AppleReceiptValidationRequest,
    ) -> Result<AppleReceiptValidationResult>;
}

#[derive(Clone)]
pub struct AppleStoreProvider {
    client: Arc<dyn AppleStoreKitClient>,
    environment: Option<AppleEnvironment>,
}

impl AppleStoreProvider {
    #[must_use]
    pub fn new(client: Arc<dyn AppleStoreKitClient>) -> Self {
        Self {
            client,
            environment: None,
        }
    }

    #[must_use]
    pub fn with_environment(mut self, environment: AppleEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }
}

#[async_trait]
impl StoreProvider for AppleStoreProvider {
    async fn products(&self, request: ProductRequest) -> Result<Vec<Product>> {
        if request.product_ids.is_empty() {
            return Err(BridgeKitError::InvalidRequest(
                "Apple product lookup requires at least one product id".into(),
            ));
        }

        let products = self.client.products(request.into()).await?;
        Ok(products.into_iter().map(Into::into).collect())
    }

    async fn purchase(&self, request: PurchaseRequest) -> Result<Purchase> {
        if request.product_id.is_empty() {
            return Err(BridgeKitError::InvalidRequest(
                "Apple purchase requires a product id".into(),
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
        if request.storefront != Storefront::AppleAppStore {
            return Err(BridgeKitError::InvalidRequest(
                "Apple receipt validation requires the Apple App Store storefront".into(),
            ));
        }

        self.client
            .validate_receipt(AppleReceiptValidationRequest {
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
pub struct ApplePushAuthorizationRequest {
    pub alert: bool,
    pub badge: bool,
    pub sound: bool,
    pub provisional: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<PushAuthorizationRequest> for ApplePushAuthorizationRequest {
    fn from(request: PushAuthorizationRequest) -> Self {
        Self {
            alert: request.alert,
            badge: request.badge,
            sound: request.sound,
            provisional: request.provisional,
            metadata: request.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplePushAuthorization {
    pub status: PushAuthorizationStatus,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl From<ApplePushAuthorization> for PushAuthorization {
    fn from(authorization: ApplePushAuthorization) -> Self {
        Self {
            status: authorization.status,
            platform: Platform::Apple,
            metadata: authorization.metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplePushRegistrationRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<AppleEnvironment>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplePushRegistration {
    pub device_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<AppleEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

impl From<ApplePushRegistration> for PushRegistration {
    fn from(registration: ApplePushRegistration) -> Self {
        Self {
            token: registration.device_token,
            platform: Platform::Apple,
            environment: registration
                .environment
                .map(|environment| environment.to_string()),
            expires_at_ms: registration.expires_at_ms,
            metadata: registration.raw,
        }
    }
}

#[async_trait]
pub trait ApplePushClient: Send + Sync {
    async fn request_authorization(
        &self,
        request: ApplePushAuthorizationRequest,
    ) -> Result<ApplePushAuthorization>;

    async fn register(
        &self,
        request: ApplePushRegistrationRequest,
    ) -> Result<ApplePushRegistration>;

    async fn unregister(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct ApplePushProvider {
    client: Arc<dyn ApplePushClient>,
    environment: Option<AppleEnvironment>,
}

impl ApplePushProvider {
    #[must_use]
    pub fn new(client: Arc<dyn ApplePushClient>) -> Self {
        Self {
            client,
            environment: None,
        }
    }

    #[must_use]
    pub fn with_environment(mut self, environment: AppleEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }
}

#[async_trait]
impl PushProvider for ApplePushProvider {
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
            .map(AppleEnvironment::try_from)
            .transpose()?;

        self.client
            .register(ApplePushRegistrationRequest {
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
