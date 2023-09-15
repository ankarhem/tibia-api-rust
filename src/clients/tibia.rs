use std::{collections::HashMap, time::Duration};

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::models::ResidenceType;

const COMMUNITY_URL: &str = "https://www.tibia.com/community/";

#[derive(Debug, Clone)]
pub struct TibiaClient<S: HttpSend = Sender> {
    client: ClientWithMiddleware,
    sender: S,
}
pub trait HttpSend {
    async fn send(
        &self,
        request: reqwest_middleware::RequestBuilder,
    ) -> Result<reqwest::Response, reqwest_middleware::Error>;
}

#[derive(Debug, Clone)]
pub struct Sender;
impl HttpSend for Sender {
    async fn send(
        &self,
        request: reqwest_middleware::RequestBuilder,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        request.send().await
    }
}

impl TibiaClient<Sender> {
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

        Self {
            client,
            sender: Sender,
        }
    }
}

impl Default for TibiaClient<Sender> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: HttpSend> TibiaClient<S> {
    pub fn with_sender(sender: S) -> TibiaClient<S> {
        TibiaClient {
            client: TibiaClient::new().client,
            sender,
        }
    }

    #[instrument(skip(self))]
    pub async fn fetch_worlds_page(&self) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "worlds");
        let request = self.client.get(COMMUNITY_URL).query(&params);
        let response = self.sender.send(request).await?;

        Ok(response)
    }
    #[instrument(skip(self))]
    pub async fn fetch_towns_page(&self) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let mut params = HashMap::new();
        params.insert("subtopic", "houses");

        let request = self.client.get(COMMUNITY_URL).query(&params);
        let response = self.sender.send(request).await?;
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
        let request = self.client.get(COMMUNITY_URL).query(&params);
        let response = self.sender.send(request).await?;

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
        let request = self.client.get(COMMUNITY_URL).query(&params);
        let response = self.sender.send(request).await?;

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
        let request = self.client.get(COMMUNITY_URL).query(&params);
        let response = self.sender.send(request).await?;

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
        let request = self.client.get(COMMUNITY_URL).query(&params);
        let response = self.sender.send(request).await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpSend, TibiaClient};
    use http::response;

    #[derive(Default)]
    pub struct MockSender {
        status: reqwest::StatusCode,
        body: &'static str,
    }

    impl MockSender {
        pub fn new(status: reqwest::StatusCode, body: &'static str) -> Self {
            Self { status, body }
        }

        pub fn status(self, status: reqwest::StatusCode) -> Self {
            Self { status, ..self }
        }

        pub fn body(self, body: &'static str) -> Self {
            Self { body, ..self }
        }
    }

    impl TibiaClient<MockSender> {
        pub fn mocked_client(
            status: reqwest::StatusCode,
            body: &'static str,
        ) -> TibiaClient<MockSender> {
            TibiaClient::with_sender(MockSender::new(status, body))
        }
    }

    impl HttpSend for MockSender {
        async fn send(
            &self,
            _: reqwest_middleware::RequestBuilder,
        ) -> Result<reqwest::Response, reqwest_middleware::Error> {
            let response = response::Builder::new()
                .status(self.status)
                .body(self.body)
                .expect("Could not construct mocked response");
            Ok(response.into())
        }
    }
}
