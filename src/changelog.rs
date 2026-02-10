use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;

use crate::config::{Repo, TimePeriod};
use crate::gemini::GeminiClient;
use crate::github::{GitHubClient, PullRequest};
use crate::jira::{JiraClient, JiraIssue, extract_jira_keys};

/// Aggregated data for a single PR
pub struct PrContext {
    pub pr: PullRequest,
    pub jira_issues: Vec<JiraIssue>,
}

/// Service responsible for generating changelogs
pub struct ChangelogService {
    github: GitHubClient,
    jira: Option<JiraClient>,
    gemini: GeminiClient,
}

impl ChangelogService {
    /// Creates a new changelog service
    /// Jira client is optional - if credentials are missing, Jira context will be skipped
    pub fn new() -> Result<Self> {
        let github = GitHubClient::new()?;
        let gemini = GeminiClient::new()?;

        // Jira is optional
        let jira = JiraClient::new().ok();

        Ok(Self {
            github,
            jira,
            gemini,
        })
    }

    /// Generates a changelog for a single repository
    pub async fn generate_for_repo(&self, repo: &Repo, period: TimePeriod) -> Result<PathBuf> {
        // 1. Fetch merged PRs within the configured period
        let prs = self.github.get_merged_prs(repo, period).await?;

        if prs.is_empty() {
            anyhow::bail!("No PRs merged in the {}", period.description());
        }

        // 2. Fetch Jira context for each PR
        let pr_contexts = self.enrich_with_jira(&prs).await;

        // 3. Aggregate data into text format for Gemini
        let context_text = self.format_pr_context(&pr_contexts);

        // 4. Generate changelog with Gemini
        let changelog = self
            .gemini
            .generate_changelog(&repo.full_name(), &context_text)
            .await?;

        // 5. Save to file
        let path = self.save_changelog(repo, &changelog)?;

        Ok(path)
    }

    /// Enriches PRs with Jira context
    async fn enrich_with_jira(&self, prs: &[PullRequest]) -> Vec<PrContext> {
        let mut contexts = Vec::new();

        for pr in prs {
            let mut jira_issues = Vec::new();

            // Extract Jira keys from title and body
            let mut all_keys = extract_jira_keys(&pr.title);
            if let Some(body) = &pr.body {
                all_keys.extend(extract_jira_keys(body));
            }

            // Deduplicate keys
            all_keys.sort();
            all_keys.dedup();

            // Fetch Jira issues if client is available
            if let Some(jira) = &self.jira {
                for key in all_keys {
                    match jira.get_issue(&key).await {
                        Ok(Some(issue)) => jira_issues.push(issue),
                        Ok(None) => {} // Issue not found, skip
                        Err(_) => {}   // API error, skip
                    }
                }
            }

            contexts.push(PrContext {
                pr: PullRequest {
                    number: pr.number,
                    title: pr.title.clone(),
                    body: pr.body.clone(),
                    merged_at: pr.merged_at,
                    user: None, // We don't need user for context
                    html_url: pr.html_url.clone(),
                },
                jira_issues,
            });
        }

        contexts
    }

    /// Formats PR contexts as text for Gemini
    fn format_pr_context(&self, contexts: &[PrContext]) -> String {
        let mut output = String::new();

        for ctx in contexts {
            output.push_str(&format!("## PR #{}: {}\n", ctx.pr.number, ctx.pr.title));
            output.push_str(&format!("URL: {}\n", ctx.pr.html_url));

            if let Some(merged) = ctx.pr.merged_at {
                output.push_str(&format!(
                    "Merged at: {}\n",
                    merged.format("%Y-%m-%d %H:%M UTC")
                ));
            }

            if let Some(body) = &ctx.pr.body {
                if !body.trim().is_empty() {
                    output.push_str(&format!("Description:\n{}\n", body));
                }
            }

            if !ctx.jira_issues.is_empty() {
                output.push_str("\nJira Context:\n");
                for issue in &ctx.jira_issues {
                    output.push_str(&format!("- {}: {}\n", issue.key, issue.fields.summary));
                    if let Some(status) = &issue.fields.status {
                        output.push_str(&format!("  Status: {}\n", status.name));
                    }
                    if let Some(desc) = issue.description_text() {
                        if !desc.trim().is_empty() {
                            let truncated: String = desc.chars().take(500).collect();
                            output.push_str(&format!("  Details: {}\n", truncated));
                        }
                    }
                }
            }

            output.push_str("\n---\n\n");
        }

        output
    }

    /// Saves the changelog to a file and returns the path
    fn save_changelog(&self, repo: &Repo, content: &str) -> Result<PathBuf> {
        let date = Local::now().format("%Y-%m-%d");
        let filename = format!("changelog_{}_{}.md", repo.name, date);

        let path = PathBuf::from(&filename);

        fs::write(&path, content).context("Failed to write changelog file")?;

        Ok(path)
    }
}
