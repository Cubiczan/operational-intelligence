pub mod client;
pub mod metered;

pub use client::TechEconomistClient;
pub use metered::MeteredLlm;
use oi_llm::MockLlm;
use std::sync::Arc;

/// Wraps the default mock LLM with Tech Economist ingest when `OI_TECH_ECONOMIST_URL` is set.
pub fn build_metered_llm(session_id: impl Into<String>) -> Arc<dyn oi_llm::LlmProvider> {
    let base: Arc<dyn oi_llm::LlmProvider> = Arc::new(MockLlm);
    match TechEconomistClient::from_env() {
        Some(client) => Arc::new(MeteredLlm::new(
            base,
            Arc::new(client),
            session_id.into(),
        )),
        None => base,
    }
}
