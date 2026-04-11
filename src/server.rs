// This module wires the server submodules together and exposes the public entrypoint.
mod dto;
mod http;
mod notifications;
mod pairing;
mod service;

pub use http::{spawn, ServerConnectionInfo};
pub use notifications::PushNotificationClient;
pub use pairing::PairingState;

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
