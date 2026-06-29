use crate::client::TechEconomistClient;
use async_trait::async_trait;
use oi_llm::{LlmCompletion, LlmError, LlmProvider, LlmRequest};
use std::sync::Arc;

pub struct MeteredLlm {
    inner: Arc<dyn LlmProvider>,
    client: Arc<TechEconomistClient>,
    session_id: String,
}

impl MeteredLlm {
    pub fn new(
        inner: Arc<dyn LlmProvider>,
        client: Arc<TechEconomistClient>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            client,
            session_id: session_id.into(),
        }
    }
}

#[async_trait]
impl LlmProvider for MeteredLlm {
    async fn complete(&self, request: LlmRequest) -> Result<LlmCompletion, LlmError> {
        let agent_id = request.agent_id.clone();
        let completion = self.inner.complete(request).await?;
        if let Err(err) = self
            .client
            .ingest_completion(&self.session_id, agent_id.as_deref(), &completion)
            .await
        {
            tracing::warn!("tech-economist usage ingest failed: {err}");
        }
        Ok(completion)
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }
}
