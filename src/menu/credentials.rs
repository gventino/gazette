use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use inquire::{Confirm, Select, Text};
use owo_colors::OwoColorize;

use crate::config::{AIProvider, Config, configure_ai_model};

const ENV_FILE: &str = ".env";

#[derive(Debug, Clone, Copy)]
pub enum CredentialsOption {
    UpdateGithubToken,
    UpdateAIProvider,
    UpdateAIModel,
    UpdateAIApiKey,
    UpdateJiraCredentials,
    Back,
}

impl fmt::Display for CredentialsOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpdateGithubToken => write!(f, "Update GitHub token"),
            Self::UpdateAIProvider => write!(f, "Change AI provider"),
            Self::UpdateAIModel => write!(f, "Change AI model"),
            Self::UpdateAIApiKey => write!(f, "Update AI API key"),
            Self::UpdateJiraCredentials => write!(f, "Update Jira credentials"),
            Self::Back => write!(f, "Back to main menu"),
        }
    }
}

impl CredentialsOption {
    pub fn all() -> Vec<Self> {
        vec![
            Self::UpdateGithubToken,
            Self::UpdateAIProvider,
            Self::UpdateAIModel,
            Self::UpdateAIApiKey,
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
        CredentialsOption::UpdateAIProvider => {
            let provider = select_ai_provider()?;
            prompt_ai_api_key(provider)?;
            println!("{}", "✔ AI provider updated successfully!".green());
        }
        CredentialsOption::UpdateAIModel => {
            configure_ai_model()?;
        }
        CredentialsOption::UpdateAIApiKey => {
            let config = Config::load()?;
            prompt_ai_api_key(config.ai_provider)?;
            println!("{}", "✔ AI API key updated successfully!".green());
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

    // AI provider and API key
    load_ai_credentials()?;

    // Jira credentials (optional)
    load_jira_credentials()?;

    Ok(())
}

fn load_ai_credentials() -> Result<()> {
    let mut config = Config::load()?;
    let provider = config.ai_provider;
    let env_var = provider.api_key_env_var();

    // Check if we need to select a provider (first run or missing API key)
    let has_api_key = env::var(env_var).map(|v| !v.is_empty()).unwrap_or(false);

    if !has_api_key {
        println!(
            "{}",
            "AI provider not configured. Let's set it up!".yellow()
        );

        // Ask user to select AI provider
        let selected_provider = select_ai_provider()?;
        config.ai_provider = selected_provider;
        config.save()?;

        // Prompt for API key
        prompt_ai_api_key(selected_provider)?;
    }

    println!(
        "{} {}",
        "✔ AI provider:".green(),
        config.ai_provider.to_string().cyan()
    );

    Ok(())
}

fn select_ai_provider() -> Result<AIProvider> {
    let selection = Select::new("Select AI provider:", AIProvider::all()).prompt()?;

    let mut config = Config::load()?;
    config.ai_provider = selection;
    config.save()?;

    Ok(selection)
}

fn prompt_ai_api_key(provider: AIProvider) -> Result<()> {
    let env_var = provider.api_key_env_var();
    let prompt = provider.api_key_prompt();

    let value = if let Some(default) = provider.default_value() {
        Text::new(prompt).with_default(default).prompt()?
    } else {
        Text::new(prompt).prompt()?
    };

    save_env_var(env_var, &value)?;
    Ok(())
}

/// Ensures the API key for the given provider is configured
/// If not configured, prompts the user to enter it
pub fn ensure_provider_api_key(provider: AIProvider) -> Result<()> {
    let env_var = provider.api_key_env_var();
    let has_api_key = env::var(env_var).map(|v| !v.is_empty()).unwrap_or(false);

    if !has_api_key {
        println!("{}", format!("{} not configured.", env_var).yellow());
        prompt_ai_api_key(provider)?;
        println!("{}", "✔ API key saved".green());
    }

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
