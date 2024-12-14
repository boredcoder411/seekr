use std::io::{Read, Write};

use tracing::{debug, warn};

pub const APP_ID: &str = "dev.luxluth.seekr";
pub const DEFAULT_CONFIG: &str = r#"# General seekr application config
[general]

# icon theme
theme = Adwaita

# terminal used to run shell based programs
terminal = kitty

# terminal launch args
args = -e
"#;

pub const DEFAULT_CSS: &str = include_str!("./style.css");

#[derive(Clone)]
pub struct GeneralConf {
    pub theme: String,
    pub terminal: String,
    pub args: Vec<String>,
}

impl Default for GeneralConf {
    fn default() -> Self {
        GeneralConf {
            theme: "Adwaita".to_string(),
            terminal: "kitty".to_string(),
            args: vec!["-e".to_string()],
        }
    }
}

#[derive(Default, Clone)]
pub struct Config {
    pub general: GeneralConf,
    pub css: String,
}

impl Config {
    pub fn parse(path: std::path::PathBuf) -> Self {
        let mut css = DEFAULT_CSS.to_string();
        let css_path = path.parent().unwrap().join("style.css");
        if css_path.exists() {
            if let Ok(mut f) = std::fs::File::open(&css_path) {
                css = String::new();
                let _ = f.read_to_string(&mut css);
            }
        } else {
            if let Ok(mut f) = std::fs::File::create(&css_path) {
                let _ = f.write(DEFAULT_CSS.as_bytes());
            }
        }

        if let Ok(mut f) = std::fs::File::open(&path) {
            let mut data = String::new();
            let _ = f.read_to_string(&mut data);
            let mut is_in_general = false;
            let mut general = GeneralConf::default();

            for (line, item) in ini_roundtrip::Parser::new(&data).enumerate() {
                match item {
                    ini_roundtrip::Item::Error(e) => {
                        warn!("{}:{line}: {e}", path.display());
                    }
                    ini_roundtrip::Item::Section {
                        name: "general", ..
                    } => {
                        is_in_general = true;
                    }
                    ini_roundtrip::Item::Property {
                        key: "theme", val, ..
                    } => {
                        if is_in_general {
                            general.theme = val.unwrap_or("Adwaita").trim().to_string();
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "terminal",
                        val,
                        ..
                    } => {
                        if is_in_general {
                            general.terminal = val.unwrap_or("kitty").trim().to_string();
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "args", val, ..
                    } => {
                        if is_in_general {
                            general.args = val
                                .unwrap_or("-e")
                                .trim()
                                .split(' ')
                                .map(|x| x.to_string())
                                .collect();
                        }
                    }
                    _ => {}
                }
            }

            return Self { general, css };
        }

        return Self::default();
    }
}

pub fn init_config_dir() -> std::path::PathBuf {
    let raw_path = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or(format!("{}/.config", std::env::var("HOME").unwrap()));
    let base_dir = std::path::Path::new(&raw_path);
    let config_dir = base_dir.join("seekr/");

    if !config_dir.exists() {
        let _ = std::fs::create_dir_all(&config_dir);
    }

    let config_file = config_dir.join("default.conf");
    debug!("config_path: {}", config_file.display());

    if !config_file.exists() {
        if let Ok(mut f) = std::fs::File::create(&config_file) {
            let _ = f.write(DEFAULT_CONFIG.as_bytes());
        }
    }

    return config_file;
}
