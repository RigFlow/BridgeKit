use bridgekit::{
    BridgeKit, Product, ProductRequest, Purchase, PurchaseRequest, PushAuthorization,
    PushAuthorizationRequest, PushRegistration, PushRegistrationRequest, ReceiptValidationRequest,
    ReceiptValidationResult,
};
use tauri::{plugin::TauriPlugin, Manager, Runtime, State};

pub type CommandResult<T> = std::result::Result<T, String>;

#[derive(Clone)]
pub struct BridgeKitState {
    bridgekit: BridgeKit,
}

impl BridgeKitState {
    #[must_use]
    pub fn new(bridgekit: BridgeKit) -> Self {
        Self { bridgekit }
    }

    #[must_use]
    pub fn bridgekit(&self) -> &BridgeKit {
        &self.bridgekit
    }
}

pub fn init<R: Runtime>(bridgekit: BridgeKit) -> TauriPlugin<R> {
    tauri::plugin::Builder::new("bridgekit")
        .setup(move |app, _api| {
            app.manage(BridgeKitState::new(bridgekit));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bridgekit_products,
            bridgekit_purchase,
            bridgekit_restore_purchases,
            bridgekit_validate_receipt,
            bridgekit_request_push_authorization,
            bridgekit_register_push,
            bridgekit_unregister_push,
        ])
        .build()
}

#[tauri::command]
async fn bridgekit_products(
    state: State<'_, BridgeKitState>,
    request: ProductRequest,
) -> CommandResult<Vec<Product>> {
    state
        .bridgekit()
        .products(request)
        .await
        .map_err(command_error)
}

#[tauri::command]
async fn bridgekit_purchase(
    state: State<'_, BridgeKitState>,
    request: PurchaseRequest,
) -> CommandResult<Purchase> {
    state
        .bridgekit()
        .purchase(request)
        .await
        .map_err(command_error)
}

#[tauri::command]
async fn bridgekit_restore_purchases(
    state: State<'_, BridgeKitState>,
) -> CommandResult<Vec<Purchase>> {
    state
        .bridgekit()
        .restore_purchases()
        .await
        .map_err(command_error)
}

#[tauri::command]
async fn bridgekit_validate_receipt(
    state: State<'_, BridgeKitState>,
    request: ReceiptValidationRequest,
) -> CommandResult<ReceiptValidationResult> {
    state
        .bridgekit()
        .validate_receipt(request)
        .await
        .map_err(command_error)
}

#[tauri::command]
async fn bridgekit_request_push_authorization(
    state: State<'_, BridgeKitState>,
    request: PushAuthorizationRequest,
) -> CommandResult<PushAuthorization> {
    state
        .bridgekit()
        .request_push_authorization(request)
        .await
        .map_err(command_error)
}

#[tauri::command]
async fn bridgekit_register_push(
    state: State<'_, BridgeKitState>,
    request: PushRegistrationRequest,
) -> CommandResult<PushRegistration> {
    state
        .bridgekit()
        .register_push(request)
        .await
        .map_err(command_error)
}

#[tauri::command]
async fn bridgekit_unregister_push(state: State<'_, BridgeKitState>) -> CommandResult<()> {
    state
        .bridgekit()
        .unregister_push()
        .await
        .map_err(command_error)
}

fn command_error(error: bridgekit::BridgeKitError) -> String {
    error.to_string()
}
