// This file stores the paired device token and sends placeholder push notification requests.
use std::sync::{Arc, RwLock};

use serde::Serialize;
use serde_json::{Map, Value};
use uuid::Uuid;

const PUSH_NOTIFICATION_URL: &str = "https://turn-checker-apns-relay.debortolichris.workers.dev/";

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
        let push_notification_url = self
            .push_notification_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("push notification URL is not configured."))?;
        let device_token = self.device_token().ok_or_else(|| {
            anyhow::anyhow!(
                "device token is not available yet. Pair the iOS app and accept push notification permissions."
            )
        })?;

        let mut data_map = Map::new();
        let new_turn_value = Value::String("new_turn".to_string());
        data_map.insert("type".to_string(), new_turn_value);
        // let push_id = Value::String(Uuid::new_v4());
        if let Ok(id_value) = serde_json::to_value(Uuid::new_v4()) {
            data_map.insert("id".to_string(), id_value);
        }

        self.http_client
            .post(push_notification_url)
            .bearer_auth("3rGs4L3mRe5cLJ30")
            .json(&PushNotifRequest {
                device_token: &device_token,
                title: "New turn",
                body: "The new turn action was received.",
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

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
#[path = "notifications_tests.rs"]
mod tests;
