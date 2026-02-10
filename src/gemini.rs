use std::env;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const DEFAULT_MODEL: &str = "gemini-2.0-flash";

/// Gemini API client
pub struct GeminiClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<CandidatePart>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: String,
}

impl GeminiClient {
    /// Creates a new Gemini client from environment variable GEMINI_API_KEY
    pub fn new() -> Result<Self> {
        let api_key =
            env::var("GEMINI_API_KEY").context("GEMINI_API_KEY not found in environment")?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model: DEFAULT_MODEL.to_string(),
        })
    }

    /// Generates text from a prompt
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_URL, self.model, self.api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Gemini API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error ({}): {}", status, body);
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        let text = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .map(|c| {
                c.content
                    .parts
                    .into_iter()
                    .map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        Ok(text)
    }

    /// Generates a changelog markdown from PR data
    pub async fn generate_changelog(&self, repo_name: &str, prs_context: &str) -> Result<String> {
        let prompt = format!(
            r#"You are a technical writer. Generate a concise markdown changelog for the repository "{repo_name}" based on the following Pull Request information merged in the last 24 hours.

The changelog should:
- Have a header with the repository name and today's date
- Group changes by category (Features, Bug Fixes, Improvements, etc.) if applicable
- Be concise but informative
- Include PR numbers as clickable markdown links using the provided URLs (e.g., [#123](url))
- If Jira context is available, mention the ticket purpose briefly

PR Information:
{prs_context}

Generate only the markdown content, no explanations."#
        );

        self.generate(&prompt).await
    }
}
