use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tag {
    #[serde(skip_serializing)]
    pub id: i64,
    pub uuid: Uuid,
    pub name: String,
    pub color: String,
    pub text_color: String,
    pub is_sent: bool,
}

impl Tag {
    pub fn new(
        name: impl Into<String>,
        color: impl Into<String>,
        text_color: impl Into<String>,
    ) -> Self {
        Self {
            id: 0,
            uuid: Uuid::new_v4(),
            name: name.into(),
            color: color.into(),
            text_color: text_color.into(),
            is_sent: false,
        }
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self::new("", "#000000", "#FFFFFF")
    }
}
