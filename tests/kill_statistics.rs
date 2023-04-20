#[test]
fn can_scrape_kill_statistics() {
    let html = include_str!("./kill_statistics.html");
    tibia_api::scrape_kill_statistics(html).unwrap();
}
