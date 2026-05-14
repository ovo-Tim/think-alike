use crate::{config::Config, error::AppError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct EmbeddingClient {
    client: Client,
    config: Arc<Config>,
}

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingDatum>,
}

#[derive(Deserialize)]
struct EmbeddingDatum {
    embedding: Vec<f32>,
}

impl EmbeddingClient {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn embed_text(&self, title: &str, description: &str) -> Result<Vec<f32>, AppError> {
        let input = format!("Title: {title}\nDescription: {description}");
        let response = self
            .client
            .post(&self.config.openai_embedding_url)
            .bearer_auth(&self.config.openai_api_key)
            .json(&EmbeddingRequest {
                model: &self.config.openai_embedding_model,
                input: &input,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Upstream(format!(
                "embedding request failed with {status}: {body}"
            )));
        }

        let body: EmbeddingResponse = response.json().await?;
        body.data
            .into_iter()
            .next()
            .map(|datum| datum.embedding)
            .ok_or_else(|| AppError::Upstream("embedding API returned no vectors".to_string()))
    }
}
