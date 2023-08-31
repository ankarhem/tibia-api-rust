use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::Vocation;

/// The character sex
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum Sex {
    Male,
    Female,
}

/// Houses owned by a character
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct House {
    /// The "houseId" of the residence
    pub id: u32,
    /// The name of the house
    pub name: String,
    /// The date until which the rent is paid for
    #[schema(value_type = Option<String>, format = Date)]
    pub paid_until: NaiveDate,
    /// The town where the house is located
    pub town: String,
}

/// The character's guild
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GuildMember {
    #[schema(example = "King")]
    pub role: String,
    #[schema(example = "Hill")]
    pub guild_name: String,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CharacterInfo {
    #[schema(example = "Glada Pojken")]
    pub name: String,
    /// Names this character has formerly gone by
    #[schema(example = json!(["Max Lurifax", "Top Minimum"]))]
    pub former_names: Option<Vec<String>>,
    pub title: Option<String>, // TODO: list unlocked titles?
    pub sex: Sex,
    pub vocation: Option<Vocation>,
    /// The character's level
    pub level: u16,
    /// The number of achievement points the character has
    pub achievement_points: u16,
    /// The town where the character spawns (Residence)
    pub spawn_point: String,
    /// The world the character is on
    pub world: String,
    /// The residences which the character owns
    pub houses: Option<Vec<House>>,
    pub guild: Option<GuildMember>,
    /// The time the character last logged in
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_login: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    /// Whether the account has premium or not
    pub has_premium: bool,
}
