use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::ClientWithMiddleware;
use std::{collections::HashMap, time::Duration};
use tracing::instrument;

use crate::{models::ResidenceType, prelude::error_chain_fmt};

const COMMUNITY_URL: &str = "https://www.tibia.com/community/";

#[derive(Debug, Clone)]
pub struct TibiaClient {
    client: ClientWithMiddleware,
}

#[derive(thiserror::Error)]
pub enum TibiaError {
    #[error("Tibia is currently undergoing maintenance")]
    Maintenance,
    #[error("The content on the page is not what was requested")]
    NotFound,
    #[error("Getting rate limited")]
    UnsuccessfulRequest(reqwest::StatusCode),
    #[error(transparent)]
    Reqwest(#[from] reqwest_middleware::Error),
}

impl std::fmt::Debug for TibiaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub const MAINTENANCE_TITLE: &str =
    "Tibia - Free Multiplayer Online Role Playing Game - Maintenance";

#[async_trait::async_trait]
pub trait Client: Send + Sync + Clone + 'static {
    async fn fetch_towns_page(&self) -> Result<reqwest::Response, TibiaError>;
    async fn fetch_worlds_page(&self) -> Result<reqwest::Response, TibiaError>;
    async fn fetch_world_details_page(
        &self,
        world_name: &str,
    ) -> Result<reqwest::Response, TibiaError>;
    async fn fetch_guilds_page(&self, world_name: &str) -> Result<reqwest::Response, TibiaError>;
    async fn fetch_killstatistics_page(
        &self,
        world_name: &str,
    ) -> Result<reqwest::Response, TibiaError>;
    async fn fetch_residences_page(
        &self,
        world_name: &str,
        residence_type: &ResidenceType,
        town: &str,
    ) -> Result<reqwest::Response, TibiaError>;
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

impl Default for TibiaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Client for TibiaClient {
    #[instrument(skip(self))]
    async fn fetch_worlds_page(&self) -> Result<reqwest::Response, TibiaError> {
        let mut params = HashMap::new();
        params.insert("subtopic", "worlds");
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        if response.status().as_u16() > 399 {
            return Err(TibiaError::UnsuccessfulRequest(response.status()))?;
        }

        Ok(response)
    }

    #[instrument(skip(self))]
    async fn fetch_towns_page(&self) -> Result<reqwest::Response, TibiaError> {
        let mut params = HashMap::new();
        params.insert("subtopic", "houses");

        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;
        Ok(response)
    }

    #[instrument(skip(self))]
    async fn fetch_world_details_page(
        &self,
        world_name: &str,
    ) -> Result<reqwest::Response, TibiaError> {
        let mut params = HashMap::new();
        params.insert("subtopic", "worlds");
        params.insert("world", world_name);
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    async fn fetch_guilds_page(&self, world_name: &str) -> Result<reqwest::Response, TibiaError> {
        let mut params = HashMap::new();
        params.insert("subtopic", "guilds");
        params.insert("world", world_name);
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    async fn fetch_killstatistics_page(
        &self,
        world_name: &str,
    ) -> Result<reqwest::Response, TibiaError> {
        let mut params = HashMap::new();
        params.insert("subtopic", "killstatistics");
        params.insert("world", world_name);
        let response = self.client.get(COMMUNITY_URL).query(&params).send().await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    async fn fetch_residences_page(
        &self,
        world_name: &str,
        residence_type: &ResidenceType,
        town: &str,
    ) -> Result<reqwest::Response, TibiaError> {
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
