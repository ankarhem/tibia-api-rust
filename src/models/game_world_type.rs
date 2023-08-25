use anyhow::Result;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameWorldType {
    Regular,
    Experimental,
}

impl std::str::FromStr for GameWorldType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "experimental" => Ok(GameWorldType::Experimental),
            _ => Ok(GameWorldType::Regular),
        }
    }
}
