use clap::Parser;

#[derive(Parser)]
#[command(
    name = "gazette",
    about = "Personal Repo Summarizer CLI",
    version = "0.0.1"
)]
pub struct Cli {
    #[arg(long)]
    pub help_only: bool,
}
