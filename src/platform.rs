#[cfg(windows)]
mod windows {
    use std::{
        env,
        fs::{self, OpenOptions},
        io::Write,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    const ENV_RENDERER: &str = "TURN_CHECKER_WINDOWS_RENDERER";
    const ENV_TRANSPARENCY: &str = "TURN_CHECKER_WINDOWS_TRANSPARENCY";
    const ENV_POWER_PREFERENCE: &str = "TURN_CHECKER_WINDOWS_POWER_PREFERENCE";
    const ENV_GLOW_ACCELERATION: &str = "TURN_CHECKER_WINDOWS_GLOW_ACCELERATION";
    const ENV_PRESENT_MODE: &str = "TURN_CHECKER_WINDOWS_PRESENT_MODE";
    const ENV_LOG_PATH: &str = "TURN_CHECKER_WINDOWS_LOG_PATH";

    pub fn configure_native_options(options: &mut eframe::NativeOptions) {
        options.vsync = true;
        options.renderer = requested_renderer().unwrap_or(eframe::Renderer::Wgpu);
        options.viewport.transparent = Some(requested_transparency().unwrap_or(true));

        match options.renderer {
            eframe::Renderer::Glow => {
                options.hardware_acceleration =
                    requested_glow_acceleration().unwrap_or(eframe::HardwareAcceleration::Off);
            }
            eframe::Renderer::Wgpu => {
                options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;
                options.wgpu_options.present_mode =
                    requested_present_mode().unwrap_or(eframe::wgpu::PresentMode::Fifo);

                if let eframe::egui_wgpu::WgpuSetup::CreateNew(ref mut create_new) =
                    options.wgpu_options.wgpu_setup
                {
                    create_new.power_preference = requested_power_preference()
                        .unwrap_or(eframe::wgpu::PowerPreference::LowPower);
                }
            }
        }
    }

    pub fn log_native_configuration(options: &eframe::NativeOptions) {
        let mut lines = vec![
            "native_configuration".to_owned(),
            format!("renderer={}", options.renderer),
            format!("vsync={}", options.vsync),
            format!(
                "transparency={}",
                options.viewport.transparent.unwrap_or(false)
            ),
            format!(
                "hardware_acceleration={}",
                hardware_acceleration_label(options.hardware_acceleration)
            ),
        ];

        if matches!(options.renderer, eframe::Renderer::Wgpu) {
            lines.push(format!(
                "wgpu.present_mode={:?}",
                options.wgpu_options.present_mode
            ));

            if let eframe::egui_wgpu::WgpuSetup::CreateNew(create_new) =
                &options.wgpu_options.wgpu_setup
            {
                lines.push(format!(
                    "wgpu.power_preference={}",
                    power_preference_label(create_new.power_preference)
                ));
            }
        }

        lines.push(format!("log_path={}", diagnostics_path().display()));
        append_diagnostics(&lines.join(" | "));
    }

    pub fn log_creation_context(cc: &eframe::CreationContext<'_>) {
        if let Some(render_state) = &cc.wgpu_render_state {
            let adapter_info = render_state.adapter.get_info();
            let mut lines = vec![
                "runtime_renderer=wgpu".to_owned(),
                format!("wgpu.adapter_name={}", adapter_info.name),
                format!("wgpu.backend={:?}", adapter_info.backend),
                format!("wgpu.device_type={:?}", adapter_info.device_type),
                format!("wgpu.driver={}", adapter_info.driver),
                format!("wgpu.driver_info={}", adapter_info.driver_info),
                format!("wgpu.target_format={:?}", render_state.target_format),
            ];

            #[cfg(not(target_arch = "wasm32"))]
            {
                for (index, adapter) in render_state.available_adapters.iter().enumerate() {
                    let info = adapter.get_info();
                    lines.push(format!(
                        "wgpu.available_adapter[{index}]={} ({:?}, {:?})",
                        info.name, info.backend, info.device_type
                    ));
                }
            }

            append_diagnostics(&lines.join(" | "));
            return;
        }

        if cc.gl.is_some() {
            append_diagnostics("runtime_renderer=glow");
        } else {
            append_diagnostics("runtime_renderer=unknown");
        }
    }

    fn requested_renderer() -> Option<eframe::Renderer> {
        match env::var(ENV_RENDERER)
            .ok()?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "wgpu" => Some(eframe::Renderer::Wgpu),
            "glow" => Some(eframe::Renderer::Glow),
            _ => None,
        }
    }

    fn requested_transparency() -> Option<bool> {
        parse_bool_env(ENV_TRANSPARENCY)
    }

    fn requested_power_preference() -> Option<eframe::wgpu::PowerPreference> {
        match env::var(ENV_POWER_PREFERENCE)
            .ok()?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "low" | "lowpower" | "low_power" => Some(eframe::wgpu::PowerPreference::LowPower),
            "high" | "highperformance" | "high_performance" => {
                Some(eframe::wgpu::PowerPreference::HighPerformance)
            }
            "none" => Some(eframe::wgpu::PowerPreference::None),
            _ => None,
        }
    }

    fn requested_glow_acceleration() -> Option<eframe::HardwareAcceleration> {
        match env::var(ENV_GLOW_ACCELERATION)
            .ok()?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "off" | "software" => Some(eframe::HardwareAcceleration::Off),
            "preferred" => Some(eframe::HardwareAcceleration::Preferred),
            "required" => Some(eframe::HardwareAcceleration::Required),
            _ => None,
        }
    }

    fn requested_present_mode() -> Option<eframe::wgpu::PresentMode> {
        match env::var(ENV_PRESENT_MODE)
            .ok()?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "fifo" => Some(eframe::wgpu::PresentMode::Fifo),
            "mailbox" => Some(eframe::wgpu::PresentMode::Mailbox),
            "immediate" => Some(eframe::wgpu::PresentMode::Immediate),
            "auto_vsync" | "auto-vsync" => Some(eframe::wgpu::PresentMode::AutoVsync),
            "auto_no_vsync" | "auto-no-vsync" => Some(eframe::wgpu::PresentMode::AutoNoVsync),
            "fifo_relaxed" | "fifo-relaxed" => Some(eframe::wgpu::PresentMode::FifoRelaxed),
            _ => None,
        }
    }

    fn parse_bool_env(key: &str) -> Option<bool> {
        match env::var(key).ok()?.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" | "enabled" => Some(true),
            "0" | "false" | "no" | "off" | "disabled" => Some(false),
            _ => None,
        }
    }

    fn power_preference_label(preference: eframe::wgpu::PowerPreference) -> &'static str {
        match preference {
            eframe::wgpu::PowerPreference::LowPower => "LowPower",
            eframe::wgpu::PowerPreference::HighPerformance => "HighPerformance",
            eframe::wgpu::PowerPreference::None => "None",
        }
    }

    fn hardware_acceleration_label(mode: eframe::HardwareAcceleration) -> &'static str {
        match mode {
            eframe::HardwareAcceleration::Required => "Required",
            eframe::HardwareAcceleration::Preferred => "Preferred",
            eframe::HardwareAcceleration::Off => "Off",
        }
    }

    fn diagnostics_path() -> PathBuf {
        if let Some(path) = env::var_os(ENV_LOG_PATH) {
            return PathBuf::from(path);
        }

        env::temp_dir().join("turn_checker_comp_windows_graphics.log")
    }

    fn append_diagnostics(message: &str) {
        let path = diagnostics_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
            return;
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();

        let _ = writeln!(file, "[{timestamp}] {message}");
    }
}

pub fn configure_native_options(options: &mut eframe::NativeOptions) {
    #[cfg(windows)]
    windows::configure_native_options(options);

    #[cfg(not(windows))]
    let _ = options;
}

pub fn log_native_configuration(options: &eframe::NativeOptions) {
    #[cfg(windows)]
    windows::log_native_configuration(options);

    #[cfg(not(windows))]
    let _ = options;
}

pub fn log_creation_context(cc: &eframe::CreationContext<'_>) {
    #[cfg(windows)]
    windows::log_creation_context(cc);

    #[cfg(not(windows))]
    let _ = cc;
}
