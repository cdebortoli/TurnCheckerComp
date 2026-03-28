use eframe::egui;
use qrcode::QrCode;

use crate::server;

pub struct PairingView {
    pairing_state: server::PairingState,
    server_connection: Option<server::ServerConnectionInfo>,
    qr_texture: Option<egui::TextureHandle>,
}

impl PairingView {
    pub fn new() -> Self {
        Self {
            pairing_state: server::PairingState::new(),
            server_connection: None,
            qr_texture: None,
        }
    }

    pub fn pairing_state(&self) -> server::PairingState {
        self.pairing_state.clone()
    }

    pub fn set_server_connection(&mut self, server_connection: server::ServerConnectionInfo) {
        self.server_connection = Some(server_connection);
        self.qr_texture = None;
    }

    pub fn is_paired(&self) -> bool {
        self.pairing_state.is_paired()
    }

    pub fn show_waiting(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scan To Connect");
        ui.label("Open the iOS app and scan the QR code to configure the server address.");
        ui.add_space(12.0);

        if let Err(error) = self.ensure_qr_texture(ui) {
            ui.label("Failed to generate pairing QR code.");
            ui.monospace(error.to_string());
            return;
        }

        if let Some(texture) = &self.qr_texture {
            let image = egui::Image::new(texture).fit_to_exact_size(egui::vec2(280.0, 280.0));
            ui.add(image);
        }

        if let Some(server_connection) = &self.server_connection {
            ui.add_space(12.0);
            ui.label("Server URL");
            ui.monospace(&server_connection.base_url);
        }
    }

    fn ensure_qr_texture(&mut self, ui: &egui::Ui) -> anyhow::Result<()> {
        if self.qr_texture.is_some() {
            return Ok(());
        }

        let qr_payload = self
            .server_connection
            .as_ref()
            .map(|info| info.qr_payload.as_str())
            .ok_or_else(|| anyhow::anyhow!("server connection info is not available"))?;

        let code = QrCode::new(qr_payload.as_bytes())?;
        let width = code.width();
        let pixels = code
            .to_colors()
            .into_iter()
            .map(|color| match color {
                qrcode::types::Color::Dark => egui::Color32::BLACK,
                qrcode::types::Color::Light => egui::Color32::WHITE,
            })
            .collect::<Vec<_>>();

        let image = egui::ColorImage::new([width, width], pixels);
        self.qr_texture = Some(ui.ctx().load_texture(
            "server-pairing-qr",
            image,
            egui::TextureOptions::NEAREST,
        ));
        Ok(())
    }
}
