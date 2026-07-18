//! Microsoft Store collections API helpers for backend license validation.

use crate::{BridgeKitError, Result};
use serde::{Deserialize, Serialize};

const COLLECTIONS_API_URL: &str = "https://collections.mp.microsoft.com/v9.0/collections/query";

/// Request to query Microsoft Store collections for a user.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftCollectionsQueryRequest {
    pub beneficiaries: Vec<MicrosoftCollectionsBeneficiary>,
    pub product_types: Vec<String>,
    pub validity_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftCollectionsBeneficiary {
    pub identity_type: String,
    pub identity_value: String,
    pub local_ticket_reference: String,
}

/// A product entry returned by the collections API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftCollectionItem {
    pub product_id: String,
    pub acquired_date: Option<String>,
    pub end_date: Option<String>,
    pub item_type: Option<String>,
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub raw: serde_json::Value,
}

/// Client for Microsoft Store collections queries.
#[derive(Debug, Clone)]
pub struct MicrosoftCollectionsClient {
    access_token: String,
    client: reqwest::Client,
}

impl MicrosoftCollectionsClient {
    #[must_use]
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Queries collections for the supplied user and product IDs.
    pub async fn query_user_collections(
        &self,
        user_id: &str,
        product_ids: &[String],
    ) -> Result<Vec<MicrosoftCollectionItem>> {
        if product_ids.is_empty() {
            return Err(BridgeKitError::InvalidRequest(
                "Microsoft collections query requires at least one product id".into(),
            ));
        }

        let request = MicrosoftCollectionsQueryRequest {
            beneficiaries: vec![MicrosoftCollectionsBeneficiary {
                identity_type: "b2b".into(),
                identity_value: user_id.into(),
                local_ticket_reference: user_id.into(),
            }],
            product_types: vec!["Durable".into(), "UnmanagedConsumable".into(), "Consumable".into()],
            validity_type: "Valid".into(),
        };

        let response = self
            .client
            .post(COLLECTIONS_API_URL)
            .bearer_auth(&self.access_token)
            .json(&request)
            .send()
            .await
            .map_err(|error| {
                BridgeKitError::Native(format!("Microsoft collections request failed: {error}"))
            })?;

        if !response.status().is_success() {
            return Err(BridgeKitError::ReceiptValidationFailed(format!(
                "Microsoft collections API returned HTTP {}",
                response.status()
            )));
        }

        let raw: serde_json::Value = response.json().await.map_err(|error| {
            BridgeKitError::Native(format!("Microsoft collections response was invalid JSON: {error}"))
        })?;

        parse_collection_items(&raw, product_ids)
    }
}

fn parse_collection_items(
    raw: &serde_json::Value,
    product_ids: &[String],
) -> Result<Vec<MicrosoftCollectionItem>> {
    let items = raw
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut matches = Vec::new();
    for item in items {
        let product_id = item
            .get("productId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();

        if !product_ids.iter().any(|id| id == &product_id) {
            continue;
        }

        matches.push(MicrosoftCollectionItem {
            product_id,
            acquired_date: item
                .get("acquiredDate")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            end_date: item
                .get("endDate")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            item_type: item
                .get("itemType")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            status: item
                .get("status")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            raw: item,
        });
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_collection_items_filters_requested_product_ids() {
        let raw = serde_json::json!({
            "items": [
                {
                    "productId": "9NBLGGH4R315",
                    "acquiredDate": "2024-01-01T00:00:00Z",
                    "status": "Active"
                },
                {
                    "productId": "9OTHERPRODUCT",
                    "status": "Active"
                }
            ]
        });

        let items = parse_collection_items(
            &raw,
            &["9NBLGGH4R315".to_string()],
        )
        .expect("collections should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].product_id, "9NBLGGH4R315");
        assert_eq!(items[0].status.as_deref(), Some("Active"));
    }
}
