use std::{collections::HashMap, time::Duration};

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::models::ResidenceType;

const COMMUNITY_URL: &str = "https://www.tibia.com/community/";

#[derive(Debug, Clone)]
pub struct TibiaClient {
    client: ClientWithMiddleware,
}

impl TibiaClient {
    pub fn new() -> Self {
        let reqwest_client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/113.0",
        )
        .brotli(true)
        .deflate(true)
        .gzip(true)
        .pool_idle_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create reqwest client");

        let client = reqwest_middleware::ClientBuilder::new(reqwest_client)
            .with(Cache(HttpCache {
                // Figure out how to use cache even though tibia sends incorrect cache headers
                mode: CacheMode::NoStore,
                manager: CACacheManager::default(),
                options: HttpCacheOptions::default(),
            }))
            .build();

        Self { client }
    }
}

impl TibiaClient {
    #[instrument(skip(self))]
    pub async fn fetch_worlds_page(&self) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "worlds");
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }
    #[instrument(skip(self))]
    pub async fn fetch_towns_page(&self) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "houses");

        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;
        Ok(response)
    }

    #[instrument(skip(self))]
    pub async fn fetch_world_details_page(
        &self,
        world_name: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "worlds");
        params.insert("world", world_name);
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    pub async fn fetch_guilds_page(
        &self,
        world_name: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "guilds");
        params.insert("world", world_name);
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    pub async fn fetch_killstatistics_page(
        &self,
        world_name: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "killstatistics");
        params.insert("world", world_name);
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    pub async fn fetch_residences_page(
        &self,
        world_name: &str,
        residence_type: &ResidenceType,
        town: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "houses");
        params.insert("world", world_name);
        params.insert("town", town);
        let residence_string = match residence_type {
            ResidenceType::House => "houses",
            ResidenceType::Guildhall => "guildhalls",
        };
        params.insert("type", residence_string);
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }
}
