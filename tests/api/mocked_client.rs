use http::response;
use tibia_api::{clients::Client, models::ResidenceType};

#[derive(Clone)]
pub struct MockedClient {
    status: reqwest::StatusCode,
    body: Option<String>,
}

impl MockedClient {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn status(self, status: reqwest::StatusCode) -> Self {
        Self { status, ..self }
    }
    pub fn body(self, body: &str) -> Self {
        Self {
            body: Some(body.into()),
            ..self
        }
    }

    fn mocked(&self) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let body = self.body.clone().unwrap_or_default();
        let response = response::Response::builder()
            .status(self.status)
            .body(body)
            .unwrap()
            .into();

        Ok(response)
    }
}

impl Default for MockedClient {
    fn default() -> Self {
        Self {
            status: reqwest::StatusCode::OK,
            body: None,
        }
    }
}

#[async_trait::async_trait]
impl Client for MockedClient {
    async fn fetch_towns_page(&self) -> Result<reqwest::Response, reqwest_middleware::Error> {
        self.mocked()
    }

    async fn fetch_worlds_page(&self) -> Result<reqwest::Response, reqwest_middleware::Error> {
        self.mocked()
    }

    async fn fetch_world_details_page(
        &self,
        _world_name: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        self.mocked()
    }

    async fn fetch_guilds_page(
        &self,
        _world_name: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        self.mocked()
    }

    async fn fetch_killstatistics_page(
        &self,
        _world_name: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        self.mocked()
    }

    async fn fetch_residences_page(
        &self,
        _world_name: &str,
        _residence_type: &ResidenceType,
        _town: &str,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        self.mocked()
    }
}
