use serde::Serialize;
use utoipa::ToSchema;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Guild {
    /// The guild's logo URL.
    pub logo: Option<String>,
    /// The guild name
    pub name: String,
    /// The guild's description (preserved formatting)
    pub description: Option<String>,
    /// Whether the guild is still in formation or not
    pub active: bool,
}
