use bridgekit::mock::{MockPushProvider, MockStoreProvider};
use bridgekit::{BridgeKit, Platform, Product, Storefront, SubscriptionInfo};
use std::sync::Arc;

fn main() {
    let bridgekit = BridgeKit::with_providers(
        Arc::new(MockStoreProvider::apple(vec![sample_product()])),
        Arc::new(MockPushProvider::new(Platform::Apple, "mock-apns-token")),
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_bridgekit::init(bridgekit))
        .run(tauri::generate_context!())
        .expect("failed to run BridgeKit mock Tauri app");
}

fn sample_product() -> Product {
    Product {
        id: "pro.monthly".into(),
        title: "Pro Monthly".into(),
        description: "Unlocks mock Pro features for one month".into(),
        price: "$4.99".into(),
        currency_code: "USD".into(),
        storefront: Storefront::AppleAppStore,
        subscription: Some(SubscriptionInfo {
            period: "P1M".into(),
            introductory_price: None,
            trial_period: Some("P7D".into()),
        }),
        metadata: serde_json::json!({
            "example": "mock-tauri-app"
        }),
    }
}
