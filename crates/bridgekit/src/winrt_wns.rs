//! Windows Push Notification Services WinRT bindings for Windows targets.
#![cfg(windows)]

use crate::microsoft::{
    MicrosoftPushAuthorization, MicrosoftPushAuthorizationRequest, MicrosoftPushRegistration,
    MicrosoftPushRegistrationRequest,
};
use crate::push::PushAuthorizationStatus;
use crate::{BridgeKitError, Result};
use windows::UI::Notifications::PushNotificationChannelManager;

pub fn request_authorization(
    request: MicrosoftPushAuthorizationRequest,
) -> Result<MicrosoftPushAuthorization> {
    Ok(MicrosoftPushAuthorization {
        status: PushAuthorizationStatus::Authorized,
        metadata: serde_json::json!({
            "source": "wns",
            "alert": request.alert,
            "badge": request.badge,
            "sound": request.sound,
            "note": "Windows uses app manifest notification capabilities"
        }),
    })
}

pub fn register(request: MicrosoftPushRegistrationRequest) -> Result<MicrosoftPushRegistration> {
    let channel = PushNotificationChannelManager::CreatePushNotificationChannelForApplicationAsync()
        .map_err(map_winrt_error)?
        .get()
        .map_err(map_winrt_error)?;

    let channel_uri = channel.Uri().map_err(map_winrt_error)?;
    let expires_at_ms = channel
        .ExpirationTime()
        .ok()
        .map(|expiration| filetime_to_unix_ms(expiration.UniversalTime));

    Ok(MicrosoftPushRegistration {
        channel_uri: channel_uri.to_string(),
        environment: request.environment,
        expires_at_ms,
        raw: serde_json::json!({
            "source": "wns"
        }),
    })
}

pub fn unregister() -> Result<()> {
    Ok(())
}

fn filetime_to_unix_ms(universal_time: i64) -> i64 {
    const WINDOWS_TO_UNIX_EPOCH_MS: i64 = 11_644_473_600_000;
    (universal_time / 10_000) - WINDOWS_TO_UNIX_EPOCH_MS
}

fn map_winrt_error(error: windows_core::Error) -> BridgeKitError {
    BridgeKitError::Native(error.to_string())
}
