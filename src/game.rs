use super::chunks::*;
use std::collections::HashMap;
use std::sync::mpsc as stdmpsc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
// Constants for use, probably temporary as runtime ability to get block name will be needed in the future.
pub type BlockId = u8;
pub const ERR_SENDING_RESULT: &str = "Error sending result";
pub const UNKNOWN_COMMAND: &str = "&cUnknown command.";
// Being used as a PSUEDO-Enum
#[allow(non_snake_case)]
pub mod BlockIds {
    use super::BlockId;
    // This is technically a sub-module,
    // so i need to import from the parent
    pub const AIR: BlockId = 0;
    pub const STONE: BlockId = 1;
    pub const GRASS: BlockId = 2;
    pub const DIRT: BlockId = 3;
    pub const COBBLESTONE: BlockId = 4;
    pub const PLANKS: BlockId = 5;
    pub const SPONGE: BlockId = 19;
    pub const SAND: BlockId = 12;
}

/* ================================================ maths.rs ================================================ */
#[derive(Clone, Debug)]
pub struct BlockPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
impl PartialEq for BlockPosition {
    fn eq(&self, other: &BlockPosition) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}
impl std::hash::Hash for BlockPosition {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        state.write_usize(self.x);
        state.write_usize(self.y);
        state.write_usize(self.z);
        state.finish();
    }
}
impl Eq for BlockPosition {}
impl BlockPosition {
    fn all_offsets(&self) -> Vec<BlockPosition> {
        let mut vec = vec![];
        let mut a = self.clone();
        a.x -= 1;
        vec.push(a);
        let mut a = self.clone();
        a.y -= 1;
        vec.push(a);
        let mut a = self.clone();
        a.z -= 1;
        vec.push(a);
        let mut a = self.clone();
        a.x += 1;
        vec.push(a);
        let mut a = self.clone();
        a.y += 1;
        vec.push(a);
        let mut a = self.clone();
        a.z += 1;
        vec.push(a);
        return vec;
    }
}
pub trait Plugin {
    fn initialize(pre_gmts: &mut PreGMTS);
}
#[derive(Clone, Copy, Debug)]
pub struct PlayerPosition {
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub yaw: u8,
    pub pitch: u8,
}
impl PlayerPosition {
    pub const FEET_DISTANCE: u16 = 51;
    pub fn from_pos(x: u16, y: u16, z: u16) -> Self {
        PlayerPosition {
            x: (x << 5) + 16,
            y: (y << 5) + PlayerPosition::FEET_DISTANCE,
            z: (z << 5) + 16,
            yaw: 0,
            pitch: 0,
        }
    }
    pub fn distance_to(&self, target: BlockPosition) -> f64 {
        return (((self.x as f64 / 32.0) - target.x as f64).powf(2.0)
            + ((self.y as f64 / 32.0) - target.y as f64).powf(2.0)
            + ((self.z as f64 / 32.0) - target.z as f64).powf(2.0))
        .sqrt();
    }
    pub fn distance_to_plr(&self, target: PlayerPosition) -> f64 {
        return (((self.x as f64 / 32.0) - (target.x as f64 / 32.0)).powf(2.0)
            + ((self.y as f64 / 32.0) - (target.y as f64 / 32.0)).powf(2.0)
            + ((self.z as f64 / 32.0) - (target.z as f64 / 32.0)).powf(2.0))
        .sqrt();
    }
}
impl Default for PlayerPosition {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            yaw: 0,
            pitch: 0,
        }
    }
}
pub type BlockID = u8;
#[derive(Clone, Debug)]
pub struct Block {
    pub position: BlockPosition,
    pub id: BlockID,
}

pub struct PlayerData {
    pub position: PlayerPosition,
}
pub struct Player {
    pub data: PlayerData,
    pub op: bool,
    pub permission_level: usize,
    // unique identifier, shorthand
    pub id: u32,
    pub name: String,
    pub message_send: stdmpsc::Sender<PlayerCommand>,
    pub supported_extensions: Option<HashMap<String, CPEExtensionData>>,
}
impl Player {
    pub fn send_teleport(&self, id: i8, position: &PlayerPosition) -> Option<()> {
        self.message_send
            .send(PlayerCommand::PlayerTeleport {
                position: *position,
                id,
            })
            .ok()
    }
}
/*
              for player in &players {
                if player.1.id != id {
                  let x = player.1.message_send.send(PlayerCommand::PlayerTeleport {
                    position: position.clone(),
                    id: (my_id as u8) as i8,
                  });
                  if x.is_err() {
                    panic!("Shouldn't fail!");
                  }
                }
              }
*/
pub enum Protocol {
    Classic { plr_id: u8 },
}
#[derive(Clone)]
pub struct GMTS {
    // GMTS short for: Game Managing Task ~~Senders~~ System
    pub world_send: mpsc::Sender<WorldCommand>,
    pub players_send: mpsc::Sender<PlayersCommand>,
    pub tempcrntid_send: mpsc::Sender<TempCrntIdCommand>,
    pub commands_send: mpsc::Sender<CommandsCommand>,
    pub extensions: HashMap<String, CPEExtensionData>,
    pub storage_send: mpsc::Sender<StorageCommand>,
    pub cpe_required: bool,
    pub onconnect_hooks: Arc<Vec<
        Box<
            dyn Fn(
                    GMTS,
                    Arc<Mutex<tokio::net::TcpStream>>,
                    i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    >>,
    pub earlyonconnect_hooks: Arc<Vec<
        Box<
            dyn Fn(
                    GMTS,
                    Arc<Mutex<tokio::net::TcpStream>>,
                    i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    >>,
    pub packet_recv_hooks: Arc<HashMap<
        u8,
        Box<
            dyn Fn(
                    GMTS,
                    Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
                    u8,
                    i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    >>,
    pub ondisconnect_hooks: Arc<Vec<Box<
        dyn Fn(
            GMTS,
            i8,
            ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
            + Send
            + Sync,
    >>>,
}
#[derive(Clone)]
pub struct CPEExtensionData {
    pub version: usize,
    pub required: bool,
}
#[derive(Clone)]
pub struct CMDGMTS {
    pub world_send: mpsc::Sender<WorldCommand>,
    pub players_send: mpsc::Sender<PlayersCommand>,
    pub tempcrntid_send: mpsc::Sender<TempCrntIdCommand>,
    pub storage_send: mpsc::Sender<StorageCommand>,
}
impl CMDGMTS {
    pub async fn chat_broadcast(&self, message: &str, id: i8) -> Option<()> {
        log::info!("[CHAT]: {}", message);
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::Message {
            id: (id as u8) as i8,
            message: message.to_string(),
        };
        self.players_send
            .send(PlayersCommand::PassMessageToAll { message, res_send })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn chat_to_id(&self, message: &str, id: i8, target_id: i8) -> Option<()> {
        if let Some(n) = Self::get_username(&self, target_id).await {
            log::info!("[CHAT to {}]: {}", n, message);
        } else {
            log::info!("[CHAT to {}]: {}", target_id, message);
        }
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::Message {
            id: (id as u8) as i8,
            message: message.to_string(),
        };
        self.players_send
            .send(PlayersCommand::PassMessageToID {
                message,
                id: target_id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn message_to_id(&self, message: PlayerCommand, target_id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::PassMessageToID {
                message,
                id: target_id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn block_to_id(&self, block: Block, target_id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::SetBlock { block };
        self.players_send
            .send(PlayersCommand::PassMessageToID {
                message,
                id: target_id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn chat_to_username(
        &self,
        message: &str,
        id: i8,
        target_username: String,
    ) -> Option<()> {
        log::info!("[CHAT to {}]: {}", target_username, message);
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::Message {
            id: (id as u8) as i8,
            message: message.to_string(),
        };
        self.players_send
            .send(PlayersCommand::PassMessageByName {
                message,
                username: target_username,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn get_username(&self, id: i8) -> Option<String> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::GetUsername {
                id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn set_permission_level(&self, id: i8, level: usize) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::SetPermissionLevel {
                id: id as u32,
                level,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn get_permission_level(&self, id: i8) -> Option<usize> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::GetPermissionLevel {
                id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn get_supported_extensions(&self, id: i8) -> Option<HashMap<String, CPEExtensionData>> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::GetSupportedExtensions {
                id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn get_id(&self, username: String) -> Option<i8> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::GetID { username, res_send })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn remove_user(&self, id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::RemoveUser {
                user_id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn return_id(&self, id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.tempcrntid_send
            .send(TempCrntIdCommand::ReturnFreeID {
                id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn get_unused_id(&self) -> Option<u32> {
        let (res_send, res_recv) = oneshot::channel();
        self.tempcrntid_send
            .send(TempCrntIdCommand::FetchFreeID { res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn pass_message_to_all(&self, message: PlayerCommand) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::PassMessageToAll { message, res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn send_position_update(&self, id: i8, position: PlayerPosition) {
        let (res_send, res_recv) = oneshot::channel();
        let x = self
            .players_send
            .send(PlayersCommand::UpdatePosition {
                my_id: id as u32,
                position,
                res_send,
            })
            .await;
        if x.is_err() {
            panic!("Error sending position update!");
        }
        res_recv.await.expect("Error sending position update!");
    }
    pub async fn get_block(&self, block: BlockPosition) -> Option<Block> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::GetBlock {
                pos: block,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn set_block(&self, block: Block, sender_id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::SetBlockP {
                block,
                sender_id: sender_id as u32,
                players_send: self.players_send.clone(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn save_world(&self) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::SaveWorld { res_send })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn get_world(&self) -> Option<World> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::GetWorld { res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn kick_user_by_name(&self, name: &str, reason: &str) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::KickUserByName {
                username: name.to_string(),
                reason: reason.to_string(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn register_user(&self, user: Player) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        let pos = user.data.position.clone();
        self.players_send
            .send(PlayersCommand::NewUser { user, res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn spawn_all_players(&self, id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::SpawnAllPlayers {
                my_id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn get_value(&self, key: &str) -> Option<GMTSElement> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::GetValue {
                key: key.to_string(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn rem_value(&self, key: &str) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::RemoveValue {
                key: key.to_string(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn new_value(&self, key: &str, value: GMTSElement) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::NewValue {
                key: key.to_string(),
                value,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn set_value(&self, key: &str, value: GMTSElement) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::SetValue {
                key: key.to_string(),
                value,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn player_count(&self) -> Option<usize> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::OnlinePlayerCount {
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
}
use std::any::{Any, TypeId};
use std::sync::Arc;
#[derive(Clone)]
pub struct GMTSElement {
    pub val: Arc<Box<dyn Any + Sync + Send>>,
}
use std::future::Future;
use std::pin::Pin;
pub struct PreGMTS {
    pub commands: HashMap<
        String,
        Box<
            dyn Fn(CMDGMTS, Vec<String>, i8) -> Pin<Box<dyn Future<Output = usize> + Send>>
                + Send
                + Sync,
        >,
    >,
    pub pmta_hooks: Vec<
        Box<
            dyn Fn(CMDGMTS, PlayerCommand) -> Pin<Box<dyn Future<Output = PlayerCommand> + Send>>
                + Send
                + Sync,
        >,
    >,
    pub getblock_hooks:
        Vec<fn(CMDGMTS, BlockPosition) -> Pin<Box<dyn Future<Output = BlockPosition> + Send>>>,
    pub setblock_hooks: Vec<
        Box<
            dyn Fn(
                    CMDGMTS,
                    Block,
                    u32,
                ) -> Pin<Box<dyn Future<Output = Option<(Block, u32)>> + Send>>
                + Send
                + Sync,
        >,
    >,
    pub onconnect_hooks: Vec<
        Box<
            dyn Fn(
                    GMTS,
                    Arc<Mutex<tokio::net::TcpStream>>,
                    i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    >,
    pub earlyonconnect_hooks: Vec<
        Box<
            dyn Fn(
                    GMTS,
                    Arc<Mutex<tokio::net::TcpStream>>,
                    i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    >,
    pub packet_recv_hooks: HashMap<
        u8,
        Box<
            dyn Fn(
                    GMTS,
                    Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
                    u8,
                    i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    >,
    pub ondisconnect_hooks: Vec<Box<
        dyn Fn(
            GMTS,
            i8,
            ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
            + Send
            + Sync,
    >>,
    pub values: HashMap<String, GMTSElement>,
    pub extensions: HashMap<String, CPEExtensionData>,
    pub cpe_required: bool,
}
impl PreGMTS {
    pub fn new() -> Self {
        return Self {
            commands: HashMap::new(),
            pmta_hooks: Vec::new(),
            getblock_hooks: Vec::new(),
            setblock_hooks: Vec::new(),
            values: HashMap::new(),
            extensions: HashMap::new(),
            onconnect_hooks: Vec::new(),
            earlyonconnect_hooks: Vec::new(),
            packet_recv_hooks: HashMap::new(),
            ondisconnect_hooks: Vec::new(),
            cpe_required: false,
        };
    }
    pub fn register_command(
        &mut self,
        command: String,
        closure: Box<
            dyn Fn(CMDGMTS, Vec<String>, i8) -> Pin<Box<dyn Future<Output = usize> + Send>>
                + Send
                + Sync,
        >,
    ) {
        self.commands.insert(command, closure);
    }
    pub fn register_pmta_hook(
        &mut self,
        closure: Box<
            dyn Fn(CMDGMTS, PlayerCommand) -> Pin<Box<dyn Future<Output = PlayerCommand> + Send>>
                + Send
                + Sync,
        >,
    ) {
        self.pmta_hooks.push(closure);
    }
    pub fn register_getblock_hook(
        &mut self,
        closure: fn(CMDGMTS, BlockPosition) -> Pin<Box<dyn Future<Output = BlockPosition> + Send>>,
    ) {
        self.getblock_hooks.push(closure);
    }
    pub fn register_setblock_hook(
        &mut self,
        closure: Box<
            dyn Fn(
                    CMDGMTS,
                    Block,
                    u32,
                ) -> Pin<Box<dyn Future<Output = Option<(Block, u32)>> + Send>>
                + Send
                + Sync,
        >,
    ) {
        self.setblock_hooks.push(closure);
    }
    pub fn register_onconnect_hook(
        &mut self,
        closure: Box<
            dyn Fn(
                GMTS,
                Arc<Mutex<tokio::net::TcpStream>>,
                i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    ) {
        self.onconnect_hooks.push(closure);
    }
    pub fn register_ondisconnect_hook(
        &mut self,
        closure: Box<
            dyn Fn(
                GMTS,
                i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >,
    ) {
        self.ondisconnect_hooks.push(closure);
    }
    pub fn register_early_onconnect_hook(
        &mut self,
        closure: Box<
            dyn Fn(
                GMTS,
                Arc<Mutex<tokio::net::TcpStream>>,
                i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync
        >,
    ) {
        self.earlyonconnect_hooks.push(closure);
    }
    pub fn register_packet_hook(
        &mut self,
        id: u8,
        closure:             Box<
            dyn Fn(
                    GMTS,
                    Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
                    u8,
                    i8,
                ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
                + Send
                + Sync,
        >
    ) {
        self.packet_recv_hooks.insert(id, closure);
    }
    pub fn register_value(&mut self, name: &str, value: GMTSElement) -> Option<()> {
        if self.values.get(&name.to_string()).is_none() {
            self.values.insert(name.to_string(), value);
            return Some(());
        } else {
            return None;
        }
    }
    pub fn register_extension(&mut self, name: &str, version: usize, required: bool) -> Option<()> {
        if self.extensions.get(&name.to_string()).is_none() {
            let data = CPEExtensionData { version, required };
            self.extensions.insert(name.to_string(), data);
            return Some(());
        } else {
            return None;
        }
    }
    pub fn cpe_required(&mut self, val: bool) {
        self.cpe_required = val;
    }
}
impl GMTS {
    pub async fn setup(pre_gmts: PreGMTS) -> Self {
        // Initialize Physics Thread
        let (world_send, ph_recv) = mpsc::channel::<WorldCommand>(10000000);
        let (players_send, players_recv) = mpsc::channel::<PlayersCommand>(10000000);
        let (temp_crnt_id_send, tci_recv) = mpsc::channel::<TempCrntIdCommand>(10);
        let (storage_send, store_recv) = mpsc::channel::<StorageCommand>(10);
        let storage_send_2 = storage_send.clone();
        let cmd_gmts = CMDGMTS {
            world_send: world_send.clone(),
            players_send: players_send.clone(),
            tempcrntid_send: temp_crnt_id_send.clone(),
            storage_send: storage_send.clone(),
        };
        let storage = pre_gmts.values;
        let mut recv = store_recv;
        tokio::spawn(async move {
            let mut storage = storage;
            loop {
                match recv.recv().await.unwrap() {
                    StorageCommand::GetValue { key, res_send } => {
                        if let Some(x) = storage.get(&key) {
                            let x = x.clone();
                            res_send.send(Some(x)).ok().expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).ok().expect(ERR_SENDING_RESULT);
                        }
                    }
                    StorageCommand::SetValue {
                        key,
                        value,
                        res_send,
                    } => {
                        if let Some(x) = storage.get_mut(&key) {
                            *x = value;
                            res_send.send(Some(())).expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    StorageCommand::NewValue {
                        key,
                        value,
                        res_send,
                    } => {
                        if let None = storage.get(&key) {
                            storage.insert(key, value);
                            res_send.send(Some(())).expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    StorageCommand::RemoveValue { key, res_send } => {
                        res_send.send(if let Some(_) = storage.remove(&key) {
                            Some(())
                        } else {
                            None
                        }).expect(ERR_SENDING_RESULT);
                    }
                }
            }
        });
        // Initialize World Managing Task
        let mut recv = ph_recv;
        let setblock_hooks = pre_gmts.setblock_hooks;
        let cmd_gmts_2 = cmd_gmts.clone();
        tokio::spawn(async move {
            //let generator = FlatWorldGenerator::new(64, BlockIds::AIR, BlockIds::SAND, BlockIds::AIR);
            //let mut world = World::new(generator, 128, 128, 128);
            log::info!("Loading world from {}", &super::CONFIGURATION.world_file);
            let mut world = World::from_file(&super::CONFIGURATION.world_file).expect("Failed to init world");
            log::info!("Finished initializing world");
            loop {
                match recv.recv().await.unwrap() {
                    WorldCommand::GetBlock { pos, res_send } => {
                        if let Some(id) = world.get_block(pos.x, pos.y, pos.z) {
                            let block = Block { position: pos, id };
                            res_send.send(Some(block)).expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    WorldCommand::GetWorld { res_send } => {
                        if let Err(e) = res_send.send(world.clone()) {
                            panic!("Shouldn't fail!");
                        }
                    }
                    WorldCommand::GetSpawnPosition { res_send } => {
                        if let Err(e) = res_send.send(*world.get_world_spawnpos()) {
                            panic!("Shouldn't fail!");
                        }
                    }
                    WorldCommand::SaveWorld { res_send } => {
                        log::info!("Saving world");
                        res_send.send(world.save()).expect(ERR_SENDING_RESULT);
                    }
                    WorldCommand::SetBlockP {
                        block,
                        mut sender_id,
                        players_send,
                        res_send,
                    } => {
                        let mut none = false;
                        let mut block = block.clone();
                        for hook in &setblock_hooks {
                            let x = hook(cmd_gmts_2.clone(), block.clone(), sender_id).await;
                            let x = if let None = x {
                                none = true;
                                break;
                            } else {
                                x.unwrap()
                            };
                            block = x.0;
                            sender_id = x.1;
                        }
                        if !none {
                            if let Some(_) = world.set_block(block.clone()) {
                                let (res_send2, res_recv2) = oneshot::channel();
                                if let Ok(_) = players_send
                                    .send(PlayersCommand::PassMessageToAll {
                                        message: PlayerCommand::SetBlock { block },
                                        res_send: res_send2,
                                    })
                                    .await
                                {
                                    res_recv2.await.expect("Shouldn't fail");
                                    res_send.send(Some(())).expect(ERR_SENDING_RESULT);
                                } else {
                                    res_send.send(None).expect(ERR_SENDING_RESULT);
                                }
                            } else {
                                res_send.send(None).expect(ERR_SENDING_RESULT);
                            }
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                        /*                         if let None = function().await {
                            if let Some(x) = cmd_gmts_2.get_block(block.position).await {
                                cmd_gmts_2.block_to_id(x, sender_id as i8).await;
                            } else {
                                res_send.send(None).expect(ERR_SENDING_RESULT);
                            }
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(Some(())).expect(ERR_SENDING_RESULT);
                        } */
                    }
                }
            }
        });
        // Initialize Players Managing Task
        let mut recv = players_recv;
        let pmta_hooks = pre_gmts.pmta_hooks;
        let cmd_gmts_2 = cmd_gmts.clone();
        tokio::spawn(async move {
            let mut players: HashMap<u32, Player> = HashMap::new();
            let mut user_ids: HashMap<u32, String> = HashMap::new();
            loop {
                match recv.recv().await.unwrap() {
                    PlayersCommand::NewUser { user, res_send } => {
                        let id = user.id;
                        let name = user.name.clone();
                        let pos = user.data.position.clone();
                        for player in &mut players {
                            let x = player.1.message_send.send(PlayerCommand::SpawnPlayer {
                                position: pos.clone(),
                                id: (id as u8) as i8,
                                name: name.clone(),
                            });
                            if x.is_err() {
                                println!("Shouldn't fail");
                            }
                        }
                        user_ids.insert(id, name);
                        players.insert(id, user);
                        res_send.send(()).expect("Shouldn't fail");
                    }
                    PlayersCommand::RemoveUser { user_id, res_send } => {
                        players.remove(&user_id);
                        user_ids.remove(&user_id);
                        for player in &mut players {
                            player.1.message_send.send(PlayerCommand::DespawnPlayer {
                                id: (user_id as u8) as i8,
                            });
                        }
                        res_send.send(());
                    }
                    PlayersCommand::PassMessageToAll {
                        mut message,
                        res_send,
                    } => {
                        for hook in &pmta_hooks {
                            message = hook(cmd_gmts_2.clone(), message).await
                        }
                        for player in &mut players {
                            player.1.message_send.send(message.clone());
                        }
                        res_send.send(());
                    } // ...
                    PlayersCommand::SpawnAllPlayers { my_id, res_send } => {
                        let us = players.get(&my_id);
                        if us.is_none() {
                            res_send.send(());
                        } else {
                            let us = us.unwrap();
                            for player in &players {
                                if player.1.id != us.id {
                                    us.message_send.send(PlayerCommand::SpawnPlayer {
                                        position: player.1.data.position.clone(),
                                        id: (player.1.id as u8) as i8,
                                        name: player.1.name.clone(),
                                    });
                                } else {
                                    us.message_send.send(PlayerCommand::SpawnPlayer {
                                        position: player.1.data.position.clone(),
                                        id: -1,
                                        name: player.1.name.clone(),
                                    });
                                }
                            }
                            res_send.send(());
                        }
                    }
                    PlayersCommand::IsOperator { id, res_send } => {
                        let us = players.get(&id);
                        if us.is_none() {
                            res_send.send(false).expect(ERR_SENDING_RESULT);
                        } else {
                            let us = us.unwrap();
                            res_send.send(us.op).expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::OnlinePlayerCount { res_send } => {
                        res_send.send(players.len());
                    }
                    PlayersCommand::SetPermissionLevel { id, level, res_send } => {
                        if let Some(us) = players.get_mut(&id) {
                            us.permission_level = level;
                            res_send
                                .send(Some(()))
                                .expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::GetPermissionLevel { id, res_send } => {
                        if let Some(us) = players.get(&id) {
                            res_send
                                .send(Some(us.permission_level))
                                .expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::GetSupportedExtensions { id, res_send } => {
                        if let Some(us) = players.get(&id) {
                            res_send
                                .send(us.supported_extensions.clone())
                                .ok()
                                .expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).ok().expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::UpdatePosition {
                        my_id,
                        position,
                        res_send,
                    } => {
                        if let Some(us) = players.get(&my_id) {
                            let id = us.id.clone();
                            drop(us);
                            for player in &players {
                                if player.1.id != id {
                                    if let None = player.1.send_teleport(my_id as i8, &position) {
                                        log::error!(
                                            "Error sending teleport of entity {} to position {:?}",
                                            my_id as i8,
                                            position
                                        );
                                    }
                                }
                            }
                            res_send.send(Some(())).expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::GetUsername { id, res_send } => {
                        if let Some(user) = players.get(&id) {
                            res_send
                                .send(Some(user.name.clone()))
                                .expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::GetPosition { id, res_send } => {
                        if let Some(user) = players.get(&id) {
                            res_send
                                .send(Some(user.data.position.clone()))
                                .expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::PassMessageToID {
                        id,
                        message,
                        res_send,
                    } => {
                        if let Some(user) = players.get(&id) {
                            user.message_send.send(message).expect(ERR_SENDING_RESULT);
                            res_send.send(Some(())).expect(ERR_SENDING_RESULT);
                        } else {
                            res_send.send(None).expect(ERR_SENDING_RESULT);
                        }
                    }
                    PlayersCommand::KickUserByName {
                        username,
                        reason,
                        res_send,
                    } => {
                        let mut f = false;
                        for (id, user) in &user_ids {
                            if user == &username {
                                let user = players.get(&id).unwrap();
                                user.message_send
                                    .send(PlayerCommand::Disconnect { reason })
                                    .expect("Shouldn't fail");
                                f = true;
                                break;
                            }
                        }
                        if f {
                            res_send.send(Some(())).expect("Shouldn't fail");
                        } else {
                            res_send.send(None).expect("Shouldn't fail");
                        }
                    }
                    PlayersCommand::GetID { username, res_send } => {
                        let mut f = false;
                        let mut id_f = 0;
                        for (id, user) in &user_ids {
                            if user == &username {
                                let user = players.get(&id).unwrap();
                                id_f = user.id;
                                f = true;
                                break;
                            }
                        }
                        if f {
                            res_send.send(Some(id_f as i8)).expect("Shouldn't fail");
                        } else {
                            res_send.send(None).expect("Shouldn't fail");
                        }
                    }
                    PlayersCommand::PassMessageByName {
                        username,
                        message,
                        res_send,
                    } => {
                        let mut f = false;
                        for (id, user) in &user_ids {
                            if user == &username {
                                let user = players.get(&id).unwrap();
                                user.message_send.send(message).expect("Shouldn't fail");
                                f = true;
                                break;
                            }
                        }
                        if f {
                            res_send.send(Some(())).expect("Shouldn't fail");
                        } else {
                            res_send.send(None).expect("Shouldn't fail");
                        }
                    }
                }
            }
        });
        // Initialize Temp Crnt Id Managing Task
        let mut recv = tci_recv;
        tokio::spawn(async move {
            let mut ids = vec![0; 127];
            for i in 0..127 {
                ids[i] = i;
            }
            loop {
                match recv.recv().await.unwrap() {
                    TempCrntIdCommand::FetchFreeID { res_send } => {
                        res_send
                            .send(ids.pop().unwrap() as u32)
                            .expect("Shouldn't fail");
                    } // ...
                    TempCrntIdCommand::ReturnFreeID { id, res_send } => {
                        ids.push(id as usize);
                        res_send.send(()).expect("Shouldn't fail");
                    }
                }
            }
        });
        let (commands_send, mut recv) = mpsc::channel::<CommandsCommand>(10);
        let commands = pre_gmts.commands;
        tokio::spawn(async move {
            loop {
                match recv.recv().await.unwrap() {
                    CommandsCommand::SendCommand {
                        mut command,
                        executor_id,
                    } => {
                        command.remove(0);
                        let sender_name = cmd_gmts.get_username(executor_id).await.expect("Shouldn't fail! Bug happened!");
                        log::info!(r#"{} executed server command "/{}""#, sender_name, command);
                        let command = command
                            .split(" ")
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>();
                        if let Some(c) = commands.get(&command[0]) {
                            let x = c(cmd_gmts.clone(), command[1..].to_vec(), executor_id).await;
                            match x {
                                0 => (),
                                1 => {
                                    cmd_gmts
                                        .chat_to_id(&format!("&cInvalid syntax."), -1, executor_id)
                                        .await;
                                }
                                2 => {
                                    cmd_gmts
                                        .chat_to_id(
                                            &format!("&cInsufficient permission."),
                                            -1,
                                            executor_id,
                                        )
                                        .await;
                                }
                                x => {
                                    cmd_gmts.chat_to_id(&format!("&cAn unexpected error occured. (Command returned code {})", x), -1, executor_id).await;
                                }
                            }
                            //test_ref_input_as_fntrait(&|| c(cmd_gmts.clone(), command[1..].join(""), executor_id);
                        } else {
                            cmd_gmts.chat_to_id(UNKNOWN_COMMAND, -1, executor_id).await;
                        }
                    }
                }
            }
        });

        GMTS {
            world_send,
            players_send,
            tempcrntid_send: temp_crnt_id_send,
            commands_send,
            storage_send: storage_send_2,
            extensions: pre_gmts.extensions,
            cpe_required: pre_gmts.cpe_required,
            onconnect_hooks: Arc::new(pre_gmts.onconnect_hooks),
            earlyonconnect_hooks: Arc::new(pre_gmts.earlyonconnect_hooks),
            packet_recv_hooks: Arc::new(pre_gmts.packet_recv_hooks),
            ondisconnect_hooks: Arc::new(pre_gmts.ondisconnect_hooks),
        }
    }
    //    pub extensions: HashMap<String, CPEExtensionData>,
    pub async fn chat_broadcast(&self, message: &str, id: i8) -> Option<()> {
        log::info!("[CHAT]: {}", message);
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::Message {
            id: (id as u8) as i8,
            message: message.to_string(),
        };
        self.players_send
            .send(PlayersCommand::PassMessageToAll { message, res_send })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn cpe_required(&self) -> &bool {
        &self.cpe_required
    }
    pub async fn get_extensions(&self) -> &HashMap<String, CPEExtensionData> {
        &self.extensions
    }
    pub async fn get_packetrecv_hooks(&self) -> Arc<HashMap<u8, Box<
        dyn Fn(
            GMTS,
            Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
            u8,
            i8,
            ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
            + Send
            + Sync,
    >>> {
        self.packet_recv_hooks.clone()
    }
    pub async fn get_ondisconnect_hooks(&self) -> Arc<Vec<Box<
        dyn Fn(
            GMTS,
            i8,
            ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
            + Send
            + Sync,
    >>> {
        self.ondisconnect_hooks.clone()
    }
    pub async fn get_onconnect_hooks(&self) -> Arc<Vec<Box<
        dyn Fn(
            GMTS,
            Arc<Mutex<tokio::net::TcpStream>>,
            i8,
            ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
            + Send
            + Sync,
    >>> {
        self.onconnect_hooks.clone()
    }
    pub async fn get_earlyonconnect_hooks(&self) -> Arc<Vec<Box<
        dyn Fn(
            GMTS,
            Arc<Mutex<tokio::net::TcpStream>>,
            i8,
            ) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>
            + Send
            + Sync,
    >>> {
        self.earlyonconnect_hooks.clone()
    }
    pub async fn get_value(&self, key: &str) -> Option<GMTSElement> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::GetValue {
                key: key.to_string(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn rem_value(&self, key: &str) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::RemoveValue {
                key: key.to_string(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn new_value(&self, key: &str, value: GMTSElement) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::NewValue {
                key: key.to_string(),
                value,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn set_value(&self, key: &str, value: GMTSElement) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.storage_send
            .send(StorageCommand::SetValue {
                key: key.to_string(),
                value,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn get_spawnpos(&self) -> Option<PlayerPosition> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::GetSpawnPosition {
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn player_count(&self) -> Option<usize> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::OnlinePlayerCount {
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn message_to_id(&self, message: PlayerCommand, target_id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::PassMessageToID {
                message,
                id: target_id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn block_to_id(&self, block: Block, target_id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::SetBlock { block };
        self.players_send
            .send(PlayersCommand::PassMessageToID {
                message,
                id: target_id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn get_block(&self, block: BlockPosition) -> Option<Block> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::GetBlock {
                pos: block,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn chat_to_id(&self, message: &str, id: i8, target_id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::Message {
            id: (id as u8) as i8,
            message: message.to_string(),
        };
        self.players_send
            .send(PlayersCommand::PassMessageToID {
                message,
                id: target_id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn chat_to_username(
        &self,
        message: &str,
        id: i8,
        target_username: String,
    ) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        let message = PlayerCommand::Message {
            id: (id as u8) as i8,
            message: message.to_string(),
        };
        self.players_send
            .send(PlayersCommand::PassMessageByName {
                message,
                username: target_username,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?;
        Some(())
    }
    pub async fn remove_user(&self, id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::RemoveUser {
                user_id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn return_id(&self, id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.tempcrntid_send
            .send(TempCrntIdCommand::ReturnFreeID {
                id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn get_unused_id(&self) -> Option<u32> {
        let (res_send, res_recv) = oneshot::channel();
        self.tempcrntid_send
            .send(TempCrntIdCommand::FetchFreeID { res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn pass_message_to_all(&self, message: PlayerCommand) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::PassMessageToAll { message, res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn get_username(&self, id: i8) -> Option<String> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::GetUsername {
                id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn send_position_update(&self, id: i8, position: PlayerPosition) {
        let (res_send, res_recv) = oneshot::channel();
        let x = self
            .players_send
            .send(PlayersCommand::UpdatePosition {
                my_id: id as u32,
                position,
                res_send,
            })
            .await;
        if x.is_err() {
            panic!("Error sending position update!");
        }
        res_recv.await.expect("Error sending position update!");
    }
    pub async fn get_supported_extensions(&self, id: i8) -> Option<HashMap<String, CPEExtensionData>> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::GetSupportedExtensions {
                id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn set_block(&self, block: &Block, sender_id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::SetBlockP {
                block: block.clone(),
                sender_id: sender_id as u32,
                players_send: self.players_send.clone(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn save_world(&self) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::SaveWorld { res_send })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn get_world(&self) -> Option<World> {
        let (res_send, res_recv) = oneshot::channel();
        self.world_send
            .send(WorldCommand::GetWorld { res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn kick_user_by_name(&self, name: &str, reason: &str) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::KickUserByName {
                username: name.to_string(),
                reason: reason.to_string(),
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()?
    }
    pub async fn register_user(&self, user: Player) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        let pos = user.data.position.clone();
        self.players_send
            .send(PlayersCommand::NewUser { user, res_send })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn spawn_all_players(&self, id: i8) -> Option<()> {
        let (res_send, res_recv) = oneshot::channel();
        self.players_send
            .send(PlayersCommand::SpawnAllPlayers {
                my_id: id as u32,
                res_send,
            })
            .await
            .ok()?;
        res_recv.await.ok()
    }
    pub async fn execute_command(&self, id: i8, command: String) -> Option<()> {
        self.commands_send
            .send(CommandsCommand::SendCommand {
                executor_id: id,
                command,
            })
            .await
            .ok()?;
        Some(())
    }
}
/*
let (res_send, res_recv) = oneshot::channel();
let x = gmts
  .players_send
  .send(PlayersCommand::UpdatePosition {
    my_id: our_id,
    position,
    res_send,
  })
  .await;
if x.is_err() {
  panic!("Shouldn't fail!");
}
res_recv.await.unwrap();*/
pub enum PhysicsCommand {
    PropogateFluid { block: Block },
    PropogateGravityFast { block: Block },
    PropogateGravityFancy { block: Block },
}
pub enum SpleefCommand {
    FillArena,
    StartFall { pos: BlockPosition },
}
// block is already defined
#[derive(Clone)]
pub enum PlayerCommand {
    SetBlock {
        block: Block,
    },
    SpawnPlayer {
        position: PlayerPosition,
        id: i8,
        name: String,
    },
    DespawnPlayer {
        id: i8,
    },
    PlayerTeleport {
        position: PlayerPosition,
        id: i8,
    },
    Message {
        id: i8,
        message: String,
    },
    Disconnect {
        reason: String,
    },
}
pub struct ExtEntry {
    extname: String,
    version: i32,
}
pub enum HeartbeatCommand {}
pub enum WorldCommand {
    GetBlock {
        pos: BlockPosition,
        res_send: oneshot::Sender<Option<Block>>,
    },
    SetBlockP {
        block: Block,
        players_send: mpsc::Sender<PlayersCommand>,
        sender_id: u32,
        res_send: oneshot::Sender<Option<()>>,
    },
    GetWorld {
        res_send: oneshot::Sender<World>,
    },
    GetSpawnPosition {
        res_send: oneshot::Sender<Option<PlayerPosition>>,
    },
    SaveWorld {
        res_send: oneshot::Sender<Option<()>>,
    },
}
pub enum CommandsCommand {
    SendCommand { command: String, executor_id: i8 },
}
pub enum StorageCommand {
    GetValue {
        key: String,
        res_send: oneshot::Sender<Option<GMTSElement>>,
    },
    SetValue {
        key: String,
        value: GMTSElement,
        res_send: oneshot::Sender<Option<()>>,
    },
    RemoveValue {
        key: String,
        res_send: oneshot::Sender<Option<()>>,
    },
    NewValue {
        key: String,
        value: GMTSElement,
        res_send: oneshot::Sender<Option<()>>,
    },
}
pub enum PlayersCommand {
    /*   GetUserPos {
      user_id: u32,
      res_send: oneshot::Sender<PlayerPosition>
    }, */
    NewUser {
        user: Player,
        res_send: oneshot::Sender<()>,
    },
    RemoveUser {
        user_id: u32,
        res_send: oneshot::Sender<()>,
    },
    PassMessageToAll {
        message: PlayerCommand,
        res_send: oneshot::Sender<()>,
    },
    SpawnAllPlayers {
        my_id: u32,
        res_send: oneshot::Sender<()>,
    },
    UpdatePosition {
        my_id: u32,
        position: PlayerPosition,
        res_send: oneshot::Sender<Option<()>>,
    },
    GetPosition {
        id: u32,
        res_send: oneshot::Sender<Option<PlayerPosition>>,
    },
    GetUsername {
        id: u32,
        res_send: oneshot::Sender<Option<String>>,
    },
    OnlinePlayerCount {
        res_send: oneshot::Sender<usize>,
    },
    GetID {
        username: String,
        res_send: oneshot::Sender<Option<i8>>,
    },
    PassMessageToID {
        id: u32,
        message: PlayerCommand,
        res_send: oneshot::Sender<Option<()>>,
    },
    IsOperator {
        id: u32,
        res_send: oneshot::Sender<bool>,
    },
    SetPermissionLevel {
        id: u32,
        level: usize,
        res_send: oneshot::Sender<Option<()>>,
    },
    GetPermissionLevel {
        id: u32,
        res_send: oneshot::Sender<Option<usize>>,
    },
    GetSupportedExtensions {
        id: u32,
        res_send: oneshot::Sender<Option<HashMap<String, CPEExtensionData>>>,
    },
    KickUserByName {
        username: String,
        reason: String,
        res_send: oneshot::Sender<Option<()>>,
    },
    PassMessageByName {
        username: String,
        message: PlayerCommand,
        res_send: oneshot::Sender<Option<()>>,
    },
}
// Honestly the code looks as ugly as a raw Future implemntation
// We could probably write some macros to insert the ugly code... in the future.

pub enum TempCrntIdCommand {
    FetchFreeID {
        res_send: oneshot::Sender<u32>,
    },
    ReturnFreeID {
        id: u32,
        res_send: oneshot::Sender<()>,
    },
}