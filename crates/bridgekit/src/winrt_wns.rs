//! Windows Push Notification Services WinRT bindings for Windows targets.
#![cfg(windows)]

use crate::microsoft::{
    MicrosoftPushAuthorization, MicrosoftPushAuthorizationRequest, MicrosoftPushRegistration,
    MicrosoftPushRegistrationRequest,
};
use crate::push::PushAuthorizationStatus;
use crate::{BridgeKitError, Result};
use windows::UI::Notifications::{NotificationSetting, PushNotificationChannelManager, ToastNotificationManager};

pub fn request_authorization(
    request: MicrosoftPushAuthorizationRequest,
) -> Result<MicrosoftPushAuthorization> {
    let manager = ToastNotificationManager::GetDefault().map_err(map_winrt_error)?;
    let setting = manager.Setting().map_err(map_winrt_error)?;
    let status = map_notification_setting(setting);

    Ok(MicrosoftPushAuthorization {
        status,
        metadata: serde_json::json!({
            "source": "wns",
            "alert": request.alert,
            "badge": request.badge,
            "sound": request.sound,
            "notificationSetting": format!("{setting:?}")
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

fn map_notification_setting(setting: NotificationSetting) -> PushAuthorizationStatus {
    match setting {
        NotificationSetting::Enabled => PushAuthorizationStatus::Authorized,
        NotificationSetting::DisabledForApplication | NotificationSetting::DisabledForUser => {
            PushAuthorizationStatus::Denied
        }
        NotificationSetting::DisabledByGroupPolicy => PushAuthorizationStatus::Unsupported,
        NotificationSetting::DisabledByManifest => PushAuthorizationStatus::Denied,
        _ => PushAuthorizationStatus::NotDetermined,
    }
}

fn filetime_to_unix_ms(universal_time: i64) -> i64 {
    const WINDOWS_TO_UNIX_EPOCH_MS: i64 = 11_644_473_600_000;
    (universal_time / 10_000) - WINDOWS_TO_UNIX_EPOCH_MS
}

fn map_winrt_error(error: windows_core::Error) -> BridgeKitError {
    BridgeKitError::Native(error.to_string())
}
