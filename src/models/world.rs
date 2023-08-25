use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::{GameWorldType, Location, PvpType, TransferType};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct World {
    #[schema(example = "Antica")]
    pub name: String,
    #[schema(example = "1337")]
    pub players_online_count: u32,
    pub location: Location,
    pub pvp_type: PvpType,
    pub battl_eye: bool,
    // #[schema(example = "2014-11-28T12:45:59.324310806Z")]
    pub battl_eye_date: Option<DateTime<Utc>>,
    #[schema(example = false)]
    pub premium_required: bool,
    pub transfer_type: Option<TransferType>,
    pub game_world_type: GameWorldType,
}