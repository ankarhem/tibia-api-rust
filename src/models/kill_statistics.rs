use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KilledAmounts {
    pub killed_players: u32,
    pub killed_by_players: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RaceKillStatistics {
    pub race: String,
    pub last_day: KilledAmounts,
    pub last_week: KilledAmounts,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KillStatistics {
    pub total_last_day: KilledAmounts,
    pub total_last_week: KilledAmounts,
    pub races: Vec<RaceKillStatistics>,
}
