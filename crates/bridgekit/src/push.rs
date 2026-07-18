use crate::{Platform, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushAuthorizationRequest {
    #[serde(default = "default_true")]
    pub alert: bool,
    #[serde(default = "default_true")]
    pub badge: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
    #[serde(default)]
    pub provisional: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl Default for PushAuthorizationRequest {
    fn default() -> Self {
        Self {
            alert: true,
            badge: true,
            sound: true,
            provisional: false,
            metadata: serde_json::Value::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushAuthorizationStatus {
    NotDetermined,
    Denied,
    Authorized,
    Provisional,
    Ephemeral,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushAuthorization {
    pub status: PushAuthorizationStatus,
    pub platform: Platform,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushRegistrationRequest {
    /// Apple environments are normally `sandbox` or `production`; Microsoft
    /// uses a WNS channel URI and does not require this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

impl Default for PushRegistrationRequest {
    fn default() -> Self {
        Self {
            environment: None,
            metadata: serde_json::Value::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushRegistration {
    /// APNs device token on Apple platforms or WNS channel URI on Windows.
    pub token: String,
    pub platform: Platform,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPayload {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub data: serde_json::Map<String, serde_json::Value>,
}

#[async_trait]
pub trait PushProvider: Send + Sync {
    async fn request_authorization(
        &self,
        request: PushAuthorizationRequest,
    ) -> Result<PushAuthorization>;

    async fn register(&self, request: PushRegistrationRequest) -> Result<PushRegistration>;

    async fn unregister(&self) -> Result<()>;
}

const fn default_true() -> bool {
    true
}
