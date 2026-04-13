#![cfg_attr(windows, windows_subsystem = "windows")]

mod channels;
mod database;
mod i18n;
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

    let i18n = i18n::I18n::system();
    let app_title = i18n.t("app-title");
    let native_options = ui::TurnCheckerApp::native_options(&app_title);

    eframe::run_native(
        &app_title,
        native_options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            ui::TurnCheckerApp::configure_fonts(&cc.egui_ctx);
            Ok(Box::new(ui::TurnCheckerApp::new(
                runtime,
                cc.egui_ctx.clone(),
                channels.ui.clone(),
                i18n.clone(),
            )))
        }),
    )
    .map_err(|err| anyhow::anyhow!("failed to launch UI: {err}"))
}
