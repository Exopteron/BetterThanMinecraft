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

use tokio::net::{TcpListener, TcpStream};

/* ================================================ maths.rs ================================================ */

pub struct BlockPosition(usize, usize, usize);

pub struct Position(f32, f32, f32);

impl Position {
  pub fn distance_from_squared(&self, other: Position) -> f32 {
    (self.x - other.x).pow(2) + (self.y - other.y).pow(2) + (self.z - other.z).pow(2)
  }
  pub fn distance_from_block_squared(&self, other: BlockPosition) -> usize {
    usize doesnt have .sqrt, oh well
    (self.x as usize - other.x).pow(2) + (self.y as usize - other.y).pow + (self.z as usize - other.z).pow(2)
  }
  pub fn distance_from(&self, other: Position) -> f32 {
    self.distance_from_squared(other).sqrt()
  }
}

pub struct PlayerTransform {
  pos: Position,
  yaw: f32,
  pitch: f32,
  size: f32,
}

impl PlayerTransform {
  pub fn looking_at(&self, target: BlockPosition) -> bool {
    /*insert fancy vector maths*/
  }

  pub fn distance_from_player(&self, other: PlayerTransform) -> f32 {
    
  }
  pub fn distance_from_squared(&self, other: BlockPosition) -> usize {
    (other.x - self.x.floor()).pow(2) + (other.y - self.y.floor()).pow(2) + (other.z - self.z.floor())

    return ((other.x as f32 - self.x as usize).pow(2) + (target.y as usize - self.y as usize).pow(2) + (target.z as usize + self.z as usize).pow(2)) / 2;
  }
}

fn f32_to_fixed(x: f32) -> i16 {
  return (x * 32).round() as i16
}

pub struct PlayerPosition {
    x: Short,
    y: Short,
    z: Short,
    yaw: u8,
    pitch; u8
}
type BlockID = u8;
pub struct Block {
    position: BlockPosition,
    id: BlockID
}

pub struct World {
  
}

pub struct Message {
    sender_id: u32,
    message: String
}
pub struct PlayerData {
    chat_box: Vec<Message>,
    position: PlayerTransform
}
pub struct Player {
    data: Arc<Mutex<PlayerData>>,
    // unique identifier, shorthand
    id: u32,
    protocol: Protocol,
}
impl Player {
  pub fn get_position(&self) -> PlayerTransform {
    let data = self.data.lock().await;
    return data.position.clone();
  }
}
pub enum Protocol {
  Classic {plr_id: u8}
}

use tokio::sync::mpsc;
pub struct GameManagerSenders {
  world: mpsc::Sender,
  players: mpsc::Sender,
  userids: mpsc::Sender,
  tempcrntid: mpsc::Sender,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

  let world = World {};
  let (send, recv) = mpsc::new();
  tokio::spawn(async {

  });

  let players = Vec::new::<Player>();
  let userids = HashMap::new::<u32, String>();
  let temp_current_id: u32 = 0;
  
  let listener = TcpListener::bind("0.0.0.0:25565").await?;
  loop {
    let (stream, addr) = listener.accept().await;
    tokio::spawn(async move {
      if let Err(e) = incoming_connection_handler(stream).await {
        eprintln!("An error occured. {:?}", e);
      }
    });
  }  
}

async fn incoming_connection_handler(stream: TcpStream) -> Result<(), Box<dyn Error>> {
  
  Ok(())
}
















pub mod classic {
  pub enum Packet {
    PlayerIdentificaction {p_ver: u8, user_name: String, v_key: String, unchanged: u8 /*technically, not going to leave it in, just for a point*/},
    ServerIdentificaction {p_ver: u8, server_name: String, motd: String, is_op: u8},
    LevelInitialize,
    LevelDataChunk { chunk_length: i16, chunk_data: Box<[u8]>, percent_complete: u8},
    LevelFinalize { width: usize, height: usize, length: usize},
  }
}
  pub struct ClassicPacketUtils {

}
impl ClassicPacketUtils {
  async fn read_byte<'a>(reader:  &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>) -> std::io::Result<u8> {
    let mut byte = [0; 1];
    reader.read_exact(&mut byte).await?;
    return Ok(byte[0]);
  }
  async fn read_short<'a>(reader:  &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>) -> std::io::Result<i16> {
    let mut byte = [0; 2];
    reader.read_exact(&mut byte).await?;
    let short = i16::from_be_bytes(byte);
    return Ok(short);
  }
  async fn read_string<'a>(reader:  &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>) -> std::io::Result<String> {
    let mut byte = [0; 64];
    reader.read_exact(&mut byte).await?;
    let string = String::from_utf8_lossy(&byte).to_string();
    return Ok(string.trim_matches(char::from(0x20)).to_string());
  }
}
#[derive(Clone)]
pub enum Element {
  Byte { byte: u8 },
  SByte { byte: i8 },
  StringElement { string: String },
  Short { short: i16 },
  Bytes { bytes: Vec<u8> }
}
pub struct ClassicPacketBuilder {
  elements: Vec<Element>,
}
impl ClassicPacketBuilder {
  pub fn new() -> Self {
    return Self {
      elements: Vec::new(),
    };
  }
  pub fn insert_string(&mut self, string: &str) {
    self.elements.push(Element::StringElement {
      string: string.to_string(),
    });
  }
  pub fn insert_bytearray(&mut self, bytes: Vec<u8>) {
    self.elements
      .push(Element::Bytes { bytes: bytes });
  }
  pub fn insert_sbyte(&mut self, byte: i8) {
    self.elements.push(Element::SByte { byte: byte });
  }
  pub fn insert_byte(&mut self, byte: u8) {
    self.elements.push(Element::Byte { byte: byte });
  }
  pub fn insert_short(&mut self, short: i16) {
    self.elements.push(Element::Short { short: short });
  }
  pub fn build(mut self, id: u8) -> std::io::Result<Vec<u8>> {
    let mut packet = vec![id];
    packet.append(&mut self.internal_builder()?);
    return Ok(packet);
  }
  fn internal_builder(&mut self) -> std::io::Result<Vec<u8>> {
    let mut packet = vec![];
    for element in self.elements.clone() {
      match element.clone() {
        Element::StringElement { string } => {
          if string.len() > 64 {
            return Err(Error::new(ErrorKind::Other, "String too large!"));
          }
          let mut string = string.as_bytes().to_vec();
          for _ in 0..64 - string.len() {
            string.push(0x20);
          }
          packet.append(&mut string);
        }
        Element::Byte { byte } => {
          packet.push(byte);
        }
        Element::SByte { byte } => {
          packet.push(byte.to_le_bytes()[0]);
        }
        Element::Short { short } => {
          packet.append(&mut short.to_be_bytes().to_vec());
        }
        Element::Bytes { mut bytes } => {
          if bytes.len() > 1024 {
            return Err(Error::new(ErrorKind::Other, "Bytes too large!"));
          }
          for _ in bytes.len()..1024 {
            bytes.push(0x00);
          }
          packet.append(&mut bytes);
        }
      }
    }
    return Ok(packet);
  }
}

}