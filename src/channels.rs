use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct UiChannels {
    pub content_refresh_rx: watch::Receiver<u64>,
    pub content_refresh_tx: watch::Sender<u64>,
}

pub struct AppChannels {
    pub ui: UiChannels,
}

impl AppChannels {
    pub fn new() -> Self {
        let (content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);

        Self {
            ui: UiChannels {
                content_refresh_rx,
                content_refresh_tx,
            },
        }
    }
}
