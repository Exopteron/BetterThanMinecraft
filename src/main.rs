/*
Pirate Realm, An experimental classicube server.
Copyright (c) 2021  Galaxtone, Exopteron

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// HINT: Game struct holds many distinct properties
// together to transport together
// POSSIBLE: The game struct may be removed
// if we instead end up immediately making
// managing message-receiving tasks
// for all properties

// HINT: Make properties as big as possible while
// containing entirely similiar attributes that are
// used together
// Good e.g. PlayerBodies + PlayerNames => Players
// Bad e.g. World + Config => Worofig

// HINT: Message passing is god and it's optimised.
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
mod chunks;
pub mod classic;
pub mod plugins;
use classic::*;
use chunks::{FlatWorldGenerator, World};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
pub const ERR_SENDING_RESULT: &str = "Error sending result";
use std::sync::mpsc as stdmpsc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::broadcast;
use once_cell::sync::Lazy;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
pub mod game;
pub mod settings;
use game::*;
use serde::Deserialize;
#[derive(serde_derive::Deserialize)]
pub struct ServerOptions {
  whitelist_enabled: bool,
  listen_address: String,
  world_file: String,
  admin_slot: bool,
  public: bool,
  server_name: String,
  max_players: usize,
  motd: String,
}
static CONFIGURATION: Lazy<ServerOptions> = Lazy::new(|| settings::get_options());
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut pregmts = PreGMTS::new();
  pregmts.register_command("ver".to_string(), "", "Get server version", Box::new(|gmts: CMDGMTS, args, sender| {
    Box::pin(async move {
        gmts.chat_to_id(&format!("&aServer is running BetterThanMinecraft v{}.", VERSION), -1, sender).await;
        0
    })
}));
  if CONFIGURATION.public {}
  use tokio::runtime::Runtime;
  plugins::coreutils::CoreUtils::initialize(&mut pregmts);
  //plugins::longermessages::LongerMessagesCPE::initialize(&mut pregmts);
  //plugins::testplugin::TestPlugin::initialize(&mut pregmts);
  let gmts = GMTS::setup(pregmts).await;
  let data = PlayerData { position: None };
  let (console_send, mut console_recv) = stdmpsc::channel::<PlayerCommand>();
  let player = Player {
    data: data,
    op: true,
    permission_level: 5,
    entity: false,
    id: -69i8 as u32,
    name: "Server".to_string(),
    message_send: console_send.clone(),
    supported_extensions: None,
  };
  gmts.register_user(player).await.unwrap();
  let (send_1, mut recv_1) = oneshot::channel::<Option<()>>();
  let (send_2, mut recv_2) = oneshot::channel::<Option<()>>();
  let cgmts_1 = gmts.clone();
  let cgmts_2 = gmts.clone();
  tokio::spawn(async move {
    loop {
       let mut command = String::new();
      let x = std::io::stdin().read_line(&mut command);
      if x.is_err() {
        log::error!("Error reading command!");
        continue;
      }
      let command = command.trim().to_string();
      cgmts_1.execute_command(-69, format!("/{}", command)).await.unwrap();
    }
  });
   tokio::spawn(async move {
    loop {
        if let Ok(msg) = console_recv.recv() {
          match msg {
            PlayerCommand::Message { id, message } => {
              log::info!("[MESSAGE TO CONSOLE] {}", message);
            }
            _ => {

            }
          }
        }
    }
  });
  // Pass around immutable references, and clone the sender.

  //example(&gmts);

  let listener = TcpListener::bind(&CONFIGURATION.listen_address).await?;
  log::info!("Server listening on {}", CONFIGURATION.listen_address);
  let gmts2 = gmts.clone();
  tokio::spawn(async move {
    loop {
      let possible = listener.accept().await;
      if possible.is_err() {
        continue;
      }
      let (stream, _) = possible.unwrap();
      let gmts = gmts2.clone();
      tokio::spawn(async move {
        if let None = new_incoming_connection_handler(stream, &gmts).await {
          log::error!("Player join error!");
        }
      });
    }
  });
  loop {}
}
async fn mc_con_handler(
  mut stream: TcpStream,
  gmts: &GMTS,
) -> Result<(), Box<dyn std::error::Error>> {
  
  return Ok(());
}
async fn new_incoming_connection_handler(mut stream: TcpStream, gmts: &GMTS) -> Option<()> {
  let mut test = Box::pin(&mut stream);
  let spawn_position = gmts.get_spawnpos().await?;
  let packet = ClassicPacketReader::read_packet_reader(&mut test).await.ok()?;
  let (msg_send, recv) = stdmpsc::channel::<PlayerCommand>();
  drop(test);
  if let classic::Packet::PlayerIdentification { p_ver, user_name, v_key, cpe_id } = packet {
    let player_count = gmts.player_count().await?;
    if player_count + 1 > CONFIGURATION.max_players {
      let packet = classic::Packet::Disconnect {
        reason: "Server is full!".to_string(),
      };
      stream
        .write_all(&ClassicPacketWriter::serialize(packet).ok()?)
        .await.ok()?;
        log::info!("Kicked {} because the server is full.", user_name);
      return None;
    }
    if user_name.len() >= 20 {
      let packet = classic::Packet::Disconnect {
        reason: "Name too long!".to_string(),
      };
      stream
        .write_all(&ClassicPacketWriter::serialize(packet).ok()?)
        .await.ok()?;
        log::info!("s");
      return None;
    }
    let our_id = gmts.get_unused_id().await?; 
    let data = PlayerData {
      position: Some(spawn_position.clone()),
    };
    if let Some(_) = gmts.kick_user_by_name(&user_name, "You logged in from another location").await {
      log::info!("{} was already logged in! Kicked other instance.", user_name);
    }
    let mut permission_level: usize;
    let mut op: bool;
        let cpe = match cpe_id {
      0x42 => true,
      _ => false,
    };
    permission_level = 1;
    op = false;
    let op_list = settings::get_ops();
    for op_name in op_list {
      if user_name == op_name {
        permission_level = 4;
        op = true;
      }
    }
    let mut supported_extensions: Option<HashMap<String, CPEExtensionData>> = None;
    if cpe {
      let extensions = gmts.get_extensions().await;
      let ext_info = classic::Packet::ExtInfo { appname: format!("BetterThanMinecraft v{}", VERSION), extension_count: extensions.len() as i16};
      stream
      .write_all(&ClassicPacketWriter::serialize(ext_info).ok()?)
      .await.ok()?;
      for (extension, data) in extensions {
        let ext_entry = classic::Packet::ExtEntry { extname: extension.to_string(), version: data.version as i32};
        stream
        .write_all(&ClassicPacketWriter::serialize(ext_entry).ok()?)
        .await.ok()?;
      }
      let mut test = Box::pin(&mut stream);
      let (appname, extcount) = if let classic::Packet::ExtInfo { appname, extension_count } = ClassicPacketReader::read_packet_reader(&mut test).await.ok()? {
        (appname, extension_count)
      } else {
        return None;
      };
      let mut client_supported_extensions: HashMap<String, CPEExtensionData> = HashMap::new();
      for _ in 0..extcount {
        let (extname, version) = if let classic::Packet::ExtEntry { extname, version } = ClassicPacketReader::read_packet_reader(&mut test).await.ok()? {
          (extname, version)
        } else {
          return None;
        };
        let data = CPEExtensionData { version: version as usize, required: false };
        client_supported_extensions.insert(extname, data);
      }
      let mut required_extensions: HashMap<String, CPEExtensionData> = HashMap::new();
      for (extension, data) in extensions {
        if data.required {
          required_extensions.insert(extension.to_string(), data.clone());
        }
      }
      for (extension, data) in required_extensions {
        if client_supported_extensions.get(&extension).is_none() || client_supported_extensions.get(&extension).unwrap().version != data.version {
          let packet = classic::Packet::Disconnect {
            reason: "Missing required extensions.".to_string(),
          };
          stream
            .write_all(&ClassicPacketWriter::serialize(packet).ok()?)
            .await.ok()?;
            log::info!("{} is missing required extensions.", user_name);
          return None;
        }
      }
      let mut super_supported_extensions = HashMap::new();
      for (extension, data) in client_supported_extensions {
        if extensions.get(&extension).is_some() && extensions.get(&extension).unwrap().version == data.version {
          super_supported_extensions.insert(extension, data);
        }
      }
      supported_extensions = Some(super_supported_extensions);
    } else if *gmts.cpe_required().await {
      let packet = classic::Packet::Disconnect {
        reason: "This server requires the use of CPE.".to_string(),
      };
      stream
        .write_all(&ClassicPacketWriter::serialize(packet).ok()?)
        .await.ok()?;
        log::info!("{} doesn't support CPE.", user_name);
      return None;
    }
    let whitelist = settings::get_whitelist();
    let mut in_whitelist = false;
    for person in whitelist {
      if user_name == person {
        in_whitelist = true;
      }
    }
    if !in_whitelist && permission_level < 4 && CONFIGURATION.whitelist_enabled {
      let packet = classic::Packet::Disconnect {
        reason: "You are not white-listed on this server!".to_string(),
      };
      stream
        .write_all(&ClassicPacketWriter::serialize(packet).ok()?)
        .await.ok()?;
        log::info!("{} is not whitelisted.", user_name);
      return None;
    }
    let player = Player {
      data: data,
      op,
      permission_level,
      entity: true,
      id: our_id,
      name: user_name.clone(),
      message_send: msg_send.clone(),
      supported_extensions
    };
    gmts.register_user(player).await?;
    if let None = internal_inc_handler(stream, gmts, recv, &user_name.clone(), our_id as u32, p_ver, op, cpe).await {
      let hooks = gmts.get_ondisconnect_hooks().await;
      for hook in &*hooks {
        hook(gmts.clone(), our_id as i8).await;
      } 
      if let None = gmts.remove_user(our_id as i8).await {
        log::error!("Error removing user.");
      }
      if let None = gmts.return_id(our_id as i8).await {
        log::error!("Error returning id.");
      }
      if let None = gmts.chat_broadcast(&format!("&e{} left the game.", user_name), -1).await {
        log::error!("Error broadcasting chat message.");
      }
    }
  } else {
    log::info!("G");
    return None;
  }
  Some(())
}
async fn internal_inc_handler(stream: TcpStream, gmts: &GMTS, reciever: stdmpsc::Receiver<PlayerCommand>, our_username: &str, our_id: u32, our_p_ver: u8, op: bool, cpe: bool) -> Option<()> {
  let hooks = gmts.get_earlyonconnect_hooks().await;
  let stream = std::sync::Arc::new(tokio::sync::Mutex::new(stream));
  for hook in &*hooks {
    hook(gmts.clone(), stream.clone(), our_id as i8).await?;
  }
  let stream = std::sync::Arc::try_unwrap(stream).ok()?;
  let mut stream = stream.into_inner();
  let server_identification = ClassicPacketWriter::server_identification(
    0x07,
    CONFIGURATION.server_name.clone(),
    CONFIGURATION.motd.clone(),
    op,
  ).ok()?;
  stream.write_all(&server_identification).await.unwrap();
  log::info!(
    "{}[{}] logging in with entity id {} protocol version {}",
    our_username,
    stream.peer_addr().ok()?.to_string(),
    our_id,
    our_p_ver
  );
  if let None = gmts.chat_broadcast(&format!("&e{} logging in...", our_username), -1).await {
    log::error!("Error broadcasting chat message.");
  }
  let mut world = if let Some(w) = gmts.get_world().await {
    w
  } else {
    return None;
  };
  log::info!("Sending world to {}", our_username);
  world
    .to_packets(&mut Box::pin(&mut stream))
    .await
    .expect("Shouldn't fail!");
  log::info!("World sent to {}", our_username);
  let teleport_player = classic::Packet::PlayerTeleportS {
    player_id: -1,
    position: PlayerPosition::from_pos(94, 38, 66),
  };
  let iswrite = stream
    .write_all(&ClassicPacketWriter::serialize(teleport_player).ok()?)
    .await.ok()?;
    let gmts2 = gmts.clone();
  if let None = gmts.chat_broadcast(&format!("&e{} joined the game.", our_username), -1).await {
    log::error!("Error broadcasting chat message.");
  }
  let hooks = gmts.get_onconnect_hooks().await;
  let stream = std::sync::Arc::new(tokio::sync::Mutex::new(stream));
  for hook in &*hooks {
    hook(gmts.clone(), stream.clone(), our_id as i8).await?;
  }
  let stream = std::sync::Arc::try_unwrap(stream).ok()?;
  let stream = stream.into_inner();
  let (mut readhalf, mut writehalf) = stream.into_split();
  let (send_1, mut recv_1) = oneshot::channel::<Option<()>>();
  let (send_2, mut recv_2) = oneshot::channel::<Option<()>>();
  tokio::spawn(async move {
    let function = || async move {
      if let None = gmts2.spawn_all_players(our_id as i8).await {
        return None;
      }
      loop {
        match recv_1.try_recv() {
          Ok(_) => {
            return Some(());
          }
          _ => {

          }
        }
        let recv = reciever.try_recv();
        match recv {
          Ok(msg) => {
            match msg {
              PlayerCommand::SetBlock { block } => {
                let packet = classic::Packet::SetBlockS { block };
                let packet = ClassicPacketWriter::serialize(packet).unwrap();
                let write = writehalf.write_all(&packet).await;
                if write.is_err() {
                        return None;
                }
              }
              PlayerCommand::SpawnPlayer { position, id, name } => {
                let packet = classic::Packet::SpawnPlayer {
                  player_id: id,
                  name,
                  position,
                };
                let packet = ClassicPacketWriter::serialize(packet).unwrap();
                let write = writehalf.write_all(&packet).await;
                if write.is_err() {
                  return None;
                }
              }
              PlayerCommand::DespawnPlayer { id } => {
                let packet = classic::Packet::DespawnPlayer { player_id: id };
                let packet = ClassicPacketWriter::serialize(packet).unwrap();
                let write = writehalf.write_all(&packet).await;
                if write.is_err() {
                  return None;
                }
              }
              PlayerCommand::PlayerTeleport { position, id } => {
                let packet = classic::Packet::PlayerTeleportS {
                  player_id: id,
                  position: position,
                };
                let packet = ClassicPacketWriter::serialize(packet).unwrap();
                let write = writehalf.write_all(&packet).await;
                if write.is_err() {
                  return None;
                }
              }
              PlayerCommand::Message { id, message } => {
                let packet = classic::Packet::Message {
                  player_id: id,
                  message,
                };
                let packet = ClassicPacketWriter::serialize(packet).unwrap();
                let write = writehalf.write_all(&packet).await;
                if write.is_err() {
                  return None;
                }
              }
              PlayerCommand::Disconnect { reason } => {
                let packet = classic::Packet::Disconnect { reason };
                let packet = ClassicPacketWriter::serialize(packet).unwrap();
                let write = writehalf.write_all(&packet).await;
                if write.is_err() {
                  return None;
                }
              }
            }
          }
          Err(_) => {
            continue;
          }
        }
      }
    };
    match function().await {
      None => {
        send_2.send(None).unwrap();
      }
      _ => {

      }
    }
  });
  let gmts = gmts.clone();
  let our_username = our_username.to_string(); 
    let packet_handler_wrapper = || async move {
      //let mut stored_msg = String::new();
      use tokio::io::AsyncReadExt;
      loop {
        match recv_2.try_recv() {
          Ok(_) => {
            return Some(());
          }
          _ => {

          }
        }
        let mut s_p_id = [0; 1];
        let x = readhalf.peek(&mut s_p_id).await.ok();
        if x.is_none() {
          return None;
        }
        let hooks = gmts.get_packetrecv_hooks().await;
        if let Some(hook) = hooks.get(&s_p_id[0]) {
          let readhalf_2 = std::sync::Arc::new(tokio::sync::Mutex::new(readhalf));
          hook(gmts.clone(), readhalf_2.clone(), s_p_id[0], our_id as i8).await;
          readhalf = std::sync::Arc::try_unwrap(readhalf_2).ok()?.into_inner();
          continue;
        };
        //println!("Started");
        let packet = ClassicPacketReader::read_packet_reader(&mut Box::pin(&mut readhalf)).await;
        if packet.is_err() {
          return None;
        }
        let packet = packet.unwrap();
        match packet {
          classic::Packet::PlayerClicked {
            button,
            action,
            yaw,
            pitch,
            target_entity_id,
            target_block_x,
            target_block_y,
            target_block_z,
            target_block_face,
          } => {
    
          }
          classic::Packet::SetBlockC {
            coords,
            mode,
            block_type,
          } => {
            match mode {
              0x00 => {
                let block = Block {
                  position: coords,
                  id: 0x00,
                };
                if let None = gmts.set_block(&block, our_id as i8).await {
                    if let Some(x) = gmts.get_block(block.position).await {
                        gmts.block_to_id(x, our_id as i8).await;
                    } else {
                      log::error!("Block error!");
                    }
                }
              }
              _ => {
                let block = Block {
                  position: coords,
                  id: block_type,
                };
                if let None = gmts.set_block(&block, our_id as i8).await {
                  if let Some(x) = gmts.get_block(block.position).await {
                      gmts.block_to_id(x, our_id as i8).await;
                  } else {
                    log::error!("Block error!");
                  }
                }
              }
            }
          }
          classic::Packet::PositionAndOrientationC { position, .. } => {
            gmts.send_position_update(our_id as i8, position).await;
          }
          classic::Packet::MessageC { message, unused } => {
              //if unused == 0 {
                 if message.starts_with("/") {
                  gmts.execute_command(our_id as i8, message).await;
                } else {
                  let mut prefix = format!("<{}> ", our_username);
                  prefix.push_str(&message);
                  let message = prefix;
                  let message = message.as_bytes().to_vec();
                  let message = message.chunks(64).collect::<Vec<&[u8]>>();
                  let mut msg2 = vec![];
                  for m in message {
                    msg2.push(String::from_utf8_lossy(&m).to_string());
                  }
                  let m = msg2.remove(0);
                  gmts.chat_broadcast(&m, (our_id as u8) as i8).await;
                  for m in msg2 {
                  gmts.chat_broadcast(&format!("> {}", m), (our_id as u8) as i8).await;
                }
                }
              //}
          }
          _ => {}
        }
      }
    };
  match packet_handler_wrapper().await {
    None => {
      send_1.send(None).ok().unwrap();
    }
    _ => {

    }
  }
  //let mut test = Box::pin(&mut readhalf);
  //let (a, b) = tokio::join!(messagehandle, packethandle);
  //a.ok()?;
  //b.ok()?;
  None
}