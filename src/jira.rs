use std::env;

use anyhow::{Context, Result};
use regex::Regex;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;

/// Jira API client
pub struct JiraClient {
    client: reqwest::Client,
    base_url: String,
}

/// Represents a Jira issue
#[derive(Debug, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraFields,
}

#[derive(Debug, Deserialize)]
pub struct JiraFields {
    pub summary: String,
    pub description: Option<JiraDescription>,
    pub status: Option<JiraStatus>,
    pub issuetype: Option<JiraIssueType>,
}

#[derive(Debug, Deserialize)]
pub struct JiraDescription {
    pub content: Option<Vec<JiraContent>>,
}

#[derive(Debug, Deserialize)]
pub struct JiraContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub content: Option<Vec<JiraTextContent>>,
}

#[derive(Debug, Deserialize)]
pub struct JiraTextContent {
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraIssueType {
    pub name: String,
}

impl JiraClient {
    /// Creates a new Jira client from environment variables
    /// Requires: JIRA_URL (e.g., https://company.atlassian.net)
    ///           JIRA_EMAIL and JIRA_API_TOKEN
    pub fn new() -> Result<Self> {
        let base_url = env::var("JIRA_URL").context("JIRA_URL not found in environment")?;
        let email = env::var("JIRA_EMAIL").context("JIRA_EMAIL not found in environment")?;
        let api_token =
            env::var("JIRA_API_TOKEN").context("JIRA_API_TOKEN not found in environment")?;

        Self::with_credentials(&base_url, &email, &api_token)
    }

    /// Creates a new Jira client with explicit credentials
    pub fn with_credentials(base_url: &str, email: &str, api_token: &str) -> Result<Self> {
        use base64::Engine;
        let auth =
            base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", email, api_token));

        let mut headers = HeaderMap::new();

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", auth))
                .context("Invalid credentials format")?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Fetches a Jira issue by key (e.g., "PROJECT-123")
    /// Returns None if the issue doesn't exist
    pub async fn get_issue(&self, issue_key: &str) -> Result<Option<JiraIssue>> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, issue_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch Jira issue")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Jira API error ({}): {}", status, body);
        }

        let issue: JiraIssue = response
            .json()
            .await
            .context("Failed to parse Jira issue response")?;

        Ok(Some(issue))
    }
}

/// Extracts Jira issue keys from text (e.g., "PROJECT-123")
/// Returns all matches found in the text
pub fn extract_jira_keys(text: &str) -> Vec<String> {
    let re = Regex::new(r"[A-Z][A-Z0-9]+-\d+").expect("Invalid regex");
    re.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

impl JiraIssue {
    /// Extracts plain text description from Jira's ADF format
    pub fn description_text(&self) -> Option<String> {
        self.fields.description.as_ref().and_then(|desc| {
            desc.content.as_ref().map(|contents| {
                contents
                    .iter()
                    .filter_map(|c| {
                        c.content.as_ref().map(|texts| {
                            texts
                                .iter()
                                .filter_map(|t| t.text.clone())
                                .collect::<Vec<_>>()
                                .join("")
                        })
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_jira_keys() {
        let text = "feat(PROJECT-123): implement feature [TEAM-456]";
        let keys = extract_jira_keys(text);
        assert_eq!(keys, vec!["PROJECT-123", "TEAM-456"]);
    }

    #[test]
    fn test_extract_jira_keys_no_match() {
        let text = "fix: some bug without ticket";
        let keys = extract_jira_keys(text);
        assert!(keys.is_empty());
    }
}
