//! Windows Push Notification Services WinRT bindings for Windows targets.
#![cfg(windows)]

use crate::microsoft::{
    MicrosoftPushAuthorization, MicrosoftPushAuthorizationRequest, MicrosoftPushRegistration,
    MicrosoftPushRegistrationRequest, MicrosoftStoreEnvironment,
};
use crate::push::PushAuthorizationStatus;
use crate::{BridgeKitError, Result};
use std::sync::Mutex;
use windows::UI::Notifications::{NotificationSetting, PushNotificationChannelManager, ToastNotificationManager};

const CHANNEL_REFRESH_BUFFER_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone)]
struct CachedChannel {
    channel_uri: String,
    environment: Option<MicrosoftStoreEnvironment>,
    expires_at_ms: Option<i64>,
}

static CACHED_CHANNEL: Mutex<Option<CachedChannel>> = Mutex::new(None);

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
    if let Ok(guard) = CACHED_CHANNEL.lock() {
        if let Some(cached) = guard.as_ref() {
            if cached.environment == request.environment && !should_refresh_channel(cached.expires_at_ms)
            {
                return Ok(MicrosoftPushRegistration {
                    channel_uri: cached.channel_uri.clone(),
                    environment: request.environment,
                    expires_at_ms: cached.expires_at_ms,
                    raw: serde_json::json!({
                        "source": "wns",
                        "cache": "hit"
                    }),
                });
            }
        }
    }

    let channel = PushNotificationChannelManager::CreatePushNotificationChannelForApplicationAsync()
        .map_err(map_winrt_error)?
        .get()
        .map_err(map_winrt_error)?;

    let channel_uri = channel.Uri().map_err(map_winrt_error)?;
    let expires_at_ms = channel
        .ExpirationTime()
        .ok()
        .map(|expiration| filetime_to_unix_ms(expiration.UniversalTime));

    let registration = MicrosoftPushRegistration {
        channel_uri: channel_uri.to_string(),
        environment: request.environment,
        expires_at_ms,
        raw: serde_json::json!({
            "source": "wns",
            "cache": "miss"
        }),
    };

    if let Ok(mut guard) = CACHED_CHANNEL.lock() {
        *guard = Some(CachedChannel {
            channel_uri: registration.channel_uri.clone(),
            environment: registration.environment,
            expires_at_ms: registration.expires_at_ms,
        });
    }

    Ok(registration)
}

pub fn unregister() -> Result<()> {
    if let Ok(mut guard) = CACHED_CHANNEL.lock() {
        *guard = None;
    }
    Ok(())
}

fn should_refresh_channel(expires_at_ms: Option<i64>) -> bool {
    match expires_at_ms {
        None => false,
        Some(expires_at_ms) => unix_ms_now() + CHANNEL_REFRESH_BUFFER_MS >= expires_at_ms,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refresh_channel_when_near_expiry() {
        let soon = unix_ms_now() + CHANNEL_REFRESH_BUFFER_MS - 1;
        assert!(should_refresh_channel(Some(soon)));
    }

    #[test]
    fn should_not_refresh_channel_when_far_from_expiry() {
        let later = unix_ms_now() + CHANNEL_REFRESH_BUFFER_MS + 60_000;
        assert!(!should_refresh_channel(Some(later)));
    }
}
