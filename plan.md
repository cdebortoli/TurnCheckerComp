# Plan: Add Background Frame to Root UI

## Current Issue
- `src/ui/content/main.rs` uses `egui::Frame::new().fill(theme.bg_primary)` to create a light blue background container
- `src/ui.rs` root UI elements are just text on the default black background
- `src/ui/pairing.rs` pairing view has no background frame

## Goal
Add proper background frames to root UI elements to match the content view styling.

## Tasks

### 1. Update ui.rs - Add Frame Wrapper
Wrap all root UI elements in a styled frame:

```rust
impl eframe::App for TurnCheckerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = theme::Theme::from_visuals(ui.visuals());

        // Get theme once at the start
        self.startup
            .ensure_started(&mut self.runtime, self.pairing.pairing_state());
        self.startup.sync_pairing_connection(&mut self.pairing);

        // UI - Wrap in theme frame with proper background
        egui::Frame::new()
            .fill(theme.bg_primary)
            .inner_margin(theme.spacing_lg)
            .show(ui, |ui| {
                ui.heading(RichText::new("Turn Checker Companion").color(theme.text_primary));
                ui.add_enabled_ui(true, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("─").color(theme.text_muted).font(egui::FontId::monospace(16.0)));
                    });
                });

                if !self.startup.is_ready() {
                    self.startup.show_status(ui, &theme);
                } else if self.pairing.is_paired() {
                    self.content.show(ui);
                } else if self.startup.server_started() {
                    self.pairing.show_waiting(ui);
                } else {
                    ui.label(RichText::new("Starting the local sync server...")
                        .color(theme.text_muted));
                }

                self.startup
                    .show_restore_modal(ui, &mut self.runtime, self.pairing.pairing_state(), &theme);
            });
    }
}
```

### 2. Update pairing.rs - Add Frame Wrapper
Wrap the pairing view content in a styled frame:

```rust
pub fn show_waiting(&mut self, ui: &mut egui::Ui) {
    let theme = Theme::from_visuals(ui.visuals());

    egui::Frame::new()
        .fill(theme.bg_turn_card)
        .inner_margin(theme.spacing_md)
        .corner_radius(theme.corner_radius)
        .show(ui, |ui| {
            ui.heading(RichText::new("Scan To Connect").color(theme.text_primary));
            ui.label(RichText::new("Open the iOS app and scan the QR code to configure the server address.")
                .color(theme.text_secondary));
            ui.add_space(theme.spacing_md);

            if let Err(error) = self.ensure_qr_texture(ui) {
                ui.label(RichText::new("Failed to generate pairing QR code.")
                    .color(theme.destructive));
                ui.monospace(RichText::new(error.to_string()).color(theme.text_muted));
                return;
            }

            if let Some(texture) = &self.qr_texture {
                let image = egui::Image::new(texture).fit_to_exact_size(egui::vec2(280.0, 280.0));
                ui.add(image);
            }

            if let Some(server_connection) = &self.server_connection {
                ui.add_space(theme.spacing_md);
                ui.label(RichText::new("Server URL").color(theme.text_secondary));
                ui.monospace(RichText::new(&server_connection.base_url).color(theme.text_primary));
            }
        });
}
```

## Files to Modify
1. `src/ui.rs` - Add frame wrapper around root UI elements
2. `src/ui/pairing.rs` - Add frame wrapper around pairing view content

## Expected Result
- Root UI will have a light blue background (`theme.bg_primary`)
- Pairing view will have a card-style background (`theme.bg_turn_card`)
- Text will be readable against the background
- Consistent styling with MainContentView
