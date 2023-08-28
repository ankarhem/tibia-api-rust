use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The house type
#[derive(Serialize, Clone, Copy, Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResidenceType {
    House,
    Guildhall,
}

/// The residence status
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResidenceStatus {
    Rented,
    AuctionNoBid,
    AuctionWithBid {
        bid: u32,
        expiry_time: DateTime<Utc>,
    },
    AuctionFinished {
        bid: u32,
    },
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Residence {
    #[serde(rename = "type")]
    pub residence_type: ResidenceType,
    /// The house's name
    #[schema(example = "Coastwood 1")]
    pub name: String,
    /// The size in sqm
    #[schema(example = 16)]
    pub size: u16,
    /// The rent
    #[schema(example = 50000)]
    pub rent: u32,
    pub status: ResidenceStatus,
}
