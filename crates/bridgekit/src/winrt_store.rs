//! Microsoft Store WinRT bindings for Windows targets.
#![cfg(windows)]

use crate::microsoft::{
    format_store_duration, MicrosoftLicenseValidationRequest, MicrosoftLicenseValidationResult,
    MicrosoftProduct, MicrosoftProductKind, MicrosoftProductRequest, MicrosoftPurchase,
    MicrosoftPurchaseRequest, MicrosoftTransactionState,
};
use crate::{BridgeKitError, Result};
use std::sync::OnceLock;
use windows::Services::Store::{
    StoreContext, StoreDurationUnit, StoreProduct, StorePurchaseStatus, StorePurchaseResult,
    StoreSku,
};
use windows::System::User;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::IInitializeWithWindow;
use windows_collections::IIterable;
use windows_core::{Interface, HSTRING};

const PRODUCT_KINDS: &[&str] = &[
    "Consumable",
    "Durable",
    "UnmanagedConsumable",
    "Subscription",
];

static STORE_OWNER_HWND: OnceLock<isize> = OnceLock::new();
static STORE_USER: OnceLock<User> = OnceLock::new();

/// Associates the owner `HWND` used by Microsoft Store purchase UI.
pub fn set_store_window_handle(hwnd: isize) {
    let _ = STORE_OWNER_HWND.set(hwnd);
}

/// Resolves the current Windows user and uses `StoreContext::GetForUser` for
/// subsequent Store requests.
pub fn set_store_context_for_current_windows_user() -> Result<()> {
    let user = current_windows_user()?;
    STORE_USER
        .set(user)
        .map_err(|_| BridgeKitError::Native("Microsoft Store user context is already set".into()))
}

pub fn products(request: MicrosoftProductRequest) -> Result<Vec<MicrosoftProduct>> {
    let context = store_context()?;
    let kinds = hstring_iterable(PRODUCT_KINDS);
    let store_ids = hstring_iterable(
        &request
            .product_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );

    let query = context
        .GetStoreProductsAsync(&kinds, &store_ids)
        .map_err(map_winrt_error)?;
    let result = query.get().map_err(map_winrt_error)?;
    ensure_query_success(&result)?;

    let mut products = Vec::new();
    let product_map = result.Products().map_err(map_winrt_error)?;
    for entry in product_map.iter() {
        let (_, product) = entry.map_err(map_winrt_error)?;
        products.push(map_store_product(&product)?);
    }

    Ok(products)
}

pub fn purchase(request: MicrosoftPurchaseRequest) -> Result<MicrosoftPurchase> {
    let context = store_context()?;
    let purchase_result = context
        .RequestPurchaseAsync(&HSTRING::from(request.product_id.as_str()))
        .map_err(map_winrt_error)?
        .get()
        .map_err(map_winrt_error)?;

    map_purchase_result(&request.product_id, purchase_result)
}

pub fn restore_purchases() -> Result<Vec<MicrosoftPurchase>> {
    let context = store_context()?;
    let kinds = hstring_iterable(PRODUCT_KINDS);
    let query = context
        .GetUserCollectionAsync(&kinds)
        .map_err(map_winrt_error)?
        .get()
        .map_err(map_winrt_error)?;
    ensure_query_success(&query)?;

    let mut purchases = Vec::new();
    let product_map = query.Products().map_err(map_winrt_error)?;
    for entry in product_map.iter() {
        let (store_id, product) = entry.map_err(map_winrt_error)?;
        purchases.push(map_owned_product(&store_id, &product)?);
    }

    Ok(purchases)
}

pub fn validate_license(
    request: MicrosoftLicenseValidationRequest,
) -> Result<MicrosoftLicenseValidationResult> {
    let purchases = restore_purchases()?;

    for purchase in &purchases {
        if let Some(product_id) = &request.product_id {
            if purchase.product_id != *product_id {
                continue;
            }
        }

        let receipt_matches = purchase
            .license_token
            .as_deref()
            .is_some_and(|token| token == request.receipt);
        let transaction_matches = request
            .transaction_id
            .as_deref()
            .is_some_and(|transaction_id| transaction_id == purchase.transaction_id);

        if receipt_matches || transaction_matches {
            return Ok(MicrosoftLicenseValidationResult {
                is_valid: true,
                product_id: Some(purchase.product_id.clone()),
                transaction_id: Some(purchase.transaction_id.clone()),
                expires_at_ms: purchase.expires_at_ms,
                raw: serde_json::json!({
                    "source": "microsoft_store",
                    "verification": "verified"
                }),
            });
        }
    }

    let context = store_context()?;
    let app_license = context
        .GetAppLicenseAsync()
        .map_err(map_winrt_error)?
        .get()
        .map_err(map_winrt_error)?;

    let sku_store_id = app_license.SkuStoreId().map_err(map_winrt_error)?;
    let sku_store_id = sku_store_id.to_string();
    let product_matches = request
        .product_id
        .as_deref()
        .is_none_or(|product_id| product_id == sku_store_id);
    let receipt_matches = request.receipt.is_empty() || request.receipt == sku_store_id;

    if app_license.IsActive().map_err(map_winrt_error)? && product_matches && receipt_matches {
        return Ok(MicrosoftLicenseValidationResult {
            is_valid: true,
            product_id: Some(sku_store_id),
            transaction_id: request.transaction_id,
            expires_at_ms: app_license
                .ExpirationDate()
                .ok()
                .map(|date_time| filetime_to_unix_ms(date_time.UniversalTime)),
            raw: serde_json::json!({
                "source": "microsoft_store",
                "verification": "app_license"
            }),
        });
    }

    Ok(MicrosoftLicenseValidationResult {
        is_valid: false,
        product_id: request.product_id,
        transaction_id: request.transaction_id,
        expires_at_ms: None,
        raw: serde_json::json!({
            "source": "microsoft_store",
            "verification": "not_found"
        }),
    })
}

fn store_context() -> Result<StoreContext> {
    let context = if let Some(user) = STORE_USER.get() {
        StoreContext::GetForUser(user).map_err(map_winrt_error)?
    } else {
        StoreContext::GetDefault().map_err(map_winrt_error)?
    };

    if let Some(hwnd) = STORE_OWNER_HWND.get() {
        initialize_store_context(&context, *hwnd)?;
    }
    Ok(context)
}

fn current_windows_user() -> Result<User> {
    let users = User::FindAllAsync()
        .map_err(map_winrt_error)?
        .get()
        .map_err(map_winrt_error)?;

    for user in users {
        if user.Type().map_err(map_winrt_error)? == windows::System::UserType::LocalUser {
            return Ok(user);
        }
    }

    users
        .into_iter()
        .next()
        .ok_or_else(|| BridgeKitError::Native("no Windows user is available for StoreContext".into()))
}

fn initialize_store_context(context: &StoreContext, hwnd: isize) -> Result<()> {
    let initializer: IInitializeWithWindow = context.cast().map_err(map_winrt_error)?;
    unsafe {
        initializer
            .Initialize(HWND(hwnd as *mut std::ffi::c_void))
            .map_err(map_winrt_error)
    }
}

fn ensure_query_success(
    result: &windows::Services::Store::StoreProductQueryResult,
) -> Result<()> {
    result.ExtendedError().map_err(|error| {
        BridgeKitError::Native(format!("Microsoft Store query failed: {error}"))
    })
}

fn map_store_product(product: &StoreProduct) -> Result<MicrosoftProduct> {
    let store_id = product.StoreId().map_err(map_winrt_error)?;
    let title = product.Title().map_err(map_winrt_error)?;
    let description = product.Description().map_err(map_winrt_error)?;
    let price = product.Price().map_err(map_winrt_error)?;
    let product_kind = product.ProductKind().map_err(map_winrt_error)?;
    let (subscription_period, trial_period) = subscription_metadata(product)?;

    Ok(MicrosoftProduct {
        store_id: store_id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        display_price: price.FormattedPrice().map_err(map_winrt_error)?.to_string(),
        currency_code: price.CurrencyCode().map_err(map_winrt_error)?.to_string(),
        kind: map_product_kind(&product_kind.to_string()),
        subscription_period,
        trial_period,
        raw: serde_json::json!({
            "source": "microsoft_store",
            "productKind": product_kind.to_string()
        }),
    })
}

fn subscription_metadata(product: &StoreProduct) -> Result<(Option<String>, Option<String>)> {
    let skus = product.Skus().map_err(map_winrt_error)?;
    for entry in skus.iter() {
        let sku = entry.map_err(map_winrt_error)?;
        if !sku.IsSubscription().map_err(map_winrt_error)? {
            continue;
        }

        let info = sku.SubscriptionInfo().map_err(map_winrt_error)?;
        let subscription_period = Some(format_store_duration(
            info.BillingPeriod().map_err(map_winrt_error)?,
            &duration_unit_name(info.BillingPeriodUnit().map_err(map_winrt_error)?),
        ));
        let trial_period = if info.HasTrialPeriod().map_err(map_winrt_error)? {
            Some(format_store_duration(
                info.TrialPeriod().map_err(map_winrt_error)?,
                &duration_unit_name(info.TrialPeriodUnit().map_err(map_winrt_error)?),
            ))
        } else {
            None
        };

        return Ok((subscription_period, trial_period));
    }

    Ok((None, None))
}

fn duration_unit_name(unit: StoreDurationUnit) -> String {
    format!("{unit:?}")
}

fn map_owned_product(store_id: &str, product: &StoreProduct) -> Result<MicrosoftPurchase> {
    let mapped = map_store_product(product)?;
    let license_token = product
        .Skus()
        .ok()
        .and_then(|skus| first_sku(&skus).ok())
        .and_then(|sku| sku_license_token(&sku))
        .or_else(|| Some(store_id.to_string()));

    Ok(MicrosoftPurchase {
        transaction_id: format!("restored-{store_id}"),
        product_id: mapped.store_id,
        state: MicrosoftTransactionState::Restored,
        license_token,
        collection_id: Some(store_id.to_string()),
        purchased_at_ms: Some(unix_ms_now()),
        expires_at_ms: None,
        raw: serde_json::json!({
            "source": "microsoft_store",
            "verification": "user_collection"
        }),
    })
}

fn first_sku(
    skus: &windows::Foundation::Collections::IMapView<HSTRING, StoreSku>,
) -> Result<StoreSku> {
    for entry in skus.iter() {
        let (_, sku) = entry.map_err(map_winrt_error)?;
        return Ok(sku);
    }

    Err(BridgeKitError::Native(
        "Microsoft Store product did not contain any SKUs".into(),
    ))
}

fn sku_license_token(sku: &StoreSku) -> Option<String> {
    sku.InAppOfferToken()
        .ok()
        .map(|token| token.to_string())
        .filter(|token| !token.is_empty())
        .or_else(|| {
            sku.ExtendedJsonData()
                .ok()
                .map(|value| value.to_string())
                .filter(|value| !value.is_empty())
        })
}

fn map_purchase_result(
    product_id: &str,
    purchase_result: StorePurchaseResult,
) -> Result<MicrosoftPurchase> {
    let status = purchase_result.Status().map_err(map_winrt_error)?;
    let state = match status {
        StorePurchaseStatus::Succeeded => MicrosoftTransactionState::Purchased,
        StorePurchaseStatus::AlreadyPurchased => MicrosoftTransactionState::Restored,
        StorePurchaseStatus::NotPurchased => MicrosoftTransactionState::Cancelled,
        _ => MicrosoftTransactionState::Failed,
    };

    let license_token = purchase_result
        .ExtendedJsonData()
        .ok()
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty());

    Ok(MicrosoftPurchase {
        transaction_id: format!("{product_id}-{}", unix_ms_now()),
        product_id: product_id.to_string(),
        state,
        license_token,
        collection_id: None,
        purchased_at_ms: Some(unix_ms_now()),
        expires_at_ms: None,
        raw: serde_json::json!({
            "source": "microsoft_store",
            "status": format!("{status:?}")
        }),
    })
}

fn map_product_kind(product_kind: &str) -> MicrosoftProductKind {
    match product_kind {
        "Consumable" | "UnmanagedConsumable" => MicrosoftProductKind::Consumable,
        "Subscription" => MicrosoftProductKind::Subscription,
        _ => MicrosoftProductKind::Durable,
    }
}

fn hstring_iterable(values: &[&str]) -> IIterable<HSTRING> {
    IIterable::from(
        values
            .iter()
            .map(|value| HSTRING::from(*value))
            .collect::<Vec<_>>(),
    )
}

fn unix_ms_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn filetime_to_unix_ms(universal_time: i64) -> i64 {
    const WINDOWS_TO_UNIX_EPOCH_MS: i64 = 11_644_473_600_000;
    (universal_time / 10_000) - WINDOWS_TO_UNIX_EPOCH_MS
}

fn map_winrt_error(error: windows_core::Error) -> BridgeKitError {
    BridgeKitError::Native(error.to_string())
}
