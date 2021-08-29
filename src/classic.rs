use super::{game::*, Block, BlockPosition, PlayerPosition};
// common:               p_ver           name             string        isop, client just says theyre not op
// it's unused, doesn't need to be there
// anything marked as "unused" is a reverse engineering byproduct, it should more accurately be:
// "unchanging" as that's what they observed to give it the name of "unused"
// and an unchanged permission is valid.
pub enum Packet {
  PlayerIdentification {
    p_ver: u8,
    user_name: String,
    v_key: String, /*technically, not going to leave it in, just for a point*/
    cpe_id: u8,
  },
  ServerIdentification {
    p_ver: u8,
    server_name: String,
    motd: String,
    is_op: u8,
  },
  LevelInitialize,
  LevelDataChunk {
    chunk_length: i16,
    chunk_data: Box<[u8]>,
    percent_complete: u8,
  },
  LevelFinalize {
    width: usize,
    height: usize,
    length: usize,
  },
  PositionAndOrientationC {
    player_id: u8,
    position: PlayerPosition,
  },
  SetBlockC {
    coords: BlockPosition,
    mode: u8,
    block_type: u8,
  },
  MessageC {
    message: String,
    unused: u8,
  },
  SetBlockS {
    block: Block,
  },
  PlayerTeleportS {
    player_id: i8,
    position: PlayerPosition,
  },
  SpawnPlayer {
    player_id: i8,
    name: String,
    position: PlayerPosition,
  },
  DespawnPlayer {
    player_id: i8,
  },
  Message {
    player_id: i8,
    message: String,
  },
  Disconnect {
    reason: String,
  },
  ExtInfo {
    appname: String,
    extension_count: i16,
  },
  ExtEntry {
    extname: String,
    version: i32,
  },
  CustomBlockSupportLevel {
    support_level: u8,
  },
  SetBlockPermission {
    block_type: u8,
    allow_placement: u8,
    allow_deletion: u8,
  },
  PlayerClicked {
    button: u8,
    action: u8,
    yaw: i16,
    pitch: i16,
    target_entity_id: u8,
    target_block_x: i16,
    target_block_y: i16,
    target_block_z: i16,
    target_block_face: u8,
  },
  UpdateUserType {
    user_type: u8,
  }
}

use std::pin::Pin;
use tokio::io::AsyncReadExt;
pub struct ClassicPacketWriter {}
impl ClassicPacketWriter {
  pub fn serialize(packet: Packet) -> std::io::Result<Vec<u8>> {
    match packet {
      Packet::UpdateUserType { user_type } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_byte(user_type);
        return builder.build(0x0f);
      }
      Packet::LevelInitialize => {
        let builder = ClassicPacketBuilder::new();
        return builder.build(0x02);
      }
      Packet::SetBlockPermission { block_type, allow_placement, allow_deletion } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_byte(block_type);
        builder.insert_byte(allow_placement);
        builder.insert_byte(allow_deletion);
        return builder.build(0x1C);
      }
      Packet::CustomBlockSupportLevel { support_level } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_byte(support_level);
        return builder.build(0x13);
      }
      Packet::LevelDataChunk {
        chunk_length,
        chunk_data,
        percent_complete,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_short(chunk_length as i16);
        builder.insert_bytearray(chunk_data.to_vec());
        builder.insert_byte(percent_complete);
        return builder.build(0x03);
      }
      Packet::ExtInfo {
        appname,
        extension_count,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_string(&appname);
        builder.insert_short(extension_count);
        return builder.build(0x10);
      }
      Packet::ExtEntry {
        extname,
        version,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_string(&extname);
        builder.insert_int(version);
        return builder.build(0x11);
      }

      Packet::LevelFinalize {
        width,
        height,
        length,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_short(width as i16);
        builder.insert_short(height as i16);
        builder.insert_short(length as i16);
        return builder.build(0x04);
      }
      Packet::SetBlockS { block } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_short(block.position.x as i16);
        builder.insert_short(block.position.y as i16);
        builder.insert_short(block.position.z as i16);
        builder.insert_byte(block.id);
        return Ok(builder.build(0x06)?);
      }
      Packet::PlayerTeleportS {
        player_id,
        position,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_short(position.x as i16);
        builder.insert_short(position.y as i16);
        builder.insert_short(position.z as i16);
        builder.insert_byte(position.yaw);
        builder.insert_byte(position.pitch);
        return Ok(builder.build(0x08)?);
      }
      Packet::SpawnPlayer {
        player_id,
        name,
        position,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_string(&name);
        builder.insert_short(position.x as i16);
        builder.insert_short(position.y as i16);
        builder.insert_short(position.z as i16);
        builder.insert_byte(position.yaw);
        builder.insert_byte(position.pitch);
        return Ok(builder.build(0x07)?);
      }
      Packet::DespawnPlayer { player_id } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        return Ok(builder.build(0x0c)?);
      }
      Packet::Message { player_id, message } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_string(&message);
        return Ok(builder.build(0x0d)?);
      }
      Packet::Disconnect { reason } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_string(&reason);
        return Ok(builder.build(0x0e)?);
      }
      _ => {
        return Err(std::io::Error::new(
          ErrorKind::Other,
          format!("Unknown packet!"),
        ));
      }
    }
  }
  pub fn server_identification(
    protocol_ver: u8,
    server_name: String,
    motd: String,
    is_op: bool,
  ) -> std::io::Result<Vec<u8>> {
    let mut builder = ClassicPacketBuilder::new();
    builder.insert_byte(protocol_ver);
    builder.insert_string(&server_name);
    builder.insert_string(&motd);
    match is_op {
      true => {
        builder.insert_byte(0x64);
      }
      false => {
        builder.insert_byte(0x00);
      }
    }
    return Ok(builder.build(0x00)?);
  }
  pub fn serialize_vec(vec: Vec<Packet>) -> std::io::Result<Vec<Vec<u8>>> {
    let mut vec2 = vec![];
    for packet in vec {
      match packet {
        Packet::LevelInitialize => {
          let builder = ClassicPacketBuilder::new();
          vec2.push(builder.build(0x02)?);
        }
        Packet::LevelDataChunk {
          chunk_length,
          chunk_data,
          percent_complete,
        } => {
          let mut builder = ClassicPacketBuilder::new();
          builder.insert_short(chunk_length as i16);
          builder.insert_bytearray(chunk_data.to_vec());
          builder.insert_byte(percent_complete);
          vec2.push(builder.build(0x03)?);
        }
        Packet::LevelFinalize {
          width,
          height,
          length,
        } => {
          let mut builder = ClassicPacketBuilder::new();
          builder.insert_short(width as i16);
          builder.insert_short(height as i16);
          builder.insert_short(length as i16);
          vec2.push(builder.build(0x04)?);
        }
        _ => {
          return Err(std::io::Error::new(
            ErrorKind::Other,
            format!("Unknown packet"),
          ));
        }
      }
    }
    Ok(vec2)
  }
}
pub struct ClassicPacketReader {}
impl ClassicPacketReader {
  pub async fn read_packet_reader<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<Packet> {
    let id = ClassicPacketUtils::read_byte(reader).await?;
    match id {
      0x00 => {
        let protocol_ver = ClassicPacketUtils::read_byte(reader).await?;
        let username = ClassicPacketUtils::read_string(reader).await?;
        let verification_key = ClassicPacketUtils::read_string(reader).await?;
        let unused = ClassicPacketUtils::read_byte(reader).await?;
        let packet = Packet::PlayerIdentification {
          p_ver: protocol_ver,
          user_name: username,
          v_key: verification_key,
          cpe_id: unused,
        };
        return Ok(packet);
      }
      0x22 => {
        let button = ClassicPacketUtils::read_byte(reader).await?;
        let action = ClassicPacketUtils::read_byte(reader).await?;
        let yaw = ClassicPacketUtils::read_short(reader).await?;
        let pitch = ClassicPacketUtils::read_short(reader).await?;
        let target_entity_id = ClassicPacketUtils::read_byte(reader).await?;
        let target_block_x = ClassicPacketUtils::read_short(reader).await?;
        let target_block_y = ClassicPacketUtils::read_short(reader).await?;
        let target_block_z = ClassicPacketUtils::read_short(reader).await?;
        let target_block_face = ClassicPacketUtils::read_byte(reader).await?;
        let packet = Packet::PlayerClicked {
          button,
          action,
          yaw,
          pitch,
          target_entity_id,
          target_block_x,
          target_block_y,
          target_block_z,
          target_block_face
        };
        return Ok(packet);
      }
      0x13 => {
        let support_level = ClassicPacketUtils::read_byte(reader).await?;
        let packet = Packet::CustomBlockSupportLevel {
          support_level
        };
        return Ok(packet);
      }
      0x10 => {
        let appname = ClassicPacketUtils::read_string(reader).await?;
        let extension_count = ClassicPacketUtils::read_short(reader).await?;
        let packet = Packet::ExtInfo {
          appname,
          extension_count
        };
        return Ok(packet);
      }
      0x11 => {
        let extname = ClassicPacketUtils::read_string(reader).await?;
        let version = ClassicPacketUtils::read_int(reader).await?;
        let packet = Packet::ExtEntry {
          extname,
          version
        };
        return Ok(packet);
      }
      0x08 => {
        let pid = ClassicPacketUtils::read_byte(reader).await?;
        let x = ClassicPacketUtils::read_short(reader).await?;
        let y = ClassicPacketUtils::read_short(reader).await?;
        let z = ClassicPacketUtils::read_short(reader).await?;
        let yaw = ClassicPacketUtils::read_byte(reader).await?;
        let pitch = ClassicPacketUtils::read_byte(reader).await?;
        let coords = PlayerPosition {
          x: x as u16,
          y: y as u16,
          z: z as u16,
          yaw: yaw,
          pitch: pitch,
        };
        let packet = Packet::PositionAndOrientationC {
          player_id: pid,
          position: coords,
        };
        return Ok(packet);
      }
      0x05 => {
        let x = ClassicPacketUtils::read_short(reader).await?;
        let y = ClassicPacketUtils::read_short(reader).await?;
        let z = ClassicPacketUtils::read_short(reader).await?;
        let mode = ClassicPacketUtils::read_byte(reader).await?;
        let blocktype = ClassicPacketUtils::read_byte(reader).await?;
        let coords = BlockPosition {
          x: x as usize,
          y: y as usize,
          z: z as usize,
        };
        let packet = Packet::SetBlockC {
          coords: coords,
          mode: mode,
          block_type: blocktype,
        };
        return Ok(packet);
      }
      0x0d => {
        let x = ClassicPacketUtils::read_byte(reader).await?;
        let message: String;
        if x == 0x01 {
          message = ClassicPacketUtils::read_string_trimless(reader).await?;
        } else {
          message = ClassicPacketUtils::read_string(reader).await?;
        }
        let packet = Packet::MessageC { message: message, unused: x };
        return Ok(packet);
      }
      id => {
        return Err(std::io::Error::new(
          ErrorKind::Other,
          format!("Unknown packet id {}!", id),
        ));
      }
    }
  }
}
pub struct ClassicPacketUtils {}
impl ClassicPacketUtils {
  async fn read_byte<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<u8> {
    let mut byte = [0; 1];
    reader.read_exact(&mut byte).await?;
    return Ok(byte[0]);
  }
  async fn read_short<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<i16> {
    let mut byte = [0; 2];
    reader.read_exact(&mut byte).await?;
    let short = i16::from_be_bytes(byte);
    return Ok(short);
  }
  async fn read_int<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<i32> {
    let mut byte = [0; 4];
    reader.read_exact(&mut byte).await?;
    let short = i32::from_be_bytes(byte);
    return Ok(short);
  }
  async fn read_string<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<String> {
    let mut byte = [0; 64];
    reader.read_exact(&mut byte).await?;
    let string = String::from_utf8_lossy(&byte).to_string();
    return Ok(string.trim_matches(char::from(0x20)).to_string());
  }
  async fn read_string_trimless<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<String> {
    let mut byte = [0; 64];
    reader.read_exact(&mut byte).await?;
    let string = String::from_utf8_lossy(&byte).to_string();
    return Ok(string);
  }
}
use std::io::ErrorKind;
#[derive(Clone)]
pub enum Element {
  Byte { byte: u8 },
  SByte { byte: i8 },
  Int { int: i32 },
  StringElement { string: String },
  Short { short: i16 },
  Bytes { bytes: Vec<u8> },
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
    self.elements.push(Element::Bytes { bytes: bytes });
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
  pub fn insert_int(&mut self, int: i32) {
    self.elements.push(Element::Int { int: int });
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
            return Err(std::io::Error::new(ErrorKind::Other, "String too large!"));
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
        Element::Int { int } => {
          packet.append(&mut int.to_be_bytes().to_vec());
        }
        Element::Bytes { mut bytes } => {
          if bytes.len() > 1024 {
            return Err(std::io::Error::new(ErrorKind::Other, "Bytes too large!"));
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
