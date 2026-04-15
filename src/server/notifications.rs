// This file stores the paired device token and sends placeholder push notification requests.
use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use serde::Serialize;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::i18n::{I18n, I18nValue};

const PUSH_NOTIFICATION_URL: &str = "https://turn-checker-apns-relay.debortolichris.workers.dev/";
const PUSH_NOTIFICATION_BEARER_TOKEN_ENV: &str = "TURN_CHECKER_PUSH_BEARER_TOKEN";
const PUSH_NOTIFICATION_BEARER_TOKEN_FILE_ENV: &str = "TURN_CHECKER_PUSH_BEARER_TOKEN_FILE";
const PUSH_NOTIFICATION_BEARER_TOKEN_FILE_NAME: &str = ".turn_checker_push_bearer_token";
const COMPILED_PUSH_NOTIFICATION_BEARER_TOKEN: Option<&str> =
    option_env!("TURN_CHECKER_PUSH_BEARER_TOKEN");

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PushNotificationEnvironment {
    Sandbox,
    Production,
}

#[derive(Debug, Clone)]
pub struct PushNotificationClient {
    http_client: reqwest::Client,
    push_notification_url: Option<String>,
    device_token: Arc<RwLock<Option<String>>>,
    environment: PushNotificationEnvironment,
}

impl Default for PushNotificationClient {
    fn default() -> Self {
        Self::new()
    }
}

impl PushNotificationClient {
    pub fn new() -> Self {
        Self::new_with_url(PUSH_NOTIFICATION_URL.to_string())
    }

    pub fn new_with_url(push_notification_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            push_notification_url: Some(push_notification_url),
            device_token: Arc::new(RwLock::new(None)),
            environment: default_push_notification_environment(),
        }
    }

    pub fn set_device_token(&self, device_token: Option<String>) {
        let mut stored_device_token = match self.device_token.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *stored_device_token = normalize_optional_string(device_token);
    }

    pub fn device_token(&self) -> Option<String> {
        match self.device_token.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    pub async fn send_new_turn_notification(&self) -> anyhow::Result<()> {
        let i18n = I18n::system();
        let push_notification_url = self
            .push_notification_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!(i18n.t("notification-url-not-configured")))?;
        let device_token = self
            .device_token()
            .ok_or_else(|| anyhow::anyhow!(i18n.t("notification-device-token-unavailable")))?;

        let mut data_map = Map::new();
        let new_turn_value = Value::String("new_turn".to_string());
        data_map.insert("type".to_string(), new_turn_value);
        if let Ok(id_value) = serde_json::to_value(Uuid::new_v4()) {
            data_map.insert("id".to_string(), id_value);
        }

        let bearer_token = push_notification_bearer_token()?;
        let title = i18n.t("notification-title-new-turn");
        let body = i18n.t("notification-body-new-turn");

        self.http_client
            .post(push_notification_url)
            .bearer_auth(&bearer_token)
            .json(&PushNotifRequest {
                device_token: &device_token,
                title: &title,
                body: &body,
                data: data_map,
                environment: &self.environment,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct PushNotifRequest<'a> {
    device_token: &'a str,
    title: &'a str,
    body: &'a str,
    data: Map<String, Value>,
    environment: &'a PushNotificationEnvironment,
}

fn default_push_notification_environment() -> PushNotificationEnvironment {
    if option_env!("GITHUB_ACTIONS") == Some("true") || !cfg!(debug_assertions) {
        PushNotificationEnvironment::Production
    } else {
        PushNotificationEnvironment::Sandbox
    }
}

fn push_notification_bearer_token() -> anyhow::Result<String> {
    if let Some(token) =
        normalize_optional_string(env::var(PUSH_NOTIFICATION_BEARER_TOKEN_ENV).ok())
    {
        return Ok(token);
    }

    if let Some(path) =
        normalize_optional_string(env::var(PUSH_NOTIFICATION_BEARER_TOKEN_FILE_ENV).ok())
    {
        if let Some(token) = read_bearer_token_file(Path::new(&path))? {
            return Ok(token);
        }
    }

    for path in default_bearer_token_paths() {
        if let Some(token) = read_bearer_token_file(&path)? {
            return Ok(token);
        }
    }

    if let Some(token) = normalize_optional_str(COMPILED_PUSH_NOTIFICATION_BEARER_TOKEN) {
        return Ok(token.to_string());
    }

    let i18n = I18n::system();
    Err(anyhow::anyhow!(i18n.tr(
        "notification-bearer-token-missing",
        &[
            ("env", I18nValue::from(PUSH_NOTIFICATION_BEARER_TOKEN_ENV)),
            (
                "file_env",
                I18nValue::from(PUSH_NOTIFICATION_BEARER_TOKEN_FILE_ENV),
            ),
            (
                "compile_env",
                I18nValue::from(PUSH_NOTIFICATION_BEARER_TOKEN_ENV),
            ),
        ],
    )))
}

fn read_bearer_token_file(path: &Path) -> anyhow::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(normalize_optional_string(Some(contents))),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => {
            let i18n = I18n::system();
            Err(anyhow::anyhow!(i18n.tr(
                "notification-bearer-token-file-read-failed",
                &[
                    ("path", I18nValue::from(path.display().to_string())),
                    ("error", I18nValue::from(error.to_string())),
                ],
            )))
        }
    }
}

fn default_bearer_token_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        paths.push(current_dir.join(PUSH_NOTIFICATION_BEARER_TOKEN_FILE_NAME));
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(executable_dir) = current_exe.parent() {
            let executable_path = executable_dir.join(PUSH_NOTIFICATION_BEARER_TOKEN_FILE_NAME);
            if !paths.iter().any(|path| path == &executable_path) {
                paths.push(executable_path);
            }
        }
    }

    paths
}

fn normalize_optional_str(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    normalize_optional_str(value.as_deref()).map(str::to_string)
}

#[cfg(test)]
#[path = "notifications_tests.rs"]
mod tests;
