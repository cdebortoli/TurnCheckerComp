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

#[derive(Debug, Clone)]
pub struct UiChannels {
    pub watcher_command_tx: mpsc::Sender<WatcherCommand>,
    pub watcher_status_rx: watch::Receiver<WatcherEvent>,
    pub content_refresh_rx: watch::Receiver<u64>,
    pub content_refresh_tx: watch::Sender<u64>,
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
        let (content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);

        Self {
            ui: UiChannels {
                watcher_command_tx,
                watcher_status_rx,
                content_refresh_rx,
                content_refresh_tx,
            },
            background: BackgroundChannels {
                watcher_command_rx,
                watcher_status_tx,
            },
        }
    }
}
