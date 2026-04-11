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
