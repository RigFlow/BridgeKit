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

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod native {
    use super::*;
    #[cfg(feature = "apple-storekit-ffi")]
    use std::ffi::{CStr, CString};
    #[cfg(feature = "apple-storekit-ffi")]
    use std::os::raw::c_char;

    #[derive(Debug, Clone, Default)]
    pub struct NativeAppleStoreKitClient;

    impl NativeAppleStoreKitClient {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl AppleStoreKitClient for NativeAppleStoreKitClient {
        async fn products(&self, request: AppleProductRequest) -> Result<Vec<AppleProduct>> {
            storekit_products(request)
        }

        async fn purchase(&self, request: ApplePurchaseRequest) -> Result<AppleTransaction> {
            storekit_purchase(request)
        }

        async fn restore_purchases(&self) -> Result<Vec<AppleTransaction>> {
            storekit_restore_purchases()
        }

        async fn validate_receipt(
            &self,
            _request: AppleReceiptValidationRequest,
        ) -> Result<AppleReceiptValidationResult> {
            unavailable("App Store receipt validation")
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct NativeApplePushClient;

    impl NativeApplePushClient {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl ApplePushClient for NativeApplePushClient {
        async fn request_authorization(
            &self,
            _request: ApplePushAuthorizationRequest,
        ) -> Result<ApplePushAuthorization> {
            unavailable("APNs notification authorization")
        }

        async fn register(
            &self,
            _request: ApplePushRegistrationRequest,
        ) -> Result<ApplePushRegistration> {
            unavailable("APNs device token registration")
        }

        async fn unregister(&self) -> Result<()> {
            unavailable("APNs unregister")
        }
    }

    fn unavailable<T>(operation: &str) -> Result<T> {
        Err(BridgeKitError::ProviderUnavailable(format!(
            "{operation} native Apple bindings are not implemented yet"
        )))
    }

    #[cfg(not(feature = "apple-storekit-ffi"))]
    fn storekit_products(_request: AppleProductRequest) -> Result<Vec<AppleProduct>> {
        Err(BridgeKitError::ProviderUnavailable(
            "StoreKit product lookup native Apple bindings are not enabled; enable the \
             `apple-storekit-ffi` feature and provide the BridgeKit StoreKit FFI symbols"
                .into(),
        ))
    }

    #[cfg(feature = "apple-storekit-ffi")]
    fn storekit_products(request: AppleProductRequest) -> Result<Vec<AppleProduct>> {
        let request_json = serde_json::to_string(&request)?;
        let request_json = CString::new(request_json).map_err(|_| {
            BridgeKitError::InvalidRequest(
                "StoreKit product lookup request contained an interior NUL byte".into(),
            )
        })?;

        let response = unsafe { bridgekit_storekit_products_json(request_json.as_ptr()) };
        if response.is_null() {
            return Err(BridgeKitError::ProviderUnavailable(
                "StoreKit product lookup native bridge returned a null response".into(),
            ));
        }

        let response_json = unsafe {
            let response_json = CStr::from_ptr(response)
                .to_str()
                .map(str::to_owned)
                .map_err(|error| BridgeKitError::Native(error.to_string()));
            bridgekit_string_free(response);
            response_json
        }?;

        parse_storekit_products_json(&response_json)
    }

    #[cfg(feature = "apple-storekit-ffi")]
    fn parse_storekit_products_json(response_json: &str) -> Result<Vec<AppleProduct>> {
        serde_json::from_str(response_json).map_err(Into::into)
    }

    #[cfg(not(feature = "apple-storekit-ffi"))]
    fn storekit_purchase(_request: ApplePurchaseRequest) -> Result<AppleTransaction> {
        Err(BridgeKitError::ProviderUnavailable(
            "StoreKit purchase native Apple bindings are not enabled; enable the \
             `apple-storekit-ffi` feature and provide the BridgeKit StoreKit FFI symbols"
                .into(),
        ))
    }

    #[cfg(feature = "apple-storekit-ffi")]
    fn storekit_purchase(request: ApplePurchaseRequest) -> Result<AppleTransaction> {
        let request_json = serde_json::to_string(&request)?;
        let request_json = CString::new(request_json).map_err(|_| {
            BridgeKitError::InvalidRequest(
                "StoreKit purchase request contained an interior NUL byte".into(),
            )
        })?;

        let response = unsafe { bridgekit_storekit_purchase_json(request_json.as_ptr()) };
        if response.is_null() {
            return Err(BridgeKitError::ProviderUnavailable(
                "StoreKit purchase native bridge returned a null response".into(),
            ));
        }

        let response_json = unsafe {
            let response_json = CStr::from_ptr(response)
                .to_str()
                .map(str::to_owned)
                .map_err(|error| BridgeKitError::Native(error.to_string()));
            bridgekit_string_free(response);
            response_json
        }?;

        parse_storekit_transaction_json(&response_json)
    }

    #[cfg(feature = "apple-storekit-ffi")]
    fn parse_storekit_transaction_json(response_json: &str) -> Result<AppleTransaction> {
        serde_json::from_str(response_json).map_err(Into::into)
    }

    #[cfg(not(feature = "apple-storekit-ffi"))]
    fn storekit_restore_purchases() -> Result<Vec<AppleTransaction>> {
        Err(BridgeKitError::ProviderUnavailable(
            "StoreKit restore purchases native Apple bindings are not enabled; enable the \
             `apple-storekit-ffi` feature and provide the BridgeKit StoreKit FFI symbols"
                .into(),
        ))
    }

    #[cfg(feature = "apple-storekit-ffi")]
    fn storekit_restore_purchases() -> Result<Vec<AppleTransaction>> {
        let response = unsafe { bridgekit_storekit_restore_purchases_json() };
        if response.is_null() {
            return Err(BridgeKitError::ProviderUnavailable(
                "StoreKit restore purchases native bridge returned a null response".into(),
            ));
        }

        let response_json = unsafe {
            let response_json = CStr::from_ptr(response)
                .to_str()
                .map(str::to_owned)
                .map_err(|error| BridgeKitError::Native(error.to_string()));
            bridgekit_string_free(response);
            response_json
        }?;

        parse_storekit_transactions_json(&response_json)
    }

    #[cfg(feature = "apple-storekit-ffi")]
    fn parse_storekit_transactions_json(response_json: &str) -> Result<Vec<AppleTransaction>> {
        serde_json::from_str(response_json).map_err(Into::into)
    }

    #[cfg(feature = "apple-storekit-ffi")]
    unsafe extern "C" {
        fn bridgekit_storekit_products_json(request_json: *const c_char) -> *mut c_char;
        fn bridgekit_storekit_purchase_json(request_json: *const c_char) -> *mut c_char;
        fn bridgekit_storekit_restore_purchases_json() -> *mut c_char;
        fn bridgekit_string_free(value: *mut c_char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_product_json_matches_native_ffi_contract() {
        let json = r#"[
          {
            "id": "pro.monthly",
            "localizedTitle": "Pro Monthly",
            "localizedDescription": "Monthly Pro access",
            "displayPrice": "$4.99",
            "currencyCode": "USD",
            "subscriptionPeriod": "P1M",
            "trialPeriod": "P7D",
            "raw": {
              "source": "storekit"
            }
          }
        ]"#;

        let products: Vec<AppleProduct> =
            serde_json::from_str(json).expect("native StoreKit JSON should parse");
        assert_eq!(products[0].id, "pro.monthly");
        assert_eq!(products[0].localized_title, "Pro Monthly");
        assert_eq!(products[0].subscription_period.as_deref(), Some("P1M"));
    }

    #[test]
    fn apple_transaction_json_matches_native_ffi_contract() {
        let json = r#"{
          "transactionId": "100000000000001",
          "productId": "pro.monthly",
          "state": "purchased",
          "signedTransactionJws": "signed-transaction-jws",
          "originalTransactionId": "100000000000000",
          "purchasedAtMs": 1700000000000,
          "expiresAtMs": 1702592000000,
          "raw": {
            "source": "storekit",
            "verification": "verified"
          }
        }"#;

        let transaction: AppleTransaction =
            serde_json::from_str(json).expect("native StoreKit transaction JSON should parse");
        assert_eq!(transaction.transaction_id, "100000000000001");
        assert_eq!(transaction.product_id, "pro.monthly");
        assert_eq!(transaction.state, AppleTransactionState::Purchased);
        assert_eq!(
            transaction.signed_transaction_jws.as_deref(),
            Some("signed-transaction-jws")
        );
    }

    #[test]
    fn apple_restored_transactions_json_matches_native_ffi_contract() {
        let json = r#"[
          {
            "transactionId": "100000000000002",
            "productId": "pro.monthly",
            "state": "restored",
            "signedTransactionJws": "signed-transaction-jws",
            "originalTransactionId": "100000000000000",
            "purchasedAtMs": 1700000000000,
            "expiresAtMs": 1702592000000,
            "raw": {
              "source": "storekit",
              "verification": "verified"
            }
          }
        ]"#;

        let transactions: Vec<AppleTransaction> =
            serde_json::from_str(json).expect("native StoreKit restore JSON should parse");
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_id, "100000000000002");
        assert_eq!(transactions[0].state, AppleTransactionState::Restored);
    }
}
