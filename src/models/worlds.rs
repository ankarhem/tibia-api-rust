use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::{GameWorldType, Location, PvpType, TransferType};

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

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct World {
    #[schema(example = "Antica")]
    pub name: String,
    /// Current number of players online in this world
    #[schema(example = 1337)]
    pub players_online_count: u32,
    pub location: Location,
    pub pvp_type: PvpType,
    /// Whether the world has battlEye enabled
    pub battl_eye: bool,
    /// The date battlEye was enabled, if it has battlEye
    #[schema(value_type = Option<String>, format = Date)]
    pub battl_eye_date: Option<NaiveDate>,
    /// If premium is required to play on this world
    #[schema(example = false)]
    pub premium_required: bool,
    pub transfer_type: Option<TransferType>,
    pub game_world_type: GameWorldType,
}
