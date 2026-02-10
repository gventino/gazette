use std::env;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::AIClient;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Anthropic API client
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Option<Vec<ContentBlock>>,
    error: Option<AnthropicError>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicError {
    message: String,
}

impl AnthropicClient {
    /// Creates a new Anthropic client from environment variable ANTHROPIC_API_KEY
    pub fn new(model: &str) -> Result<Self> {
        let api_key =
            env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY not found in environment")?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model: model.to_string(),
        })
    }
}

#[async_trait]
impl AIClient for AnthropicClient {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error ({}): {}", status, body);
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic response")?;

        if let Some(error) = anthropic_response.error {
            anyhow::bail!("Anthropic API error: {}", error.message);
        }

        let text = anthropic_response
            .content
            .map(|blocks| {
                blocks
                    .into_iter()
                    .filter(|b| b.content_type == "text")
                    .filter_map(|b| b.text)
                    .collect::<Vec<_>>()
                    .concat()
            })
            .unwrap_or_default();

        Ok(text)
    }
}
