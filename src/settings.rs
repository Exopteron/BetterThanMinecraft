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
pub struct Ban {
    pub username: String,
    pub reason: String,
}
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializeBans {
    banlist: Vec<Ban>,
}
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializeOPS {
    ops: Vec<String>,
}
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializeWhitelist {
    whitelisted: Vec<String>,
}
const DEFAULT_CONFIG: &str = r#"# Default config
spawn_protection_radius = 32
whitelist_enabled = false
world_file = "world.cw"
listen_address = "0.0.0.0:25565"
admin_slot = false
public = true
server_name = "BetterThanMinecraft"
max_players = 20
motd = "A BetterThanMinecraft server"
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
    let file = if let Ok(f) = std::fs::read_to_string("config.toml") {
        f
    } else {
        log::info!("Generating configuration file.");
        std::fs::write("config.toml", DEFAULT_CONFIG).unwrap();
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
    let mut doit = true;
    for name in &config.whitelisted {
        if name == username {
            doit = false;
        }
    }
    if doit {
        config.whitelisted.push(username.to_string());
        std::fs::write("whitelist.toml", toml::to_string(&config).unwrap()).unwrap();
    }
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



pub fn get_banlist() -> Vec<Ban> {
    let file = if let Ok(f) = std::fs::read_to_string("banlist.toml") {
        f
    } else {
        log::info!("Generating banlist file.");
        std::fs::write("banlist.toml", r#"banlist = []"#).unwrap();
        r#"banlist = []"#.to_string()
    };
    let config: SerializeBans = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid banlist file!");
        std::process::exit(1);
    };
    config.banlist
}
pub fn add_banlist(username: &str, reason: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("banlist.toml") {
        f
    } else {
        log::info!("Generating banlist file.");
        std::fs::write("banlist.toml", r#"banlist = []"#).unwrap();
        r#"banlist = []"#.to_string()
    };
    let mut config: SerializeBans = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid banlist file!");
        std::process::exit(1);
    };
    config.banlist.push(Ban { username: username.to_string(), reason: reason.to_string()});
    std::fs::write("banlist.toml", toml::to_string(&config).unwrap()).unwrap();
}
pub fn remove_banlist(username: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("banlist.toml") {
        f
    } else {
        log::info!("Generating banlist file.");
        std::fs::write("banlist.toml", r#"banlist = []"#).unwrap();
        r#"banlist = []"#.to_string()
    };
    let mut config: SerializeBans = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid banlist file!");
        std::process::exit(1);
    };
    config.banlist.retain(|ban| {
        ban.username != username
    });
    std::fs::write("banlist.toml", toml::to_string(&config).unwrap()).unwrap();
}