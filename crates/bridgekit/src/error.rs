use crate::{Capability, Platform};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BridgeKitError>;

#[derive(Debug, Error)]
pub enum BridgeKitError {
    #[error("{capability} is not supported on {platform}")]
    UnsupportedPlatform {
        capability: Capability,
        platform: Platform,
    },

    #[error("provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("receipt validation failed: {0}")]
    ReceiptValidationFailed(String),

    #[error("native integration failed: {0}")]
    Native(String),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}
