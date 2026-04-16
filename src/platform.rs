#[cfg(windows)]
mod windows {
    use serde::Deserialize;
    use std::{
        env, fs,
        path::{Path, PathBuf},
        sync::OnceLock,
    };

    const GRAPHICS_CONFIG_FILE: &str = "graphics.toml";
    const PARAMETERS_FILE: &str = "parameters.txt";
    const DEFAULT_GRAPHICS_CONFIG: &str = include_str!("../graphics.toml");
    const DEFAULT_PARAMETERS: &str = include_str!("../parameters.txt");

    static CONFIG: OnceLock<GraphicsConfig> = OnceLock::new();

    #[derive(Debug, Clone)]
    struct GraphicsConfig {
        renderer: eframe::Renderer,
        transparency: bool,
        glow_hardware_acceleration: eframe::HardwareAcceleration,
        wgpu_present_mode: eframe::wgpu::PresentMode,
        wgpu_power_preference: eframe::wgpu::PowerPreference,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(default)]
    struct GraphicsConfigFile {
        renderer: Option<String>,
        transparency: Option<bool>,
        glow_hardware_acceleration: Option<String>,
        wgpu_present_mode: Option<String>,
        wgpu_power_preference: Option<String>,
    }

    pub fn configure_native_options(options: &mut eframe::NativeOptions) {
        let config = config();

        options.vsync = true;
        options.renderer = config.renderer;
        options.viewport.transparent = Some(config.transparency);

        match config.renderer {
            eframe::Renderer::Glow => {
                options.hardware_acceleration = config.glow_hardware_acceleration;
            }
            eframe::Renderer::Wgpu => {
                options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;
                options.wgpu_options.present_mode = config.wgpu_present_mode;

                if let eframe::egui_wgpu::WgpuSetup::CreateNew(ref mut create_new) =
                    options.wgpu_options.wgpu_setup
                {
                    create_new.power_preference = config.wgpu_power_preference;
                }
            }
        }
    }

    fn config() -> &'static GraphicsConfig {
        CONFIG.get_or_init(GraphicsConfig::load)
    }

    impl GraphicsConfig {
        fn load() -> Self {
            let config_path = ensure_companion_files();

            let file = match fs::read_to_string(&config_path) {
                Ok(contents) => toml::from_str::<GraphicsConfigFile>(&contents)
                    .unwrap_or_else(|error| GraphicsConfigFile::default()),
                Err(error) => GraphicsConfigFile::default(),
            };

            let renderer = parse_renderer(file.renderer.as_deref());
            let transparency = file.transparency.unwrap_or(true);
            let glow_hardware_acceleration =
                parse_glow_hardware_acceleration(file.glow_hardware_acceleration.as_deref());
            let wgpu_present_mode = parse_wgpu_present_mode(file.wgpu_present_mode.as_deref());
            let wgpu_power_preference =
                parse_wgpu_power_preference(file.wgpu_power_preference.as_deref());

            Self {
                renderer,
                transparency,
                glow_hardware_acceleration,
                wgpu_present_mode,
                wgpu_power_preference,
            }
        }
    }

    fn ensure_companion_files() -> PathBuf {
        let executable_dir = executable_dir();
        let config_path = executable_dir.join(GRAPHICS_CONFIG_FILE);
        let parameters_path = executable_dir.join(PARAMETERS_FILE);

        write_if_missing(&config_path, DEFAULT_GRAPHICS_CONFIG);
        write_if_missing(&parameters_path, DEFAULT_PARAMETERS);

        config_path
    }

    fn executable_dir() -> PathBuf {
        env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(env::temp_dir)
    }

    fn write_if_missing(path: &Path, content: &str) {
        if path.exists() {
            return;
        }

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let _ = fs::write(path, content);
    }

    fn parse_renderer(raw: Option<&str>) -> eframe::Renderer {
        match normalize(raw) {
            None => eframe::Renderer::Glow,
            Some(value) if value == "glow" => eframe::Renderer::Glow,
            Some(value) if value == "wgpu" => eframe::Renderer::Wgpu,
            Some(value) => eframe::Renderer::Glow,
        }
    }

    fn parse_glow_hardware_acceleration(raw: Option<&str>) -> eframe::HardwareAcceleration {
        match normalize(raw) {
            None => eframe::HardwareAcceleration::Required,
            Some(value) if value == "off" || value == "software" => {
                eframe::HardwareAcceleration::Off
            }
            Some(value) if value == "preferred" => eframe::HardwareAcceleration::Preferred,
            Some(value) if value == "required" => eframe::HardwareAcceleration::Required,
            Some(value) => eframe::HardwareAcceleration::Required,
        }
    }

    fn parse_wgpu_present_mode(raw: Option<&str>) -> eframe::wgpu::PresentMode {
        match normalize(raw) {
            None => eframe::wgpu::PresentMode::Fifo,
            Some(value) if value == "fifo" => eframe::wgpu::PresentMode::Fifo,
            Some(value) if value == "mailbox" => eframe::wgpu::PresentMode::Mailbox,
            Some(value) if value == "immediate" => eframe::wgpu::PresentMode::Immediate,
            Some(value) if value == "auto_vsync" => eframe::wgpu::PresentMode::AutoVsync,
            Some(value) if value == "auto_no_vsync" => eframe::wgpu::PresentMode::AutoNoVsync,
            Some(value) if value == "fifo_relaxed" => eframe::wgpu::PresentMode::FifoRelaxed,
            Some(value) => eframe::wgpu::PresentMode::Fifo,
        }
    }

    fn parse_wgpu_power_preference(raw: Option<&str>) -> eframe::wgpu::PowerPreference {
        match normalize(raw) {
            None => eframe::wgpu::PowerPreference::LowPower,
            Some(value) if value == "low" || value == "low_power" => {
                eframe::wgpu::PowerPreference::LowPower
            }
            Some(value) if value == "high" || value == "high_performance" => {
                eframe::wgpu::PowerPreference::HighPerformance
            }
            Some(value) if value == "none" => eframe::wgpu::PowerPreference::None,
            Some(value) => eframe::wgpu::PowerPreference::LowPower,
        }
    }

    fn normalize(raw: Option<&str>) -> Option<String> {
        raw.map(|value| value.trim().to_ascii_lowercase().replace('-', "_"))
            .filter(|value| !value.is_empty())
    }
}

pub fn configure_native_options(options: &mut eframe::NativeOptions) {
    #[cfg(windows)]
    windows::configure_native_options(options);

    #[cfg(not(windows))]
    let _ = options;
}
