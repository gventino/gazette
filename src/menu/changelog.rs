use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use futures::future::join_all;
use inquire::Select;
use owo_colors::OwoColorize;

use crate::changelog::ChangelogService;
use crate::config::{Repo, load_repos, load_time_period};

#[derive(Debug, Clone, Copy)]
pub enum ChangelogOption {
    SingleRepo,
    AllRepos,
    Back,
}

impl fmt::Display for ChangelogOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleRepo => write!(f, "Generate changelog for a single repo"),
            Self::AllRepos => write!(f, "Generate changelog for all subscribed repos"),
            Self::Back => write!(f, "Back to main menu"),
        }
    }
}

impl ChangelogOption {
    pub fn all() -> Vec<Self> {
        vec![Self::SingleRepo, Self::AllRepos, Self::Back]
    }
}

/// Wrapper for repo selection with a Back option
#[derive(Debug, Clone)]
enum RepoSelection {
    Repo(Repo),
    Back,
}

impl fmt::Display for RepoSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repo(repo) => write!(f, "{}", repo.full_name()),
            Self::Back => write!(f, "← Back"),
        }
    }
}

pub async fn menu_changelog() -> Result<()> {
    let ans = Select::new("Generation type:", ChangelogOption::all()).prompt()?;

    match ans {
        ChangelogOption::SingleRepo => {
            let repos = load_repos()?;

            if repos.is_empty() {
                println!(
                    "{}",
                    "No subscribed repos. Subscribe to a repo first.".yellow()
                );
                return Ok(());
            }

            let mut options: Vec<RepoSelection> =
                repos.into_iter().map(RepoSelection::Repo).collect();
            options.push(RepoSelection::Back);

            let selection = Select::new("Select a repo:", options).prompt()?;

            match selection {
                RepoSelection::Repo(repo) => generate_changelog_single(&repo).await?,
                RepoSelection::Back => return Ok(()),
            }
        }
        ChangelogOption::AllRepos => {
            println!("{}", "Generating full report...".italic());
            generate_changelog_all().await?;
        }
        ChangelogOption::Back => return Ok(()),
    }
    Ok(())
}

async fn generate_changelog_single(repo: &Repo) -> Result<()> {
    let period = load_time_period()?;

    println!(
        "{} {}",
        "Generating changelog for".cyan(),
        repo.full_name().yellow()
    );

    println!(
        "{}",
        format!("  → Fetching merged PRs from {}...", period.description()).dimmed()
    );

    let service = ChangelogService::new()?;

    match service.generate_for_repo(repo, period).await {
        Ok(path) => {
            println!(
                "\n{} {}",
                "✔ Changelog saved to:".green().bold(),
                path.display().to_string().cyan()
            );
        }
        Err(e) => {
            println!("{} {}", "✖ Error:".red().bold(), e);
        }
    }

    Ok(())
}

async fn generate_changelog_all() -> Result<()> {
    let repos = load_repos()?;
    let period = load_time_period()?;

    if repos.is_empty() {
        println!("{}", "No subscribed repos.".yellow());
        return Ok(());
    }

    println!(
        "{} {} repos in parallel...",
        "Processing".cyan(),
        repos.len().to_string().yellow()
    );

    let service = Arc::new(ChangelogService::new()?);

    // Create futures for all repos
    let futures: Vec<_> = repos
        .into_iter()
        .map(|repo| {
            let service = Arc::clone(&service);
            async move {
                let result = service.generate_for_repo(&repo, period).await;
                (repo, result)
            }
        })
        .collect();

    // Execute all in parallel
    let results = join_all(futures).await;

    // Print results
    println!();
    for (repo, result) in results {
        match result {
            Ok(path) => {
                println!(
                    "{} {} → {}",
                    "✔".green(),
                    repo.full_name().cyan(),
                    path.display().to_string().dimmed()
                );
            }
            Err(e) => {
                println!("{} {} → {}", "✖".red(), repo.full_name().cyan(), e);
            }
        }
    }

    Ok(())
}
