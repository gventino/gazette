# Gazette CLI

<p align="center">
  <img src="https://img.shields.io/badge/rust-2024-orange" alt="Rust 2024">
  <img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

**Gazette** is a CLI tool that automatically generates AI-powered changelogs from GitHub Pull Requests, enriched with Jira context.

```
     ▗▄▄▖ ▗▄▖ ▗▄▄▄▄▖▗▄▄▄▖▗▄▄▄▖▗▄▄▄▖▗▄▄▄▖
    ▐▌   ▐▌ ▐▌   ▗▞▘▐▌     █    █  ▐▌   
    ▐▌▝▜▌▐▛▀▜▌ ▗▞▘  ▐▛▀▀▘  █    █  ▐▛▀▀▘
    ▝▚▄▞▘▐▌ ▐▌▐▙▄▄▄▖▐▙▄▄▖  █    █  ▐▙▄▄▖
```

## Features

- 📋 **GitHub Integration** — Fetches merged PRs from your repositories
- 🎫 **Jira Context** — Automatically extracts and enriches changelogs with Jira ticket information
- 🤖 **AI-Powered** — Uses Google Gemini to generate concise, well-structured changelogs
- ⏱️ **Configurable Time Periods** — Filter PRs by last hour, 6h, 12h, 24h, or custom periods
- 📦 **Repository Subscriptions** — Subscribe to multiple repos and generate changelogs in batch
- 🔐 **Credential Management** — Securely stores API tokens in a local `.env` file

## Installation

### Quick Install (macOS/Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/gventino/gazette/main/install.sh | bash
```

### From Source

Requires [Rust](https://rustup.rs/) (edition 2024).

```bash
git clone https://github.com/your-username/gazette.git
cd gazette
cargo build --release
cp target/release/gazette /usr/local/bin/
```

### Manual Download

Download the latest binary from the [Releases](https://github.com/your-username/gazette/releases) page and add it to your PATH.

## Configuration

On first run, Gazette will prompt you for the required API credentials:

| Credential | Required | Description |
|------------|----------|-------------|
| `GITHUB_TOKEN` | ✅ | GitHub Personal Access Token with `repo` scope |
| `GEMINI_API_KEY` | ✅ | Google Gemini API key |
| `JIRA_BASE_URL` | ❌ | Your Jira instance URL (e.g., `https://company.atlassian.net`) |
| `JIRA_EMAIL` | ❌ | Jira account email |
| `JIRA_API_TOKEN` | ❌ | Jira API token |

Credentials are stored in a `.env` file in your working directory.

### Getting API Keys

#### GitHub Token
1. Go to [GitHub Settings → Developer settings → Personal access tokens](https://github.com/settings/tokens)
2. Generate a new token with `repo` scope

#### Gemini API Key
1. Visit [Google AI Studio](https://aistudio.google.com/app/apikey)
2. Create a new API key

#### Jira API Token (Optional)
1. Go to [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens)
2. Create a new API token

## Usage

Run the CLI:

```bash
gazette
```

### Main Menu Options

| Option | Description |
|--------|-------------|
| **Subscribe to a repo** | Add a repository to track (format: `owner/name`) |
| **Unsubscribe from a repo** | Remove a repository from tracking |
| **List subscribed repos** | Show all tracked repositories |
| **Configure time period** | Set the time window for PR filtering |
| **Generate changelog** | Create a changelog for one or all subscribed repos |
| **Update credentials** | Modify stored API tokens |

### Time Period Options

- Last hour
- Last 6 hours
- Last 12 hours
- Last 24 hours (default)
- Custom (format: `HH:MM:SS`)

### Output

Changelogs are saved as Markdown files in the current directory:

```
changelog_<repo-name>_<date>.md
```

#### Example Output

```markdown
# Changelog for acme/backend - 2026-02-10

## Features
- Add user authentication via OAuth2 ([#142](https://github.com/acme/backend/pull/142))
- Implement rate limiting for API endpoints ([#138](https://github.com/acme/backend/pull/138))

## Bug Fixes
- Fix memory leak in connection pool ([#141](https://github.com/acme/backend/pull/141))

## Improvements
- Refactor database queries for better performance ([#139](https://github.com/acme/backend/pull/139))
```

## Configuration File

Gazette stores repository subscriptions and settings in `config.json`:

```json
{
  "repos": [
    { "owner": "acme", "name": "backend" },
    { "owner": "acme", "name": "frontend" }
  ],
  "time_period": { "type": "Last24Hours" }
}
```

## Dependencies

- [clap](https://crates.io/crates/clap) — Command-line argument parsing
- [inquire](https://crates.io/crates/inquire) — Interactive prompts
- [reqwest](https://crates.io/crates/reqwest) — HTTP client
- [tokio](https://crates.io/crates/tokio) — Async runtime
- [serde](https://crates.io/crates/serde) — Serialization
- [chrono](https://crates.io/crates/chrono) — Date/time handling
- [crossterm](https://crates.io/crates/crossterm) — Terminal manipulation
- [owo-colors](https://crates.io/crates/owo-colors) — Terminal colors

## Development

```bash
# Run in development
cargo run

# Run tests
cargo test

# Build release binary
cargo build --release

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT License — see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

---

Made with ❤️ and 🦀