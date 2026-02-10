use std::fmt;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Duration;
use inquire::{Select, Text};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum AIProvider {
    #[default]
    Gemini,
    OpenAI,
    Anthropic,
    Ollama,
}

impl AIProvider {
    /// Returns all available AI providers
    pub fn all() -> Vec<Self> {
        vec![Self::Gemini, Self::OpenAI, Self::Anthropic, Self::Ollama]
    }

    /// Returns the environment variable name for the API key
    pub fn api_key_env_var(&self) -> &'static str {
        match self {
            Self::Gemini => "GEMINI_API_KEY",
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::Ollama => "OLLAMA_HOST",
        }
    }

    /// Returns a user-friendly prompt for the API key
    pub fn api_key_prompt(&self) -> &'static str {
        match self {
            Self::Gemini => "Enter your Gemini API key:",
            Self::OpenAI => "Enter your OpenAI API key:",
            Self::Anthropic => "Enter your Anthropic API key:",
            Self::Ollama => "Enter your Ollama host (default: http://localhost:11434):",
        }
    }

    /// Returns default value for the credential (if any)
    pub fn default_value(&self) -> Option<&'static str> {
        match self {
            Self::Ollama => Some("http://localhost:11434"),
            _ => None,
        }
    }

    /// Returns available models for this provider
    pub fn available_models(&self) -> Vec<&'static str> {
        match self {
            Self::Gemini => vec![
                "gemini-2.0-flash",
                "gemini-2.0-flash-lite",
                "gemini-1.5-pro",
                "gemini-1.5-flash",
            ],
            Self::OpenAI => vec![
                "gpt-4o",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "gpt-4",
                "gpt-3.5-turbo",
            ],
            Self::Anthropic => vec![
                "claude-sonnet-4-20250514",
                "claude-3-5-sonnet-20241022",
                "claude-3-5-haiku-20241022",
                "claude-3-opus-20240229",
            ],
            Self::Ollama => vec![
                "llama3.2",
                "llama3.1",
                "mistral",
                "codellama",
                "deepseek-coder",
            ],
        }
    }

    /// Returns the default model for this provider
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini-2.0-flash",
            Self::OpenAI => "gpt-4o",
            Self::Anthropic => "claude-sonnet-4-20250514",
            Self::Ollama => "llama3.2",
        }
    }

    /// Returns a short name for display
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Gemini => "Gemini",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Claude",
            Self::Ollama => "Ollama",
        }
    }
}

impl fmt::Display for AIProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gemini => write!(f, "Gemini (Google)"),
            Self::OpenAI => write!(f, "OpenAI (GPT)"),
            Self::Anthropic => write!(f, "Anthropic (Claude)"),
            Self::Ollama => write!(f, "Ollama (Local)"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(tag = "type", content = "value")]
pub enum TimePeriod {
    LastHour,
    Last6Hours,
    Last12Hours,
    #[default]
    Last24Hours,
    Custom {
        seconds: i64,
    },
}

impl TimePeriod {
    /// Returns the duration for this time period
    pub fn to_duration(&self) -> Duration {
        match self {
            Self::LastHour => Duration::hours(1),
            Self::Last6Hours => Duration::hours(6),
            Self::Last12Hours => Duration::hours(12),
            Self::Last24Hours => Duration::hours(24),
            Self::Custom { seconds } => Duration::seconds(*seconds),
        }
    }

    /// Human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::LastHour => "last hour".to_string(),
            Self::Last6Hours => "last 6 hours".to_string(),
            Self::Last12Hours => "last 12 hours".to_string(),
            Self::Last24Hours => "last 24 hours".to_string(),
            Self::Custom { seconds } => {
                let hours = seconds / 3600;
                let mins = (seconds % 3600) / 60;
                let secs = seconds % 60;
                format!("last {:02}:{:02}:{:02}", hours, mins, secs)
            }
        }
    }
}

impl fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LastHour => write!(f, "Last hour"),
            Self::Last6Hours => write!(f, "Last 6 hours"),
            Self::Last12Hours => write!(f, "Last 12 hours"),
            Self::Last24Hours => write!(f, "Last 24 hours"),
            Self::Custom { seconds } => {
                let hours = seconds / 3600;
                let mins = (seconds % 3600) / 60;
                let secs = seconds % 60;
                write!(f, "Custom ({:02}:{:02}:{:02})", hours, mins, secs)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Repo {
    pub owner: String,
    pub name: String,
}

impl Repo {
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// Parses "owner/name" format into a Repo
    pub fn from_full_name(full_name: &str) -> Option<Self> {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub repos: Vec<Repo>,
    #[serde(default)]
    pub time_period: TimePeriod,
    #[serde(default)]
    pub ai_provider: AIProvider,
    #[serde(default)]
    pub ai_model: Option<String>,
}

impl Config {
    /// Returns the AI model, falling back to provider default
    pub fn get_ai_model(&self) -> String {
        self.ai_model
            .clone()
            .unwrap_or_else(|| self.ai_provider.default_model().to_string())
    }
}

impl Config {
    /// Loads config from config.json, migrating from repos.json if needed
    pub fn load() -> Result<Self> {
        let config_path = Path::new(CONFIG_FILE);
        let old_repos_path = Path::new("repos.json");

        // Migrate from old repos.json if it exists
        if !config_path.exists() && old_repos_path.exists() {
            let content =
                fs::read_to_string(old_repos_path).context("Failed to read repos.json")?;
            let repos: Vec<Repo> =
                serde_json::from_str(&content).context("Failed to parse repos.json")?;

            let config = Config {
                repos,
                time_period: TimePeriod::default(),
                ai_provider: AIProvider::default(),
                ai_model: None,
            };
            config.save()?;

            // Remove old file after migration
            fs::remove_file(old_repos_path).ok();

            println!("{}", "Migrated repos.json to config.json".green());
            return Ok(config);
        }

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(config_path).context("Failed to read config.json")?;
        let config: Config =
            serde_json::from_str(&content).context("Failed to parse config.json")?;

        Ok(config)
    }

    /// Saves config to config.json
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(CONFIG_FILE, content).context("Failed to write config.json")?;
        Ok(())
    }
}

pub fn load_repos() -> Result<Vec<Repo>> {
    Ok(Config::load()?.repos)
}

pub fn load_time_period() -> Result<TimePeriod> {
    Ok(Config::load()?.time_period)
}

pub fn subscribe_repo() -> Result<()> {
    let input = Text::new("Repo (owner/name):").prompt()?;

    let repo = Repo::from_full_name(&input)
        .context("Invalid format. Use 'owner/name' (e.g., rust-lang/rust)")?;

    let mut config = Config::load()?;

    // Check if already subscribed
    if config
        .repos
        .iter()
        .any(|r| r.owner == repo.owner && r.name == repo.name)
    {
        println!(
            "{} {}",
            "Already subscribed to".yellow(),
            repo.full_name().cyan()
        );
        return Ok(());
    }

    config.repos.push(repo.clone());
    config.save()?;

    println!("{} {}", "✔ Subscribed to".green(), repo.full_name().cyan());

    Ok(())
}

pub fn unsubscribe_repo() -> Result<()> {
    let mut config = Config::load()?;

    if config.repos.is_empty() {
        println!("{}", "No subscribed repos.".yellow());
        return Ok(());
    }

    let repos = config.repos.clone();
    let selected = Select::new("Select repo to unsubscribe:", repos).prompt()?;

    config
        .repos
        .retain(|r| !(r.owner == selected.owner && r.name == selected.name));

    config.save()?;

    println!(
        "{} {}",
        "✔ Unsubscribed from".green(),
        selected.full_name().cyan()
    );

    Ok(())
}

pub fn list_repos() -> Result<()> {
    let config = Config::load()?;

    if config.repos.is_empty() {
        println!("{}", "No subscribed repos.".yellow());
        return Ok(());
    }

    println!("\n{}", "Subscribed repositories:".underline());
    for repo in &config.repos {
        println!("  {} {}", "•".green(), repo.full_name().cyan());
    }
    println!();

    Ok(())
}

#[derive(Debug, Clone)]
enum TimePeriodOption {
    Preset(TimePeriod),
    Custom,
}

impl fmt::Display for TimePeriodOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preset(period) => write!(f, "{}", period),
            Self::Custom => write!(f, "Custom..."),
        }
    }
}

pub fn configure_time_period() -> Result<()> {
    let config = Config::load()?;

    println!("Current period: {}", config.time_period.to_string().cyan());

    let options = vec![
        TimePeriodOption::Preset(TimePeriod::LastHour),
        TimePeriodOption::Preset(TimePeriod::Last6Hours),
        TimePeriodOption::Preset(TimePeriod::Last12Hours),
        TimePeriodOption::Preset(TimePeriod::Last24Hours),
        TimePeriodOption::Custom,
    ];

    let selection = Select::new("Select time period:", options).prompt()?;

    let new_period = match selection {
        TimePeriodOption::Preset(period) => period,
        TimePeriodOption::Custom => prompt_custom_period()?,
    };

    let mut config = Config::load()?;
    config.time_period = new_period;
    config.save()?;

    println!(
        "{} {}",
        "✔ Time period set to".green(),
        new_period.to_string().cyan()
    );

    Ok(())
}

fn prompt_custom_period() -> Result<TimePeriod> {
    let input = Text::new("Time period (HH:MM:SS):")
        .with_default("01:00:00")
        .with_placeholder("01:30:00")
        .prompt()?;

    let parts: Vec<&str> = input.split(':').collect();

    if parts.len() != 3 {
        anyhow::bail!("Invalid format. Use HH:MM:SS (e.g., 01:30:00)");
    }

    let hours: i64 = parts[0].parse().unwrap_or(0);
    let minutes: i64 = parts[1].parse().unwrap_or(0);
    let secs: i64 = parts[2].parse().unwrap_or(0);

    let total_seconds = hours * 3600 + minutes * 60 + secs;

    if total_seconds <= 0 {
        anyhow::bail!("Time period must be greater than 0");
    }

    Ok(TimePeriod::Custom {
        seconds: total_seconds,
    })
}

pub fn load_ai_provider() -> Result<AIProvider> {
    Ok(Config::load()?.ai_provider)
}

pub fn configure_ai_provider() -> Result<AIProvider> {
    use crate::menu::credentials::ensure_provider_api_key;

    let config = Config::load()?;

    println!(
        "Current AI provider: {}",
        config.ai_provider.to_string().cyan()
    );

    let selection = Select::new("Select AI provider:", AIProvider::all()).prompt()?;

    let mut config = Config::load()?;
    let provider_changed = config.ai_provider != selection;
    config.ai_provider = selection;

    // Reset model when provider changes
    if provider_changed {
        config.ai_model = None;
    }
    config.save()?;

    // Ensure API key is configured for the new provider
    ensure_provider_api_key(selection)?;

    println!(
        "{} {}",
        "✔ AI provider set to".green(),
        selection.to_string().cyan()
    );

    Ok(selection)
}

pub fn configure_ai_model() -> Result<()> {
    let config = Config::load()?;
    let provider = config.ai_provider;

    println!("Current model: {}", config.get_ai_model().cyan());

    let models: Vec<String> = provider
        .available_models()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let selection = Select::new("Select AI model:", models).prompt()?;

    let mut config = Config::load()?;
    config.ai_model = Some(selection.clone());
    config.save()?;

    println!("{} {}", "✔ AI model set to".green(), selection.cyan());

    Ok(())
}
