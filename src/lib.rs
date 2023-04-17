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
    let document = scraper::Html::parse_document(&page);

    let table_cell_selector =
        scraper::Selector::parse("#KillStatisticsTable tr.DataRow > td").unwrap();
    let cells = document
        .select(&table_cell_selector)
        .map(|cell| cell.inner_html())
        .collect::<Vec<String>>();

    let mut iter = cells.iter();

    let mut stats: Vec<MonsterStats> = vec![];
    while let (Some(name), Some(kp_day), Some(kbp_day), Some(kp_week), Some(kbp_week)) = (
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
    ) {
        let last_day = KillStatistics {
            killed_players: kp_day.parse().unwrap(),
            killed_by_players: kbp_day.parse().unwrap(),
        };

        let last_week = KillStatistics {
            killed_players: kp_week.parse().unwrap(),
            killed_by_players: kbp_week.parse().unwrap(),
        };

        stats.push(MonsterStats {
            name: name.to_string(),
            last_day,
            last_week,
        })
    }

    stats
}
