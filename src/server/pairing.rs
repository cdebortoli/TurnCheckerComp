// This file contains the in-memory pairing state shared by the UI and HTTP server.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct PairingState {
    paired: Arc<AtomicBool>,
}

impl PairingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_paired(&self) {
        self.paired.store(true, Ordering::SeqCst);
    }

    pub fn is_paired(&self) -> bool {
        self.paired.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.paired.store(false, Ordering::SeqCst);
    }
}
