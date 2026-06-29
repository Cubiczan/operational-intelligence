use oi_llm::{LlmCompletion, TokenUsage};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UsageClientError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api error: {0}")]
    Api(String),
}

#[derive(Debug, Clone)]
pub struct TechEconomistClient {
    base_url: String,
    workflow_id: Option<i32>,
    source: String,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct UsageIngestPayload<'a> {
    session_id: &'a str,
    source: &'a str,
    model: &'a str,
    provider: &'a str,
    usage: TokenUsage,
    workflow_id: Option<i32>,
    agent_id: Option<&'a str>,
}

impl TechEconomistClient {
    pub fn new(base_url: impl Into<String>, workflow_id: Option<i32>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            workflow_id,
            source: "operational-intelligence".into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("OI_TECH_ECONOMIST_URL").ok()?;
        let workflow_id = std::env::var("OI_TECH_ECONOMIST_WORKFLOW_ID")
            .ok()
            .and_then(|v| v.parse().ok());
        Some(Self::new(base_url, workflow_id))
    }

    pub async fn ingest_completion(
        &self,
        session_id: &str,
        agent_id: Option<&str>,
        completion: &LlmCompletion,
    ) -> Result<(), UsageClientError> {
        let payload = UsageIngestPayload {
            session_id,
            source: &self.source,
            model: &completion.model,
            provider: "canonical",
            usage: completion.usage.clone(),
            workflow_id: self.workflow_id,
            agent_id,
        };

        let resp = self
            .http
            .post(format!("{}/api/usage-ingest", self.base_url))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(UsageClientError::Api(format!("{status}: {body}")));
        }
        Ok(())
    }
}
