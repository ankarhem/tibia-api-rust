use super::Vocation;
use serde::Serialize;
use utoipa::ToSchema;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[schema(example = "Urinchoklad")]
    pub name: String,
    #[schema(example = "52")]
    pub level: u32,
    pub vocation: Option<Vocation>,
}
