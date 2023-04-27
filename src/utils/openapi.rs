use crate::error::{ClientError, ClientErrorCode};
use crate::v1;
use utoipa::openapi::{self, InfoBuilder};
use utoipa::OpenApi;

pub fn create_openapi_docs() -> openapi::OpenApi {
    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "https://tibia.ankarhem.dev"),
        ),
        paths(
            v1::worlds::get_worlds::handler,
            v1::worlds::get_world::handler,
            v1::worlds::get_world_kill_statistics::handler,
            v1::worlds::get_world_guilds::handler
        ),
        components(schemas(
            ClientErrorCode,
            ClientError,
            v1::worlds::get_worlds::WorldsData,
            v1::worlds::get_worlds::World,
            v1::worlds::get_worlds::GameWorldType,
            v1::worlds::get_worlds::TransferType,
            v1::worlds::get_worlds::Location,
            v1::worlds::get_worlds::PvpType,
            v1::worlds::get_world::Player,
            v1::worlds::get_world::Vocation,
            v1::worlds::get_world::WorldDetails,
            v1::worlds::get_world_kill_statistics::KillStatistics,
            v1::worlds::get_world_kill_statistics::RaceKillStatistics,
            v1::worlds::get_world_kill_statistics::KilledAmounts,
            v1::worlds::get_world_guilds::Guild,
        )),
        tags((name = "Worlds", description = "World related endpoints"))
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

const API_DESCRIPTION: &'static str = r#"
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
