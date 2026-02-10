use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use inquire::{Confirm, Select, Text};
use owo_colors::OwoColorize;

const ENV_FILE: &str = ".env";

#[derive(Debug, Clone, Copy)]
pub enum CredentialsOption {
    UpdateGithubToken,
    UpdateGeminiApiKey,
    UpdateJiraCredentials,
    Back,
}

impl fmt::Display for CredentialsOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpdateGithubToken => write!(f, "Update GitHub token"),
            Self::UpdateGeminiApiKey => write!(f, "Update Gemini API key"),
            Self::UpdateJiraCredentials => write!(f, "Update Jira credentials"),
            Self::Back => write!(f, "Back to main menu"),
        }
    }
}

impl CredentialsOption {
    pub fn all() -> Vec<Self> {
        vec![
            Self::UpdateGithubToken,
            Self::UpdateGeminiApiKey,
            Self::UpdateJiraCredentials,
            Self::Back,
        ]
    }
}

pub fn menu_credentials() -> Result<()> {
    let ans = Select::new("Select credential to update:", CredentialsOption::all()).prompt()?;

    match ans {
        CredentialsOption::UpdateGithubToken => {
            update_github_token()?;
            println!("{}", "✔ GitHub token updated successfully!".green());
        }
        CredentialsOption::UpdateGeminiApiKey => {
            update_gemini_api_key()?;
            println!("{}", "✔ Gemini API key updated successfully!".green());
        }
        CredentialsOption::UpdateJiraCredentials => {
            update_jira_credentials()?;
            println!("{}", "✔ Jira credentials updated successfully!".green());
        }
        CredentialsOption::Back => return Ok(()),
    }
    Ok(())
}

/// Loads all required credentials at startup
pub fn load_all_credentials() -> Result<()> {
    // GitHub token (required)
    load_env_var("GITHUB_TOKEN", "Enter your GitHub token:", true)?;
    println!("{}", "✔ GitHub token loaded".green());

    // Gemini API key (required)
    load_env_var("GEMINI_API_KEY", "Enter your Gemini API key:", true)?;
    println!("{}", "✔ Gemini API key loaded".green());

    // Jira credentials (optional)
    load_jira_credentials()?;

    Ok(())
}

fn load_jira_credentials() -> Result<()> {
    let has_jira = env::var("JIRA_URL").is_ok()
        && env::var("JIRA_EMAIL").is_ok()
        && env::var("JIRA_API_TOKEN").is_ok();

    if has_jira {
        println!("{}", "✔ Jira credentials loaded".green());
        return Ok(());
    }

    // Ask if user wants to configure Jira
    println!(
        "{}",
        "Jira credentials not found (optional for ticket context).".yellow()
    );
    let configure = Confirm::new("Would you like to configure Jira integration?")
        .with_default(false)
        .prompt()?;

    if configure {
        prompt_jira_credentials()?;
        println!("{}", "✔ Jira credentials saved".green());
    } else {
        println!("{}", "Skipping Jira integration.".dimmed());
    }

    Ok(())
}

fn prompt_jira_credentials() -> Result<()> {
    let url = Text::new("Jira URL (e.g., https://company.atlassian.net):").prompt()?;
    let email = Text::new("Jira email:").prompt()?;
    let token = Text::new("Jira API token:").prompt()?;

    save_env_var("JIRA_URL", &url)?;
    save_env_var("JIRA_EMAIL", &email)?;
    save_env_var("JIRA_API_TOKEN", &token)?;

    Ok(())
}

// ===== Token Management =====

fn load_env_var(key: &str, prompt_msg: &str, required: bool) -> Result<Option<String>> {
    if let Ok(value) = env::var(key)
        && !value.is_empty()
    {
        return Ok(Some(value));
    }

    if !required {
        return Ok(None);
    }

    println!("{} not found.", key.yellow());
    let value = Text::new(prompt_msg).prompt()?;
    save_env_var(key, &value)?;

    Ok(Some(value))
}

pub fn update_github_token() -> Result<()> {
    let token = Text::new("Enter your new GitHub token:").prompt()?;
    save_env_var("GITHUB_TOKEN", &token)?;
    Ok(())
}

pub fn update_gemini_api_key() -> Result<()> {
    let key = Text::new("Enter your new Gemini API key:").prompt()?;
    save_env_var("GEMINI_API_KEY", &key)?;
    Ok(())
}

pub fn update_jira_credentials() -> Result<()> {
    prompt_jira_credentials()
}

fn save_env_var(key: &str, value: &str) -> Result<()> {
    let env_path = Path::new(ENV_FILE);

    if env_path.exists() {
        // Read existing content and update/add the key
        let content = fs::read_to_string(env_path)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(env_path)?;

        // Remove existing key if present
        let new_content: String = content
            .lines()
            .filter(|line| !line.starts_with(&format!("{}=", key)))
            .collect::<Vec<_>>()
            .join("\n");

        if new_content.is_empty() {
            writeln!(file, "{}={}", key, value)?;
        } else {
            writeln!(file, "{}\n{}={}", new_content, key, value)?;
        }
    } else {
        // Create new .env file
        let mut file = fs::File::create(env_path)?;
        writeln!(file, "{}={}", key, value)?;
    }

    // Reload .env to make new variable available in current process
    dotenvy::dotenv().ok();

    Ok(())
}
