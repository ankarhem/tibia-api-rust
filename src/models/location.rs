use anyhow::{anyhow, Result};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum Location {
    Europe,
    SouthAmerica,
    NorthAmerica,
}

impl std::str::FromStr for Location {
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
