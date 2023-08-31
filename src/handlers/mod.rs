pub mod redocly;

// Handlers

/// /__healthcheck
pub mod __healthcheck;
/// /characters/:character_name
pub mod characters_character_name;
/// /towns
pub mod towns;
/// /worlds
pub mod worlds;
/// /worlds/:world_name
pub mod worlds_world_name;
/// /worlds/:world_name/guilds
pub mod worlds_world_name_guilds;
/// /worlds/:world_name/kill-statistics
pub mod worlds_world_name_kill_statistics;
/// /worlds/:world_name/residences
pub mod worlds_world_name_residences;
