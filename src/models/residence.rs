use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The house type
#[derive(Serialize, Clone, Copy, Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResidenceType {
    House,
    Guildhall,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Residence {
    #[serde(rename = "type")]
    pub residence_type: ResidenceType,
    /// The house's name
    pub name: String,
    /// The size in sqm
    pub size: u16,
    /// The rent
    pub rent: u32,
    pub status: String,
}
