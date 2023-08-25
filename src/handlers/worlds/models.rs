use anyhow::{anyhow, Result};
use chrono::prelude::*;
use serde::Serialize;
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum PvpType {
    Open,
    Optional,
    Hardcore,
    RetroOpen,
    RetroHardcore,
}

impl FromStr for PvpType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let string = s.to_string();
        match string.as_str() {
            "Open PvP" => Ok(PvpType::Open),
            "Optional PvP" => Ok(PvpType::Optional),
            "Hardcore PvP" => Ok(PvpType::Hardcore),
            "Retro Open PvP" => Ok(PvpType::RetroOpen),
            "Retro Hardcore PvP" => Ok(PvpType::RetroHardcore),
            _ => Err(anyhow!("Unexpected pvp type: '{}''", s)),
        }
    }
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum Location {
    Europe,
    SouthAmerica,
    NorthAmerica,
}

impl FromStr for Location {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let string = s.to_string();
        match string.as_str() {
            "Europe" => Ok(Location::Europe),
            "North America" => Ok(Location::NorthAmerica),
            "South America" => Ok(Location::SouthAmerica),
            _ => Err(anyhow!("Unexpected location: '{}'", s)),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameWorldType {
    Regular,
    Experimental,
}

impl FromStr for GameWorldType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "experimental" => Ok(GameWorldType::Experimental),
            _ => Ok(GameWorldType::Regular),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TransferType {
    Blocked,
    Locked,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct World {
    #[schema(example = "Antica")]
    pub(super) name: String,
    #[schema(example = "1337")]
    pub(super) players_online_count: u32,
    pub(super) location: Location,
    pub(super) pvp_type: PvpType,
    pub(super) battl_eye: bool,
    // #[schema(example = "2014-11-28T12:45:59.324310806Z")]
    pub(super) battl_eye_date: Option<DateTime<Utc>>,
    #[schema(example = false)]
    pub(super) premium_required: bool,
    pub(super) transfer_type: Option<TransferType>,
    pub(super) game_world_type: GameWorldType,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldsResponse {
    pub(super) players_online_total: u32,
    pub(super) record_players: u32,
    // #[schema(example = "2014-11-28T12:45:59.324310806Z")]
    pub(super) record_date: DateTime<Utc>,
    pub(super) worlds: Vec<World>,
}
