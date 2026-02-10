use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum MainMenuOption {
    Subscribe,
    Unsubscribe,
    ListRepos,
    ConfigureTimePeriod,
    ChangeAIProvider,
    ChangeAIModel,
    GenerateChangelog,
    UpdateCredentials,
    Exit,
}

impl fmt::Display for MainMenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subscribe => write!(f, "Subscribe to a repo"),
            Self::Unsubscribe => write!(f, "Unsubscribe from a repo"),
            Self::ListRepos => write!(f, "List subscribed repos"),
            Self::ConfigureTimePeriod => write!(f, "Configure time period"),
            Self::ChangeAIProvider => write!(f, "Change AI provider"),
            Self::ChangeAIModel => write!(f, "Change AI model"),
            Self::GenerateChangelog => write!(f, "Generate changelog"),
            Self::UpdateCredentials => write!(f, "Update credentials"),
            Self::Exit => write!(f, "Exit"),
        }
    }
}

impl MainMenuOption {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Subscribe,
            Self::Unsubscribe,
            Self::ListRepos,
            Self::ConfigureTimePeriod,
            Self::ChangeAIProvider,
            Self::ChangeAIModel,
            Self::GenerateChangelog,
            Self::UpdateCredentials,
            Self::Exit,
        ]
    }
}
