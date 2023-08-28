use crate::handlers;
use crate::models::*;
use crate::prelude::*;
use utoipa::openapi::{self, InfoBuilder};
use utoipa::OpenApi;

pub fn create_openapi_docs() -> openapi::OpenApi {
    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "https://tibia.ankarhem.dev"),
        ),
        paths(
            handlers::towns::get,
            handlers::worlds::get,
            handlers::worlds_world_name::get,
            handlers::worlds_world_name_guilds::get,
            handlers::worlds_world_name_kill_statistics::get,
            handlers::worlds_world_name_residences::get,
        ),
        components(schemas(
            PublicErrorBody,
            WorldDetails,
            WorldsResponse,
            GameWorldType,
            Location,
            Player,
            Vocation,
            PvpType,
            TransferType,
            World,
            Guild,
            KillStatistics,
            KilledAmounts,
            RaceKillStatistics,
            Residence,
            ResidenceType,
            ResidenceStatus,
        )),
        tags()
    )]
    struct ApiDocV1;
    let mut openapi = ApiDocV1::openapi();
    openapi.info = InfoBuilder::new()
        .title("Tibia API")
        .description(Some(API_DESCRIPTION))
        .version("1.0.0")
        .build();

    openapi
}

const API_DESCRIPTION: &str = r#"
<div style="display: flex; align-items: center; gap: 2rem;">
<img src="/favicon.png" alt="Sorcerer asset" width="150" height="150">
<h1 style="margin: 0; font-size: 2.5rem;">Tibia API</h1>
</div>

This is a helper API for grabbing the data available on the [Tibia](https://www.tibia.com/) website, written in [Rust](https://www.rust-lang.org/). It is primarily a way for me to test out Rust and its ecosystem, but feel free to use it.

The source code is available on [GitHub](https://github.com/ankarhem/tibia-api-rust).

Contact me at [jakob@ankarhem.dev](mailto:jakob@ankarhem.dev), or raise an [issue](https://github.com/ankarhem/tibia-api-rust/issues).

<h2>Disclaimer</h2>

The data is based on [tibia.com](https://www.tibia.com/), the only official Tibia website.

Tibia is a registered trademark of [CipSoft GmbH](https://www.cipsoft.com/en/). Tibia and all products related to Tibia are copyright by [CipSoft GmbH](https://www.cipsoft.com/en/).
"#;
