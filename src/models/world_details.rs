use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::{GameWorldType, Location, Player, PvpType, TransferType};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldDetails {
    #[schema(example = "Antica")]
    pub name: String,
    /// If the world is online or not
    pub is_online: bool,
    /// The current number of players online
    #[schema(example = 152)]
    pub players_online_count: u32,
    /// The record number of players online
    #[schema(example = 1211)]
    pub players_online_record: u32,
    /// The date of the record number of players online
    #[schema(value_type = String, format = DateTime)]
    pub players_online_record_date: DateTime<Utc>,
    /// The date the world was created
    #[schema(value_type = String, format = Date)]
    pub creation_date: NaiveDate,
    pub location: Location,
    pub pvp_type: PvpType,
    /// Quest titles achieved on this world
    #[schema(example = json!(["Rise of Devovorga", "The Lightbearer"]))]
    pub world_quest_titles: Vec<String>,
    /// Whether the world has battlEye enabled
    pub battl_eye: bool,
    /// The date battlEye was enabled, if it has battlEye
    #[schema(value_type = Option<String>, format = Date)]
    pub battl_eye_date: Option<NaiveDate>,
    pub game_world_type: GameWorldType,
    pub transfer_type: Option<TransferType>,
    /// If premium is required to play on this world
    pub premium_required: bool,
    pub players_online: Vec<Player>,
}
