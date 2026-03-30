#![cfg_attr(windows, windows_subsystem = "windows")]

mod channels;
mod database;
mod input;
mod models;
mod platform;
mod server;
mod ui;

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let channels = channels::AppChannels::new();

    let native_options = ui::TurnCheckerApp::native_options();
    eframe::run_native(
        "Turn Checker Companion",
        native_options,
        Box::new(move |cc| {
            ui::TurnCheckerApp::configure_fonts(&cc.egui_ctx);
            Ok(Box::new(ui::TurnCheckerApp::new(
                runtime,
                channels.ui.clone(),
            )))
        }),
    )
    .map_err(|err| anyhow::anyhow!("failed to launch UI: {err}"))
}
