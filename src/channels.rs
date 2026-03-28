#![allow(dead_code)]

use tokio::sync::{mpsc, watch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatcherCommand {
    Start,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatcherEvent {
    Idle,
    Started,
    Stopped,
}

#[derive(Debug)]
pub struct UiChannels {
    pub watcher_command_tx: mpsc::Sender<WatcherCommand>,
    pub watcher_status_rx: watch::Receiver<WatcherEvent>,
}

impl Clone for UiChannels {
    fn clone(&self) -> Self {
        Self {
            watcher_command_tx: self.watcher_command_tx.clone(),
            watcher_status_rx: self.watcher_status_rx.clone(),
        }
    }
}

pub struct BackgroundChannels {
    pub watcher_command_rx: mpsc::Receiver<WatcherCommand>,
    pub watcher_status_tx: watch::Sender<WatcherEvent>,
}

pub struct AppChannels {
    pub ui: UiChannels,
    pub background: BackgroundChannels,
}

impl AppChannels {
    pub fn new() -> Self {
        let (watcher_command_tx, watcher_command_rx) = mpsc::channel(16);
        let (watcher_status_tx, watcher_status_rx) = watch::channel(WatcherEvent::Idle);

        Self {
            ui: UiChannels {
                watcher_command_tx,
                watcher_status_rx,
            },
            background: BackgroundChannels {
                watcher_command_rx,
                watcher_status_tx,
            },
        }
    }
}
