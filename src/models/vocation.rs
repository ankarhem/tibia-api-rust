use anyhow::{anyhow, Result};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum Vocation {
    Knight,
    EliteKnight,
    Sorcerer,
    MasterSorcerer,
    Druid,
    ElderDruid,
    Paladin,
    RoyalPaladin,
}

impl std::str::FromStr for Vocation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Knight" => Ok(Vocation::Knight),
            "Elite Knight" => Ok(Vocation::EliteKnight),
            "Sorcerer" => Ok(Vocation::Sorcerer),
            "Master Sorcerer" => Ok(Vocation::MasterSorcerer),
            "Druid" => Ok(Vocation::Druid),
            "Elder Druid" => Ok(Vocation::ElderDruid),
            "Paladin" => Ok(Vocation::Paladin),
            "Royal Paladin" => Ok(Vocation::RoyalPaladin),
            _ => Err(anyhow!("Unexpected vocation: '{}''", s)),
        }
    }
}
