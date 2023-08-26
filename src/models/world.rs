use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use super::{GameWorldType, Location, PvpType, TransferType};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct World {
    #[schema(example = "Antica")]
    pub name: String,
    #[schema(example = 1337)]
    pub players_online_count: u32,
    pub location: Location,
    pub pvp_type: PvpType,
    pub battl_eye: bool,
    #[schema(value_type = Option<String>, format = Date)]
    pub battl_eye_date: Option<NaiveDate>,
    #[schema(example = false)]
    pub premium_required: bool,
    pub transfer_type: Option<TransferType>,
    pub game_world_type: GameWorldType,
}
