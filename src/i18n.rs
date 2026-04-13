use std::{borrow::Cow, collections::HashMap};

use fluent_templates::{
    fluent_bundle::{types::FluentNumber, FluentValue},
    static_loader, Loader,
};
use unic_langid::LanguageIdentifier;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
    };
}

#[derive(Clone)]
pub struct I18n {
    language: LanguageIdentifier,
}

#[derive(Clone)]
pub enum I18nValue {
    String(String),
    Number(i64),
}

impl I18n {
    pub fn system() -> Self {
        Self {
            language: detect_system_locale(),
        }
    }

    // For tests
    #[allow(dead_code)]
    pub fn from_language(language: &str) -> Self {
        Self {
            language: language.parse().expect("valid locale"),
        }
    }

    pub fn t(&self, key: &str) -> String {
        LOCALES.lookup(&self.language, key)
    }

    pub fn tr(&self, key: &str, args: &[(&str, I18nValue)]) -> String {
        let mut fluent_args: HashMap<Cow<'static, str>, FluentValue<'static>> = HashMap::new();

        for (name, value) in args {
            fluent_args.insert(
                Cow::Owned((*name).to_string()),
                value.clone().into_fluent_value(),
            );
        }

        LOCALES.lookup_with_args(&self.language, key, &fluent_args)
    }
}

impl I18nValue {
    fn into_fluent_value(self) -> FluentValue<'static> {
        match self {
            Self::String(value) => value.into(),
            Self::Number(value) => FluentNumber::from(value).into(),
        }
    }
}

impl From<&str> for I18nValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for I18nValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for I18nValue {
    fn from(value: &String) -> Self {
        Self::String(value.clone())
    }
}

impl From<i32> for I18nValue {
    fn from(value: i32) -> Self {
        Self::Number(value as i64)
    }
}

impl From<i64> for I18nValue {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<usize> for I18nValue {
    fn from(value: usize) -> Self {
        Self::Number(value as i64)
    }
}

fn detect_system_locale() -> LanguageIdentifier {
    let raw_locale = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());
    let normalized = raw_locale.replace('_', "-").to_ascii_lowercase();

    if normalized.starts_with("fr") {
        "fr-FR".parse().expect("valid fallback locale")
    } else {
        "en-US".parse().expect("valid fallback locale")
    }
}
