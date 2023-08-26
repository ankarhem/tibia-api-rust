use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::World;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldsResponse {
    /// The current number of players online in all worlds
    #[schema(example = 1234)]
    pub players_online_total: u32,
    /// The record number of players online in all worlds
    #[schema(example = 64_028)]
    pub record_players: u32,
    /// The date of the record number of players online in all worlds
    #[schema(value_type = String, format = DateTime)]
    pub record_date: DateTime<Utc>,
    pub worlds: Vec<World>,
}
