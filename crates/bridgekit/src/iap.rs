use crate::{Result, Storefront};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRequest {
    pub product_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storefront: Option<Storefront>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_user_id: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl ProductRequest {
    #[must_use]
    pub fn new(product_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            product_ids: product_ids.into_iter().map(Into::into).collect(),
            storefront: None,
            app_user_id: None,
            metadata: serde_json::Value::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: String,
    pub currency_code: String,
    pub storefront: Storefront,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription: Option<SubscriptionInfo>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionInfo {
    /// ISO 8601 duration such as `P1M` or `P1Y`.
    pub period: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introductory_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial_period: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequest {
    pub product_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_account_token: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl PurchaseRequest {
    #[must_use]
    pub fn new(product_id: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            quantity: None,
            app_account_token: None,
            metadata: serde_json::Value::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionState {
    Pending,
    Purchased,
    Restored,
    Failed,
    Deferred,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Purchase {
    pub transaction_id: String,
    pub product_id: String,
    pub state: TransactionState,
    pub storefront: Storefront,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_transaction_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchased_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptValidationRequest {
    pub receipt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    pub storefront: Storefront,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptValidationResult {
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

#[async_trait]
pub trait StoreProvider: Send + Sync {
    async fn products(&self, request: ProductRequest) -> Result<Vec<Product>>;

    async fn purchase(&self, request: PurchaseRequest) -> Result<Purchase>;

    async fn restore_purchases(&self) -> Result<Vec<Purchase>>;

    async fn validate_receipt(
        &self,
        request: ReceiptValidationRequest,
    ) -> Result<ReceiptValidationResult>;
}
