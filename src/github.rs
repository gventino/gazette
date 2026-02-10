use std::env;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;

use crate::config::{Repo, TimePeriod};

const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";

/// GitHub API client
pub struct GitHubClient {
    client: reqwest::Client,
}

/// Represents a Pull Request from GitHub API
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub merged_at: Option<DateTime<Utc>>,
    pub user: Option<GitHubUser>,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
}

impl GitHubClient {
    /// Creates a new GitHub client using GITHUB_TOKEN from environment
    pub fn new() -> Result<Self> {
        let token = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not found in environment")?;

        Self::with_token(&token)
    }

    /// Creates a new GitHub client with a specific token
    pub fn with_token(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).context("Invalid token format")?,
        );

        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static(GITHUB_API_VERSION),
        );

        headers.insert(USER_AGENT, HeaderValue::from_static("gazette-rs-cli"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Fetches merged PRs within the specified time period
    pub async fn get_merged_prs(
        &self,
        repo: &Repo,
        period: TimePeriod,
    ) -> Result<Vec<PullRequest>> {
        let url = format!(
            "{}/repos/{}/{}/pulls",
            GITHUB_API_URL, repo.owner, repo.name
        );

        let response = self
            .client
            .get(&url)
            .query(&[
                ("state", "closed"),
                ("sort", "updated"),
                ("direction", "desc"),
                ("per_page", "100"),
            ])
            .send()
            .await
            .context("Failed to fetch PRs from GitHub")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, body);
        }

        let prs: Vec<PullRequest> = response
            .json()
            .await
            .context("Failed to parse GitHub PR response")?;

        let cutoff = Utc::now() - period.to_duration();

        let merged_prs: Vec<PullRequest> = prs
            .into_iter()
            .filter(|pr| pr.merged_at.map(|merged| merged > cutoff).unwrap_or(false))
            .collect();

        Ok(merged_prs)
    }
}
