use async_trait::async_trait;
use bridgekit::apple::{
    AppleEnvironment, AppleProduct, AppleProductRequest, ApplePurchaseRequest,
    ApplePushAuthorization, ApplePushAuthorizationRequest, ApplePushClient, ApplePushRegistration,
    ApplePushRegistrationRequest, AppleReceiptValidationRequest, AppleReceiptValidationResult,
    AppleStoreKitClient, AppleTransaction, AppleTransactionState,
};
use bridgekit::mock::{MockPushProvider, MockStoreProvider};
use bridgekit::{
    BridgeKit, BridgeKitError, Capability, Platform, Product, ProductRequest, PurchaseRequest,
    PushAuthorizationRequest, PushAuthorizationStatus, PushRegistrationRequest,
    ReceiptValidationRequest, Result, Storefront,
};
use futures::executor::block_on;
use std::sync::Arc;

#[test]
fn mock_store_can_purchase_and_restore() {
    block_on(async {
        let bridge = BridgeKit::with_providers(
            Arc::new(MockStoreProvider::apple(vec![sample_product(
                "pro.monthly",
                Storefront::AppleAppStore,
            )])),
            Arc::new(MockPushProvider::new(Platform::Apple, "apns-token")),
        );

        let products = bridge
            .products(ProductRequest::new(["pro.monthly"]))
            .await
            .expect("products should load");
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].id, "pro.monthly");

        let purchase = bridge
            .purchase(PurchaseRequest::new("pro.monthly"))
            .await
            .expect("purchase should succeed");
        assert_eq!(purchase.transaction_id, "mock-pro.monthly");

        let restored = bridge
            .restore_purchases()
            .await
            .expect("restore should return mock purchases");
        assert_eq!(restored, vec![purchase]);
    });
}

#[test]
fn mock_push_can_authorize_and_register() {
    block_on(async {
        let bridge = BridgeKit::with_providers(
            Arc::new(MockStoreProvider::microsoft(vec![sample_product(
                "credits.100",
                Storefront::MicrosoftStore,
            )])),
            Arc::new(MockPushProvider::new(
                Platform::Microsoft,
                "wns-channel-uri",
            )),
        );

        let authorization = bridge
            .request_push_authorization(PushAuthorizationRequest::default())
            .await
            .expect("authorization should succeed");
        assert_eq!(authorization.platform, Platform::Microsoft);

        let registration = bridge
            .register_push(PushRegistrationRequest::default())
            .await
            .expect("registration should succeed");
        assert_eq!(registration.token, "wns-channel-uri");
        assert_eq!(registration.platform, Platform::Microsoft);
    });
}

#[test]
fn receipt_validation_result_uses_camel_case_json() {
    let request = ReceiptValidationRequest {
        receipt: "mock-receipt".into(),
        transaction_id: Some("transaction-1".into()),
        product_id: Some("pro.monthly".into()),
        storefront: Storefront::AppleAppStore,
        metadata: serde_json::Value::Null,
    };

    let json = serde_json::to_value(request).expect("request should serialize");
    assert_eq!(json["transactionId"], "transaction-1");
    assert_eq!(json["productId"], "pro.monthly");
    assert!(json.get("transaction_id").is_none());
}

#[test]
fn apple_adapter_maps_storekit_and_apns_clients() {
    block_on(async {
        let bridge = BridgeKit::apple(
            Arc::new(FakeAppleStoreKitClient),
            Arc::new(FakeApplePushClient),
        );

        let products = bridge
            .products(ProductRequest::new(["pro.monthly"]))
            .await
            .expect("Apple products should map into BridgeKit products");
        assert_eq!(products[0].storefront, Storefront::AppleAppStore);
        assert_eq!(products[0].subscription.as_ref().unwrap().period, "P1M");

        let purchase = bridge
            .purchase(PurchaseRequest::new("pro.monthly"))
            .await
            .expect("Apple purchase should map into BridgeKit purchase");
        assert_eq!(purchase.receipt.as_deref(), Some("signed-transaction-jws"));
        assert_eq!(purchase.state, bridgekit::TransactionState::Purchased);

        let validation = bridge
            .validate_receipt(ReceiptValidationRequest {
                receipt: "signed-transaction-jws".into(),
                transaction_id: Some("transaction-1".into()),
                product_id: Some("pro.monthly".into()),
                storefront: Storefront::AppleAppStore,
                metadata: serde_json::Value::Null,
            })
            .await
            .expect("Apple validation should map into BridgeKit validation");
        assert!(validation.is_valid);
        assert_eq!(validation.raw["source"], "fake-storekit");

        let authorization = bridge
            .request_push_authorization(PushAuthorizationRequest::default())
            .await
            .expect("Apple push authorization should map");
        assert_eq!(authorization.platform, Platform::Apple);
        assert_eq!(authorization.status, PushAuthorizationStatus::Authorized);

        let registration = bridge
            .register_push(PushRegistrationRequest {
                environment: Some("production".into()),
                metadata: serde_json::Value::Null,
            })
            .await
            .expect("Apple APNs registration should map");
        assert_eq!(registration.token, "apns-device-token");
        assert_eq!(registration.environment.as_deref(), Some("production"));
    });
}

#[test]
fn native_bridge_reports_unsupported_iap_on_non_store_platforms() {
    if Platform::current() != Platform::Other {
        return;
    }

    let error = block_on(BridgeKit::native().products(ProductRequest::new(["pro.monthly"])))
        .expect_err("non-store platforms should be explicit");

    match error {
        BridgeKitError::UnsupportedPlatform {
            capability,
            platform,
        } => {
            assert_eq!(capability, Capability::InAppPurchases);
            assert_eq!(platform, Platform::Other);
        }
        other => panic!("unexpected error: {other}"),
    }
}

struct FakeAppleStoreKitClient;

#[async_trait]
impl AppleStoreKitClient for FakeAppleStoreKitClient {
    async fn products(&self, request: AppleProductRequest) -> Result<Vec<AppleProduct>> {
        assert_eq!(request.product_ids, vec!["pro.monthly"]);
        Ok(vec![AppleProduct {
            id: "pro.monthly".into(),
            localized_title: "Pro Monthly".into(),
            localized_description: "Monthly Pro access".into(),
            display_price: "$4.99".into(),
            currency_code: "USD".into(),
            subscription_period: Some("P1M".into()),
            introductory_price: None,
            trial_period: Some("P7D".into()),
            raw: serde_json::json!({ "source": "fake-storekit" }),
        }])
    }

    async fn purchase(&self, request: ApplePurchaseRequest) -> Result<AppleTransaction> {
        assert_eq!(request.product_id, "pro.monthly");
        Ok(AppleTransaction {
            transaction_id: "transaction-1".into(),
            product_id: request.product_id,
            state: AppleTransactionState::Purchased,
            signed_transaction_jws: Some("signed-transaction-jws".into()),
            app_store_receipt: None,
            original_transaction_id: Some("original-transaction-1".into()),
            purchased_at_ms: Some(1_700_000_000_000),
            expires_at_ms: Some(1_702_592_000_000),
            raw: serde_json::json!({ "source": "fake-storekit" }),
        })
    }

    async fn restore_purchases(&self) -> Result<Vec<AppleTransaction>> {
        Ok(vec![])
    }

    async fn validate_receipt(
        &self,
        request: AppleReceiptValidationRequest,
    ) -> Result<AppleReceiptValidationResult> {
        assert_eq!(request.receipt, "signed-transaction-jws");
        Ok(AppleReceiptValidationResult {
            is_valid: true,
            product_id: request.product_id,
            transaction_id: request.transaction_id,
            expires_at_ms: Some(1_702_592_000_000),
            raw: serde_json::json!({ "source": "fake-storekit" }),
        })
    }
}

struct FakeApplePushClient;

#[async_trait]
impl ApplePushClient for FakeApplePushClient {
    async fn request_authorization(
        &self,
        request: ApplePushAuthorizationRequest,
    ) -> Result<ApplePushAuthorization> {
        assert!(request.alert);
        assert!(request.badge);
        assert!(request.sound);
        Ok(ApplePushAuthorization {
            status: PushAuthorizationStatus::Authorized,
            metadata: serde_json::json!({ "source": "fake-apns" }),
        })
    }

    async fn register(
        &self,
        request: ApplePushRegistrationRequest,
    ) -> Result<ApplePushRegistration> {
        assert_eq!(request.environment, Some(AppleEnvironment::Production));
        Ok(ApplePushRegistration {
            device_token: "apns-device-token".into(),
            environment: request.environment,
            expires_at_ms: None,
            raw: serde_json::json!({ "source": "fake-apns" }),
        })
    }

    async fn unregister(&self) -> Result<()> {
        Ok(())
    }
}

fn sample_product(id: &str, storefront: Storefront) -> Product {
    Product {
        id: id.into(),
        title: "Pro".into(),
        description: "Unlocks Pro features".into(),
        price: "4.99".into(),
        currency_code: "USD".into(),
        storefront,
        subscription: None,
        metadata: serde_json::Value::Null,
    }
}
