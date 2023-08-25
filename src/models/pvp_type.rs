use anyhow::{anyhow, Result};
use serde::Serialize;
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

impl std::str::FromStr for PvpType {
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
