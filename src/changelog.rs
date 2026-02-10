use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;

use crate::ai::{self, AIClient};
use crate::config::{Config, Repo, TimePeriod};
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
    ai_client: Box<dyn AIClient>,
}

impl ChangelogService {
    /// Creates a new changelog service
    /// Jira client is optional - if credentials are missing, Jira context will be skipped
    pub fn new() -> Result<Self> {
        let github = GitHubClient::new()?;

        // Load AI provider and model from config
        let config = Config::load()?;
        let model = config.get_ai_model();
        let ai_client = ai::create_ai_client(config.ai_provider, &model)?;

        // Jira is optional
        let jira = JiraClient::new().ok();

        Ok(Self {
            github,
            jira,
            ai_client,
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

        // 3. Aggregate data into text format for AI
        let context_text = self.format_pr_context(&pr_contexts);

        // 4. Generate changelog with AI
        let changelog = self
            .ai_client
            .generate_changelog(&repo.full_name(), &context_text, &period.description())
            .await?;

        // Validate AI output to avoid silently writing empty changelog files
        if changelog.trim().is_empty() {
            anyhow::bail!("AI-generated changelog is empty; please try again or check the AI provider configuration");
        }
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

    /// Formats PR contexts as text for AI
    fn format_pr_context(&self, contexts: &[PrContext]) -> String {
        let jira_base_url = std::env::var("JIRA_URL").ok();
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

            if let Some(body) = &ctx.pr.body
                && !body.trim().is_empty()
            {
                output.push_str(&format!("Description:\n{}\n", body));
            }

            if !ctx.jira_issues.is_empty() {
                output.push_str("\nJira Context:\n");
                for issue in &ctx.jira_issues {
                    let jira_url = jira_base_url
                        .as_ref()
                        .map(|base| format!("{}/browse/{}", base.trim_end_matches('/'), issue.key));
                    if let Some(url) = jira_url {
                        output.push_str(&format!(
                            "- {} ({}): {}\n",
                            issue.key, url, issue.fields.summary
                        ));
                    } else {
                        output.push_str(&format!(
                            "- {}: {}\n",
                            issue.key, issue.fields.summary
                        ));
                    }
                    if let Some(status) = &issue.fields.status {
                        output.push_str(&format!("  Status: {}\n", status.name));
                    }
                    if let Some(desc) = issue.description_text()
                        && !desc.trim().is_empty()
                    {
                        let truncated: String = desc.chars().take(500).collect();
                        output.push_str(&format!("  Details: {}\n", truncated));
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
