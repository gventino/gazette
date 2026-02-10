use std::env;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::AIClient;

const DEFAULT_HOST: &str = "http://localhost:11434";

/// Ollama API client for local models
pub struct OllamaClient {
    client: reqwest::Client,
    host: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: Option<String>,
    error: Option<String>,
}

impl OllamaClient {
    /// Creates a new Ollama client
    /// Uses OLLAMA_HOST environment variable or defaults to localhost:11434
    pub fn new(model: &str) -> Result<Self> {
        let host = env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());

        Ok(Self {
            client: reqwest::Client::new(),
            host,
            model: model.to_string(),
        })
    }
}

#[async_trait]
impl AIClient for OllamaClient {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.host);

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama. Is Ollama running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error ({}): {}", status, body);
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        if let Some(error) = ollama_response.error {
            anyhow::bail!("Ollama error: {}", error);
        }

        let text = ollama_response.response.unwrap_or_default();

        Ok(text)
    }
}
