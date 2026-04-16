use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CheckRepeatType {
    #[default]
    Everytime,
    Conditional(i32),
    Specific(i32),
    Until(i32),
}

impl CheckRepeatType {
    pub fn to_storage(self) -> (&'static str, Option<i32>) {
        match self {
            Self::Everytime => ("everytime", None),
            Self::Conditional(value) => ("conditional", Some(value)),
            Self::Specific(value) => ("specific", Some(value)),
            Self::Until(value) => ("until", Some(value)),
        }
    }

    pub fn from_storage(kind: &str, value: Option<i32>) -> Self {
        match kind {
            "conditional" => Self::Conditional(value.unwrap_or(1)),
            "specific" => Self::Specific(value.unwrap_or(1)),
            "until" => Self::Until(value.unwrap_or(1)),
            _ => Self::Everytime,
        }
    }
}

#[cfg(test)]
#[path = "check_repeat_type_tests.rs"]
mod tests;
