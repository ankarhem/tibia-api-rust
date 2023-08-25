use anyhow::{anyhow, Result};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TransferType {
    Blocked,
    Locked,
}

impl std::str::FromStr for TransferType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let string = s.to_string();
        match string.as_str() {
            "blocked" => Ok(TransferType::Blocked),
            "locked" => Ok(TransferType::Locked),
            _ => Err(anyhow!("Unexpected transfer type: '{}''", s)),
        }
    }
}
