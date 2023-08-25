use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::World;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldsResponse {
    pub players_online_total: u32,
    pub record_players: u32,
    // #[schema(example = "2014-11-28T12:45:59.324310806Z")]
    pub record_date: DateTime<Utc>,
    pub worlds: Vec<World>,
}
