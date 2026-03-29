use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{check_source_type::CheckSourceType, CheckRepeatType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub id: i64,
    pub uuid: Uuid,
    pub name: String,
    pub detail: Option<String>,
    pub source: CheckSourceType,
    #[serde(rename = "repeatCase")]
    pub repeat_case: CheckRepeatType,
    pub tag_uuid: Option<Uuid>,
    pub position: i32,
    pub is_mandatory: bool,
    pub is_checked: bool,
    pub is_sent: bool,
}

impl Check {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            uuid: Uuid::new_v4(),
            name: name.into(),
            detail: None,
            source: CheckSourceType::default(),
            repeat_case: CheckRepeatType::default(),
            tag_uuid: None,
            position: 0,
            is_mandatory: false,
            is_checked: false,
            is_sent: false,
        }
    }
}

impl Default for Check {
    fn default() -> Self {
        Self::new("")
    }
}
