use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurrentSession {
    #[serde(default, skip_serializing, skip_deserializing)]
    pub id: i64,
    #[serde(default)]
    pub game_uuid: Option<Uuid>,
    pub game_name: String,
    pub turn_number: i32,
}

impl CurrentSession {
    pub fn new(game_uuid: Option<Uuid>, game_name: impl Into<String>, turn_number: i32) -> Self {
        Self {
            id: 0,
            game_uuid,
            game_name: game_name.into(),
            turn_number,
        }
    }
}

impl Default for CurrentSession {
    fn default() -> Self {
        Self::new(None, "", 1)
    }
}
