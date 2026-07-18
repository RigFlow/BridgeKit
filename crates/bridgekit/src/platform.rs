use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Apple,
    Microsoft,
    Other,
}

impl Platform {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(any(target_os = "ios", target_os = "macos")) {
            Self::Apple
        } else if cfg!(target_os = "windows") {
            Self::Microsoft
        } else {
            Self::Other
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apple => formatter.write_str("apple"),
            Self::Microsoft => formatter.write_str("microsoft"),
            Self::Other => formatter.write_str("other"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Storefront {
    AppleAppStore,
    MicrosoftStore,
}

impl Storefront {
    #[must_use]
    pub const fn platform(self) -> Platform {
        match self {
            Self::AppleAppStore => Platform::Apple,
            Self::MicrosoftStore => Platform::Microsoft,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    InAppPurchases,
    PushNotifications,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InAppPurchases => formatter.write_str("in_app_purchases"),
            Self::PushNotifications => formatter.write_str("push_notifications"),
        }
    }
}
