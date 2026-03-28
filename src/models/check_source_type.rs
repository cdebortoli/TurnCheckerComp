use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CheckSourceType {
    #[default]
    Game,
    GlobalGame,
    Blueprint,
    Turn,
}

impl CheckSourceType {
    pub fn to_storage(&self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::GlobalGame => "globalGame",
            Self::Blueprint => "blueprint",
            Self::Turn => "turn",
        }
    }

    pub fn from_storage(kind: &str) -> Self {
        match kind {
            "game" => Self::Game,
            "globalGame" => Self::GlobalGame,
            "blueprint" => Self::Blueprint,
            _ => Self::Turn,
        }
    }
}
