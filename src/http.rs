use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct HttpClient {
    agent: ureq::Agent,
}

impl HttpClient {
    pub fn new(user_agent: String) -> Self {
        let config = ureq::Agent::config_builder().user_agent(user_agent).build();
        let agent = config.into();
        Self { agent }
    }

    pub fn json<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.agent
            .get(url)
            .call()
            .with_context(|| format!("GET {url} failed"))?
            .body_mut()
            .read_json()
            .with_context(|| format!("failed to parse JSON from {url}"))
    }

    pub fn text(&self, url: &str) -> Result<String> {
        self.agent
            .get(url)
            .call()
            .with_context(|| format!("GET {url} failed"))?
            .body_mut()
            .read_to_string()
            .with_context(|| format!("failed to read text from {url}"))
    }

    pub fn bytes(&self, url: &str) -> Result<Vec<u8>> {
        self.agent
            .get(url)
            .call()
            .with_context(|| format!("GET {url} failed"))?
            .body_mut()
            .read_to_vec()
            .with_context(|| format!("failed to read bytes from {url}"))
    }
}
