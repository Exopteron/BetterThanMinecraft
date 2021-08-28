/*
pub struct ServerOptions {
  admin_slot: bool,
  public: bool,
  server_name: String,
  max_players: usize,
  motd: String,
}*/
use super::ServerOptions;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializeOPS {
    ops: Vec<String>,
}
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializeWhitelist {
    whitelisted: Vec<String>,
}
const DEFAULT_CONFIG: &str = r#"# Default config
whitelist_enabled = false
world_file = "world.cw"
listen_address = "0.0.0.0:25565"
admin_slot = false
public = true
server_name = "Test"
max_players = 20
motd = "Hi!"
"#;
pub fn get_options() -> ServerOptions {
    Builder::new()
    .format(|buf, record| {
      writeln!(
        buf,
        "[{} {}] - {}",
        Local::now().format("%H:%M:%S"),
        record.level(),
        record.args()
      )
    })
    .filter(None, LevelFilter::Info)
    .init();
    let file = if let Ok(f) = std::fs::read_to_string("Config.toml") {
        f
    } else {
        log::info!("Generating configuration file.");
        std::fs::write("Config.toml", DEFAULT_CONFIG).unwrap();
        DEFAULT_CONFIG.to_string()
    };
    let config: ServerOptions = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid configuration file!");
        std::process::exit(1);
    };
    config
}
pub fn get_ops() -> Vec<String> {
    let file = if let Ok(f) = std::fs::read_to_string("ops.toml") {
        f
    } else {
        log::info!("Generating ops file.");
        std::fs::write("ops.toml", r#"ops = [""]"#).unwrap();
        r#"ops = [""]"#.to_string()
    };
    let config: SerializeOPS = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid ops file!");
        std::process::exit(1);
    };
    config.ops
}
pub fn add_op(username: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("ops.toml") {
        f
    } else {
        log::info!("Generating ops file.");
        std::fs::write("ops.toml", r#"ops = [""]"#).unwrap();
        r#"ops = [""]"#.to_string()
    };
    let mut config: SerializeOPS = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid ops file!");
        std::process::exit(1);
    };
    config.ops.push(username.to_string());
    std::fs::write("ops.toml", toml::to_string(&config).unwrap()).unwrap();
}
pub fn remove_op(username: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("ops.toml") {
        f
    } else {
        log::info!("Generating ops file.");
        std::fs::write("ops.toml", r#"ops = [""]"#).unwrap();
        r#"ops = [""]"#.to_string()
    };
    let mut config: SerializeOPS = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid ops file!");
        std::process::exit(1);
    };
    config.ops.retain(|name| {
        name != username
    });
    std::fs::write("ops.toml", toml::to_string(&config).unwrap()).unwrap();
}



pub fn get_whitelist() -> Vec<String> {
    let file = if let Ok(f) = std::fs::read_to_string("whitelist.toml") {
        f
    } else {
        log::info!("Generating whitelist file.");
        std::fs::write("whitelist.toml", r#"whitelisted = [""]"#).unwrap();
        r#"whitelisted = [""]"#.to_string()
    };
    let config: SerializeWhitelist = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid whitelist file!");
        std::process::exit(1);
    };
    config.whitelisted
}
pub fn add_whitelist(username: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("whitelist.toml") {
        f
    } else {
        log::info!("Generating whitelist file.");
        std::fs::write("whitelist.toml", r#"whitelisted = [""]"#).unwrap();
        r#"whitelisted = [""]"#.to_string()
    };
    let mut config: SerializeWhitelist = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid whitelist file!");
        std::process::exit(1);
    };
    config.whitelisted.push(username.to_string());
    std::fs::write("whitelist.toml", toml::to_string(&config).unwrap()).unwrap();
}
pub fn remove_whitelist(username: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("whitelist.toml") {
        f
    } else {
        log::info!("Generating whitelist file.");
        std::fs::write("whitelist.toml", r#"whitelisted = [""]"#).unwrap();
        r#"whitelisted = [""]"#.to_string()
    };
    let mut config: SerializeWhitelist = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid whitelist file!");
        std::process::exit(1);
    };
    config.whitelisted.retain(|name| {
        name != username
    });
    std::fs::write("whitelist.toml", toml::to_string(&config).unwrap()).unwrap();
}