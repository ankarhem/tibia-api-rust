use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KilledAmounts {
    /// The amount of players killed by monsters
    pub killed_players: u32,
    /// The amount of monsters killed by players
    pub killed_by_players: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RaceKillStatistics {
    /// The name of the race
    pub race: String,
    /// The kill statistics for the last day
    pub last_day: KilledAmounts,
    /// The aggregated kill statistics for the last week
    pub last_week: KilledAmounts,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KillStatistics {
    pub total_last_day: KilledAmounts,
    pub total_last_week: KilledAmounts,
    /// A list of kill statistics for each race
    pub races: Vec<RaceKillStatistics>,
}
