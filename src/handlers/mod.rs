const KILL_STATISTICS_URL: &'static str =
    "https://www.tibia.com/community/?subtopic=killstatistics";
pub mod worlds {
    use std::collections::HashMap;

    use axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    };
    use serde::{Deserialize, Serialize};

    use crate::AppState;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PathParams {
        world: String,
    }

    #[axum::debug_handler]
    pub async fn get_kill_statistics(
        State(state): State<AppState>,
        Path(path_params): Path<PathParams>,
    ) -> Response {
        let client = state.client;

        let world = path_params.world;

        println!("{world}");
        // Form data
        let mut params = HashMap::new();
        params.insert("world", "Adra");

        let response = client
            .post(super::KILL_STATISTICS_URL)
            .form(&params)
            .send()
            .await
            .unwrap();

        let page_as_str = response.text().await.unwrap();
        println!("{page_as_str}");

        let stats = tibia_api::scrape_kill_statistics(&page_as_str);

        let json = Json(stats);
        json.into_response()
    }
}
