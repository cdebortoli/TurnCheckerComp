use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CheckRepeatType {
    #[default]
    Everytime,
    Conditional(i32),
    Specific(i32),
    Until(i32),
}

impl CheckRepeatType {
    pub fn to_storage(&self) -> (&'static str, Option<i32>) {
        match self {
            Self::Everytime => ("everytime", None),
            Self::Conditional(value) => ("conditional", Some(*value)),
            Self::Specific(value) => ("specific", Some(*value)),
            Self::Until(value) => ("until", Some(*value)),
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
mod tests {
    use super::CheckRepeatType;

    #[test]
    fn repeat_type_to_storage() {
        assert_eq!(CheckRepeatType::Everytime.to_storage(), ("everytime", None));
        assert_eq!(
            CheckRepeatType::Conditional(3).to_storage(),
            ("conditional", Some(3))
        );
        assert_eq!(CheckRepeatType::Specific(7).to_storage(), ("specific", Some(7)));
        assert_eq!(CheckRepeatType::Until(4).to_storage(), ("until", Some(4)));
    }

    #[test]
    fn repeat_type_from_storage() {
        assert_eq!(
            CheckRepeatType::from_storage("conditional", Some(3)),
            CheckRepeatType::Conditional(3)
        );
        assert_eq!(
            CheckRepeatType::from_storage("specific", Some(7)),
            CheckRepeatType::Specific(7)
        );
        assert_eq!(
            CheckRepeatType::from_storage("until", Some(4)),
            CheckRepeatType::Until(4)
        );
        assert_eq!(
            CheckRepeatType::from_storage("unknown", None),
            CheckRepeatType::Everytime
        );
    }
}
