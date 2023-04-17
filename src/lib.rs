use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct KillStatistics {
    killed_players: i32,
    killed_by_players: i32,
}

#[derive(Serialize)]
pub struct MonsterStats {
    name: String,
    last_day: KillStatistics,
    last_week: KillStatistics,
}

pub fn scrape_kill_statistics(page: &str) -> Vec<MonsterStats> {
    let ks = KillStatistics {
        killed_players: 12,
        killed_by_players: 138,
    };
    let stat = MonsterStats {
        name: "Demon".to_string(),
        last_day: ks.clone(),
        last_week: ks,
    };
    vec![stat]
}
