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
    #[serde(rename_all = "camelCase")]
    AuctionWithBid {
        bid: u32,
        #[schema(value_type = String, format = DateTime)]
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
    /// The id of the residence (houseid)
    pub id: u32,
    #[schema(example = "Thais")]
    pub town: String,
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
