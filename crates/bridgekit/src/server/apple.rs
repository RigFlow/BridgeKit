//! Apple App Store Server API helpers for backend receipt validation.

use crate::apple::AppleEnvironment;
use crate::{BridgeKitError, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_STORE_AUDIENCE: &str = "appstoreconnect-v1";
const JWT_LIFETIME_SECONDS: u64 = 1200;

/// Credentials for App Store Server API requests.
#[derive(Debug, Clone)]
pub struct AppStoreServerConfig {
    pub issuer_id: String,
    pub key_id: String,
    pub bundle_id: String,
    pub private_key_pem: String,
}

/// Request to fetch transaction metadata from the App Store Server API.
#[derive(Debug, Clone)]
pub struct AppStoreTransactionInfoRequest {
    pub transaction_id: String,
    pub environment: AppleEnvironment,
}

/// Response from the App Store Server API transaction endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreTransactionInfo {
    pub signed_transaction_info: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct AppStoreJwtClaims {
    iss: String,
    iat: u64,
    exp: u64,
    aud: &'static str,
    bid: String,
}

/// Generates a short-lived JWT for App Store Server API authentication.
pub fn generate_app_store_jwt(config: &AppStoreServerConfig) -> Result<String> {
    let issued_at = unix_seconds_now();
    let claims = AppStoreJwtClaims {
        iss: config.issuer_id.clone(),
        iat: issued_at,
        exp: issued_at + JWT_LIFETIME_SECONDS,
        aud: APP_STORE_AUDIENCE,
        bid: config.bundle_id.clone(),
    };

    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(config.key_id.clone());

    encode(
        &header,
        &claims,
        &EncodingKey::from_ec_pem(config.private_key_pem.as_bytes()).map_err(|error| {
            BridgeKitError::InvalidRequest(format!(
                "App Store private key is not valid PEM: {error}"
            ))
        })?,
    )
    .map_err(|error| BridgeKitError::Native(format!("failed to sign App Store JWT: {error}")))
}

/// Fetches signed transaction info from the App Store Server API.
pub async fn get_transaction_info(
    config: &AppStoreServerConfig,
    request: AppStoreTransactionInfoRequest,
) -> Result<AppStoreTransactionInfo> {
    let jwt = generate_app_store_jwt(config)?;
    let base_url = match request.environment {
        AppleEnvironment::Sandbox => "https://api.storekit-sandbox.itunes.apple.com",
        AppleEnvironment::Production => "https://api.storekit.itunes.apple.com",
    };
    let url = format!(
        "{base_url}/inApps/v1/transactions/{}",
        request.transaction_id
    );

    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(jwt)
        .send()
        .await
        .map_err(|error| BridgeKitError::Native(format!("App Store request failed: {error}")))?;

    if !response.status().is_success() {
        return Err(BridgeKitError::ReceiptValidationFailed(format!(
            "App Store Server API returned HTTP {}",
            response.status()
        )));
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .map_err(|error| BridgeKitError::Native(format!("App Store response was invalid JSON: {error}")))?;

    let signed_transaction_info = raw
        .get("signedTransactionInfo")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            BridgeKitError::ReceiptValidationFailed(
                "App Store Server API response did not include signedTransactionInfo".into(),
            )
        })?
        .to_string();

    Ok(AppStoreTransactionInfo {
        signed_transaction_info,
        raw,
    })
}

fn unix_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\n\
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg0gF2ERdrM8eMCzkb\n\
IUOX7vKPkMv+Z1gd90L/nCuZbQmhRANCAASWpokn4pxshP27ckCtVk+Z1PJ8RGHC\n\
KMaVbIahbyUumIa78cFzwTQjtpw+ecEDJtg6FpWN73uvbcmEUI8SReZo\n\
-----END PRIVATE KEY-----";

    #[test]
    fn generate_app_store_jwt_returns_three_segments() {
        let config = AppStoreServerConfig {
            issuer_id: "issuer-id".into(),
            key_id: "key-id".into(),
            bundle_id: "com.example.app".into(),
            private_key_pem: SAMPLE_PRIVATE_KEY.into(),
        };

        let jwt = generate_app_store_jwt(&config).expect("JWT should be generated");
        assert_eq!(jwt.matches('.').count(), 2);
    }
}
