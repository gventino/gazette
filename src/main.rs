mod changelog;
mod cli;
pub mod config;
pub mod gemini;
pub mod github;
pub mod jira;
mod menu;

use std::io::{Write, stdout};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use inquire::Select;
use owo_colors::OwoColorize;

use cli::Cli;
use config::{configure_time_period, list_repos, subscribe_repo, unsubscribe_repo};
use menu::{MainMenuOption, credentials, menu_changelog, menu_credentials};

#[tokio::main]
async fn main() -> Result<()> {
    let _args = Cli::parse();

    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    // Load or request all credentials
    credentials::load_all_credentials()?;

    run_main_loop().await
}

fn clear_screen() {
    let _ = execute!(stdout(), Clear(ClearType::All));
    // Move cursor to top-left
    print!("\x1B[H");
    let _ = stdout().flush();
}

fn print_banner() {
    println!(
        "{}",
        "
     ▗▄▄▖ ▗▄▖ ▗▄▄▄▄▖▗▄▄▄▖▗▄▄▄▖▗▄▄▄▖▗▄▄▄▖
    ▐▌   ▐▌ ▐▌   ▗▞▘▐▌     █    █  ▐▌   
    ▐▌▝▜▌▐▛▀▜▌ ▗▞▘  ▐▛▀▀▘  █    █  ▐▛▀▀▘
    ▝▚▄▞▘▐▌ ▐▌▐▙▄▄▄▖▐▙▄▄▖  █    █  ▐▙▄▄▖                      
    "
        .green()
        .bold()
    );

    println!("{}", "--- Welcome to Gazette CLI ---".cyan().bold());
}

async fn run_main_loop() -> Result<()> {
    loop {
        clear_screen();
        print_banner();

        let ans = Select::new("Choose an option:", MainMenuOption::all()).prompt()?;

        clear_screen();
        print_banner();

        match ans {
            MainMenuOption::Subscribe => subscribe_repo()?,
            MainMenuOption::Unsubscribe => unsubscribe_repo()?,
            MainMenuOption::ListRepos => list_repos()?,
            MainMenuOption::ConfigureTimePeriod => configure_time_period()?,
            MainMenuOption::GenerateChangelog => menu_changelog().await?,
            MainMenuOption::UpdateCredentials => menu_credentials()?,
            MainMenuOption::Exit => {
                clear_screen();
                println!("Goodbye!");
                break;
            }
        }

        // Wait for user to press Enter before returning to menu
        println!("\n{}", "Press Enter to continue...".dimmed());
        let _ = std::io::stdin().read_line(&mut String::new());
    }

    Ok(())
}
