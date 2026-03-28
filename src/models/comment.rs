use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CommentType {
    #[default]
    Turn,
    Game,
}

impl CommentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Turn => "turn",
            Self::Game => "game",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "game" => Self::Game,
            _ => Self::Turn,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    #[serde(rename = "type")]
    pub comment_type: CommentType,
    pub content: String,
}

impl Comment {
    pub fn new(comment_type: CommentType, content: impl Into<String>) -> Self {
        Self {
            id: 0,
            comment_type,
            content: content.into(),
        }
    }
}

impl Default for Comment {
    fn default() -> Self {
        Self::new(CommentType::Turn, "")
    }
}
