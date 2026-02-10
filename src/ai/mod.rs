mod anthropic;
mod gemini;
mod ollama;
mod openai;

use std::env;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::AIProvider;

pub use anthropic::AnthropicClient;
pub use gemini::GeminiClient;
pub use ollama::OllamaClient;
pub use openai::OpenAIClient;

/// Common trait for all AI providers
#[async_trait]
pub trait AIClient: Send + Sync {
    /// Generates text from a prompt
    async fn generate(&self, prompt: &str) -> Result<String>;

    /// Generates a changelog markdown from PR data
    async fn generate_changelog(
        &self,
        repo_name: &str,
        prs_context: &str,
        time_period: &str,
    ) -> Result<String> {
        let prompt = format!(
            r#"You are a technical writer. Generate a concise markdown changelog for the repository "{repo_name}" based on the following Pull Request information merged in the {time_period}.

The changelog should:
- Have a header with the repository name and today's date
- Group changes by category (Features, Bug Fixes, Improvements, etc.) if applicable
- Be concise but informative
- Include PR numbers as clickable markdown links using the provided URLs (e.g., [#123](url))
- If Jira context is available, include the Jira ticket ID as a clickable markdown link using the provided Jira URL (e.g., [SSD-1234](jira_url))

PR Information:
{prs_context}

Generate only the markdown content, with short explanation about each change."#
        );

        self.generate(&prompt).await
    }
}

/// Creates an AI client based on the configured provider
pub fn create_ai_client(provider: AIProvider, model: &str) -> Result<Box<dyn AIClient>> {
    match provider {
        AIProvider::Gemini => {
            let client = GeminiClient::new(model)?;
            Ok(Box::new(client))
        }
        AIProvider::OpenAI => {
            let client = OpenAIClient::new(model)?;
            Ok(Box::new(client))
        }
        AIProvider::Anthropic => {
            let client = AnthropicClient::new(model)?;
            Ok(Box::new(client))
        }
        AIProvider::Ollama => {
            let client = OllamaClient::new(model)?;
            Ok(Box::new(client))
        }
    }
}

/// Checks if the API key for the given provider is configured
#[allow(dead_code)]
pub fn is_provider_configured(provider: AIProvider) -> bool {
    let env_var = provider.api_key_env_var();
    env::var(env_var).map(|v| !v.is_empty()).unwrap_or(false)
}
