use bridgekit::mock::{MockPushProvider, MockStoreProvider};
use bridgekit::{
    BridgeKit, BridgeKitError, Capability, Platform, Product, ProductRequest, PurchaseRequest,
    PushAuthorizationRequest, PushRegistrationRequest, ReceiptValidationRequest, Storefront,
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
