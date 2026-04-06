// This file stores the paired device token and sends placeholder push notification requests.
use std::sync::{Arc, RwLock};

use serde::Serialize;

const PUSH_NOTIFICATION_URL: &str = "https://turn-checker-apns-relay.debortolichris.workers.dev/";

#[derive(Debug, Clone)]
pub struct PushNotificationClient {
    http_client: reqwest::Client,
    push_notification_url: Option<String>,
    device_token: Arc<RwLock<Option<String>>>,
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
        }
    }

    pub fn set_device_token(&self, device_token: Option<String>) {
        let mut stored_device_token = self
            .device_token
            .write()
            .expect("device token lock should not be poisoned"); // No except
        *stored_device_token = normalize_optional_string(device_token);
    }

    pub fn device_token(&self) -> Option<String> {
        self.device_token
            .read()
            .expect("device token lock should not be poisoned") // No excpect
            .clone()
    }

    pub async fn send_new_turn_notification(&self) -> anyhow::Result<()> {
        let push_notification_url = self.push_notification_url.as_deref().ok_or_else(|| {
            anyhow::anyhow!("push notification URL is not configured. Set {PUSH_NOTIFICATION_URL}")
        })?;
        let device_token = self.device_token().ok_or_else(|| {
            anyhow::anyhow!(
                "device token is not available yet. Pair the iOS app and send it through /sync/connect"
            )
        })?;

        self.http_client
            .post(push_notification_url)
            .json(&NewTurnNotifRequest {
                device_token: &device_token,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NewTurnNotifRequest<'a> {
    device_token: &'a str,
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
mod tests {
    use crate::server::notifications::PUSH_NOTIFICATION_URL;

    use super::PushNotificationClient;

    #[test]
    fn device_token_is_trimmed_and_stored() {
        let client = PushNotificationClient::new_with_url(PUSH_NOTIFICATION_URL.to_string());

        client.set_device_token(Some("  token-123  ".to_string()));

        assert_eq!(client.device_token().as_deref(), Some("token-123"));
    }

    #[test]
    fn empty_device_token_clears_the_stored_value() {
        let client = PushNotificationClient::new_with_url(PUSH_NOTIFICATION_URL.to_string());
        client.set_device_token(Some("token-123".to_string()));

        client.set_device_token(Some("   ".to_string()));

        assert_eq!(client.device_token(), None);
    }
}
