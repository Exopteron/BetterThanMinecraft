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
// TODO Make CountingReader and Cursor into one AtomicCursor
use super::classic::{Packet::{LevelInitialize, LevelDataChunk, LevelFinalize}};
use super::{game::BlockID, Block, ClassicPacketWriter};
use std::pin::Pin;
use flate2::Compression;
use flate2::GzBuilder;
use flate2::write::GzEncoder;
use std::io::{Read};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use bytes::{BytesMut};
use tokio::task;
use tokio::sync::RwLock;
use std::io::Write;
#[derive(Clone)]
pub struct ChunkedWorld {
  data: Arc<Vec<Arc<RwLock<Vec<u8>>>>>, // XZY
  pub width: usize,
  pub height: usize,
  pub length: usize,
  pub path: Option<String>,
  pub spawnpos: Option<super::game::PlayerPosition>,
}
pub struct ExoReader {
  internal: Vec<u8>,
}
impl std::io::Write for ExoReader {
  fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
    let mut bytes = bytes.to_vec();
    //bytes.reverse();
    //self.internal.reverse();
    let x = std::io::Write::write(&mut self.internal, &bytes)?;
    //self.internal.reverse();
    Ok(x)
    //self.internal.write(bytes)
  }
  fn flush(&mut self) -> std::io::Result<()> {
    self.internal.reverse();
    std::io::Write::flush(&mut self.internal)?;
    self.internal.reverse();
    Ok(())
    //self.internal.flush()
  }
}
impl std::io::Read for ExoReader {
  fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
    self.internal.reverse();
    let mut x = &self.internal[..];
    let num = x.read(bytes)?;
    self.internal = x.to_vec();
    self.internal.reverse();
    Ok(num)
  }
}
impl ExoReader {
  pub fn new() -> Self {
    Self { internal: Vec::new() }
  }
}
impl ChunkedWorld {
  pub fn from_file(file_path: &str) -> Option<Self> {
    use nbt::decode::read_compound_tag;
    use flate2::read::GzDecoder;
    let cursor = std::fs::File::open(file_path).ok()?;
    let mut cursor = GzDecoder::new(cursor);
    let root_tag = read_compound_tag(&mut cursor).ok()?;
    let world = root_tag.get_i8_vec("BlockArray").ok()?;
    let width = root_tag.get_i16("X").ok()? as usize;
    let height = root_tag.get_i16("Y").ok()? as usize;
    let length = root_tag.get_i16("Z").ok()? as usize;
    let spawn = root_tag.get_compound_tag("Spawn").ok()?;
    let spawn_x = spawn.get_i16("X").ok()?;
    let spawn_y = spawn.get_i16("Y").ok()?;
    let spawn_z = spawn.get_i16("Z").ok()?;
    let mut newworld = vec![];
    for i in 0..world.len() {
      newworld.push(world[i] as u8);
    }
    let size = width * height * length;
    let mut data = vec![0; 4];
    data[0] = (size >> 24) as u8;
    data[1] = (size >> 16) as u8;
    data[2] = (size >> 8) as u8;
    data[3] = size as u8;
    data.append(&mut newworld);
    let data = data.chunks(65536).collect::<Vec<&[u8]>>();
    let mut chunks = vec![];
    for chunk in data {
      chunks.push(Arc::new(RwLock::new(chunk.to_vec())));
    }
    //let data = data.into_boxed_slice();
    Some(Self { data: Arc::new(chunks), width, height, length, path: Some(file_path.to_string()), spawnpos: Some(super::game::PlayerPosition::from_pos(spawn_x as u16, spawn_y as u16, spawn_z as u16)) })
    //Some(())
  }
  pub fn pos_to_index(&self, x: usize, y: usize, z: usize) -> Option<(usize, usize)> {
    let pos = (z + y * self.length) * self.width + x;
    let mut chunk = 0;
    let mut position = 0;
    for _ in 0..pos {
      position += 1;
      if position >= 65536 {
        position = 0;
        chunk += 1;
      }
    }
    Some((chunk, position))
    //(z.checked_add(y.checked_mul(self.length)?)?).checked_mul(self.width.checked_add(x)?)
  }

  pub async fn set_block(&mut self, mut block: Block) -> Option<()> {
    block.position.x += 4;
    let (chunk, pos) = self.pos_to_index(block.position.x as usize, block.position.y as usize, block.position.z as usize)?;
    let chunk = self.data.get(chunk);
    if let Some(chunk) = chunk {
      let mut w = chunk.write().await;
      match w.get(pos) {
        None => {
          return None;
        }
        _ => {}
      }
      w[pos] = block.id;
      return Some(());
    } else {
      return None;
    }
  }
  pub async fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockID> {
    let x = x + 4;
    let (chunk, pos) = self.pos_to_index(x, y, z)?;
    let x = self.data.get(chunk);
    match x {
      Some(chunk) => {
        let chunk = chunk.read().await;
        match chunk.get(pos) {
          None => {
            return None;
          }
          p => {
            return Some(*p?);
          }
        }
      }
      None => {
        return None;
      }
    }
  }
  pub fn get_world_spawnpos(&self) -> &Option<super::game::PlayerPosition> {
    &self.spawnpos
  }
  pub fn set_world_spawnpos(&mut self, pos: super::game::PlayerPosition) {
    self.spawnpos = Some(pos);
  }

  pub async fn to_packets(&mut self, writer: &mut Pin<Box<impl tokio::io::AsyncWriteExt>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut data = vec![];
    for chunk in &*self.data {
      let r_data = chunk.read().await;
      data.append(&mut (*r_data.clone()).to_vec());
    }
    //let mut encoder = GzEncoder::new(self.data(), Compression::fast());
    let len = self.width * self.height * self.length;
    //let mut reader = &mut self.data;
    let mut data = std::io::Cursor::new(vec![1]);
    let (mut reader, counter) = CountingReader::new(&mut data);
    let string = vec!['g' as u8; 12];
    task::yield_now().await;
    let mut chunks = self.data.iter();
    let mut encoder = GzEncoder::new(ExoReader::new(), Compression::fast());
    let mut count = 0;
    let chunk = chunks.next().ok_or(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "g")))?.read().await;
    encoder.write_all(&chunk)?;
    count += chunk.len();
    drop(chunk);
    //let mut encoder = GzBuilder::new().comment(string).read(&mut reader, Compression::fast());
    task::yield_now().await;
    let serialized = ClassicPacketWriter::serialize(LevelInitialize).unwrap();
    writer.write_all(&serialized).await?;
    loop {
      task::yield_now().await;
      let mut x = [0; 1024];
      task::yield_now().await;
      let res = encoder.read_exact(&mut x);
      //let mut x = x.to_vec();
      //x.reverse();
      //let mut x: [u8; 1024] = x[..];
      task::yield_now().await;
      if res.is_err() {
        if let Some(chunk) = chunks.next() {
          let chunk = chunk.read().await;
          encoder.write_all(&chunk)?;
          count += chunk.len();
          drop(chunk);
          continue;
        }
        let mut chunk_data = vec![];
        task::yield_now().await;
        encoder.read_to_end(&mut chunk_data)?;
        //chunk_data.reverse();
        task::yield_now().await;
        //println!("Reader: {:?}", counter.load(Ordering::SeqCst));
        if chunk_data.len() == 0 {
          let serialized = ClassicPacketWriter::serialize(LevelFinalize { width: self.width, height: self.height, length: self.length}).unwrap();
          writer.write_all(&serialized).await?;
          return Ok(());
        }
        let chunk_data = chunk_data.into_boxed_slice();
        let len = chunk_data.len();
        let serialized = ClassicPacketWriter::serialize(LevelDataChunk { chunk_length: len as i16, chunk_data, percent_complete: 255}).unwrap();
        writer.write_all(&serialized).await?;
        task::yield_now().await;
      } else {
        //log::info!("Else!");
        println!("Reader: {:?}", count);
        //println!("Sending");
        //let i = i << 10;
        let chunk_data = Box::new(x);
        let serialized = ClassicPacketWriter::serialize(LevelDataChunk { chunk_length: 1024, chunk_data,
          percent_complete: (count * 255 / len) as u8,
        }).unwrap();
        writer.write_all(&serialized).await?;
        task::yield_now().await;
      }
    }
  }
  pub async fn save(&self) -> Option<()> {
    if self.path.is_some() {
      use nbt::decode::{read_compound_tag};
      use nbt::encode::write_gzip_compound_tag;
      use flate2::read::GzDecoder;
      let cursor = std::fs::File::open(self.path.as_ref().unwrap()).ok()?;
      let mut cursor = GzDecoder::new(cursor);
      let mut root_tag = read_compound_tag(&mut cursor).ok()?;
      let spawn: &mut nbt::CompoundTag = root_tag.get_mut("Spawn").ok()?;
      if self.spawnpos.is_some() {
        let sppos = self.spawnpos.unwrap();
        let spawn_x: &mut i16 = spawn.get_mut("X").ok()?;
        *spawn_x = (sppos.x / 32) as i16;
        drop(spawn_x);
        let spawn_y: &mut i16 = spawn.get_mut("Y").ok()?;
        *spawn_y = (sppos.y / 32) as i16;
        drop(spawn_y);
        let spawn_z: &mut i16 = spawn.get_mut("Z").ok()?;
        *spawn_z = (sppos.z / 32) as i16;
        drop(spawn_z);
      }
      drop(spawn);
      let mut data = vec![];
      for chunk in &*self.data {
        let r_data = chunk.read().await;
        data.append(&mut (*r_data.clone()).to_vec());
      }
      let world: &mut Vec<i8> = root_tag.get_mut("BlockArray").ok()?;
      for i in 0..data.len() - 4 {
        if world.get_mut(i).is_none() {
          world.push(data[i + 4] as i8);
        } else {
          world[i] = data[i + 4] as i8;
        }
      }
      let mut byte_tag = vec![];
      write_gzip_compound_tag(&mut byte_tag, &root_tag).ok()?;
      std::fs::write(self.path.as_ref().unwrap(), byte_tag).ok()?;
      return Some(());
    }
    None
  }
}


#[derive(Clone)]
pub struct World {
  data: BytesMut, // XZY
  pub width: usize,
  pub height: usize,
  pub length: usize,
  pub path: Option<String>,
  pub spawnpos: Option<super::game::PlayerPosition>,
}

use std::io::Result as IoResult;
pub struct CountingReader<'a> {
  inner: &'a mut (dyn Read + Send),
  pub count: Arc<AtomicUsize>,
}

impl<'a> CountingReader<'a> {
  pub fn new(read: &'a mut (dyn Read + Send)) -> (Self, Arc<AtomicUsize>) {
    let count = Arc::new(AtomicUsize::new(0));
    (Self { inner: read, count: count.clone() }, count.clone())
  }

}

impl<'a> Read for CountingReader<'a> {
  fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
    let count = self.inner.read(buf)?;
    self.count.fetch_add(1, Ordering::SeqCst);
    Ok(count)
  }
}

impl World {
  pub fn new(generator: impl WorldGenerator, width: usize, height: usize, length: usize) -> Self {
    let size = width * height * length;
    let mut data = vec![0; size + 4];
    // Big-endian length of blocks array number
    // TODO use bytes to simplify into .put_be_i32() or something
    data[0] = (size >> 24) as u8;
    data[1] = (size >> 16) as u8;
    data[2] = (size >> 8) as u8;
    data[3] = size as u8;
    generator.generate(&mut data[4..], width, height, length);
    Self { data: BytesMut::from(&data[..]), width, height, length, path: None, spawnpos: None }
  }
  pub fn from_file(file_path: &str) -> Option<World> {
    use nbt::decode::read_compound_tag;
    use flate2::read::GzDecoder;
    let cursor = std::fs::File::open(file_path).ok()?;
    let mut cursor = GzDecoder::new(cursor);
    let root_tag = read_compound_tag(&mut cursor).ok()?;
    let world = root_tag.get_i8_vec("BlockArray").ok()?;
    let width = root_tag.get_i16("X").ok()? as usize;
    let height = root_tag.get_i16("Y").ok()? as usize;
    let length = root_tag.get_i16("Z").ok()? as usize;
    let spawn = root_tag.get_compound_tag("Spawn").ok()?;
    let spawn_x = spawn.get_i16("X").ok()?;
    let spawn_y = spawn.get_i16("Y").ok()?;
    let spawn_z = spawn.get_i16("Z").ok()?;
    let mut newworld = vec![];
    for i in 0..world.len() {
      newworld.push(world[i] as u8);
    }
    let size = width * height * length;
    let mut data = vec![0; 4];
    data[0] = (size >> 24) as u8;
    data[1] = (size >> 16) as u8;
    data[2] = (size >> 8) as u8;
    data[3] = size as u8;
    data.append(&mut newworld);
    //let data = data.into_boxed_slice();
    Some(Self { data: BytesMut::from(&data[..]), width, height, length, path: Some(file_path.to_string()), spawnpos: Some(super::game::PlayerPosition::from_pos(spawn_x as u16, spawn_y as u16, spawn_z as u16)) })
  }
  pub fn save(&self) -> Option<()> {
    if self.path.is_some() {
      use nbt::decode::{read_compound_tag};
      use nbt::encode::write_gzip_compound_tag;
      use flate2::read::GzDecoder;
      let cursor = std::fs::File::open(self.path.as_ref().unwrap()).ok()?;
      let mut cursor = GzDecoder::new(cursor);
      let mut root_tag = read_compound_tag(&mut cursor).ok()?;
      let spawn: &mut nbt::CompoundTag = root_tag.get_mut("Spawn").ok()?;
      if self.spawnpos.is_some() {
        let sppos = self.spawnpos.unwrap();
        let spawn_x: &mut i16 = spawn.get_mut("X").ok()?;
        *spawn_x = (sppos.x / 32) as i16;
        drop(spawn_x);
        let spawn_y: &mut i16 = spawn.get_mut("Y").ok()?;
        *spawn_y = (sppos.y / 32) as i16;
        drop(spawn_y);
        let spawn_z: &mut i16 = spawn.get_mut("Z").ok()?;
        *spawn_z = (sppos.z / 32) as i16;
        drop(spawn_z);
      }
      drop(spawn);
      let world: &mut Vec<i8> = root_tag.get_mut("BlockArray").ok()?;
      for i in 0..self.data.len() - 4 {
        if world.get_mut(i).is_none() {
          world.push(self.data[i + 4] as i8);
        } else {
          world[i] = self.data[i + 4] as i8;
        }
      }
      let mut byte_tag = vec![];
      write_gzip_compound_tag(&mut byte_tag, &root_tag).ok()?;
      std::fs::write(self.path.as_ref().unwrap(), byte_tag).ok()?;
      return Some(());
    }
    None
  }
  pub fn get_world_spawnpos(&self) -> &Option<super::game::PlayerPosition> {
    &self.spawnpos
  }
  pub fn set_world_spawnpos(&mut self, pos: super::game::PlayerPosition) {
    self.spawnpos = Some(pos);
  }
  pub fn pos_to_index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
    Some((z + y * self.length) * self.width + x)
    //(z.checked_add(y.checked_mul(self.length)?)?).checked_mul(self.width.checked_add(x)?)
  }

  // TODO position struct type stuff
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockID> {
    let x = x + 4;
    let x = self.data.get(self.pos_to_index(x, y, z)?);
    match x {
      Some(x) => {
        return Some(*x);
      }
      None => {
        return None;
      }
    }
  }

  pub fn set_block(&mut self, mut block: Block) -> Option<()> {
    block.position.x += 4;
    let pos = self.pos_to_index(block.position.x as usize, block.position.y as usize, block.position.z as usize)?;
    let p2 = pos.clone();
    drop(pos);
    let test = self.data.get(p2);
    if test.is_some() {
      self.data[p2] = block.id;
      return Some(());
    } else {
      return None;
    }
  }

  pub fn data(&self) -> &[BlockID] {
    &self.data
  }
  pub fn new_data(&self) -> BytesMut {
    self.data.clone()
  }

  pub fn data_mut(&mut self) -> &mut [BlockID] {
    &mut self.data
  }
      // QUICK AND DIRTY
      pub async fn to_packets(&mut self, writer: &mut Pin<Box<impl tokio::io::AsyncWriteExt>>) -> Result<(), Box<dyn std::error::Error>> {
        //let mut encoder = GzEncoder::new(self.data(), Compression::fast());
        let len = self.data().len();
        //let mut reader = &mut self.data;
        let mut data = self.data();
        let (mut reader, counter) = CountingReader::new(&mut data);
        let string = vec!['g' as u8; 12];
        task::yield_now().await;
        let mut encoder = GzBuilder::new().comment(string).read(&mut reader, Compression::fast());
        task::yield_now().await;
        let serialized = ClassicPacketWriter::serialize(LevelInitialize).unwrap();
        writer.write_all(&serialized).await?;
        loop {
          task::yield_now().await;
          let mut x = [0; 1024];
          task::yield_now().await;
          let res = encoder.read_exact(&mut x);
          task::yield_now().await;
          if res.is_err() {
            let mut chunk_data = vec![];
            task::yield_now().await;
            encoder.read_to_end(&mut chunk_data)?;
            task::yield_now().await;
            //println!("Reader: {:?}", counter.load(Ordering::SeqCst));
            if chunk_data.len() == 0 {
              let serialized = ClassicPacketWriter::serialize(LevelFinalize { width: self.width, height: self.height, length: self.length}).unwrap();
              writer.write_all(&serialized).await?;
              return Ok(());
            }
            let chunk_data = chunk_data.into_boxed_slice();
            let len = chunk_data.len();
            let serialized = ClassicPacketWriter::serialize(LevelDataChunk { chunk_length: len as i16, chunk_data, percent_complete: 255}).unwrap();
            writer.write_all(&serialized).await?;
            task::yield_now().await;
          } else {
            let count = counter.load(Ordering::SeqCst);
            //println!("Reader: {:?}", count);
            //println!("Sending");
            //let i = i << 10;
            let chunk_data = Box::new(x);
            let serialized = ClassicPacketWriter::serialize(LevelDataChunk { chunk_length: 1024, chunk_data,
              percent_complete: (count * 255 / len) as u8,
            }).unwrap();
            writer.write_all(&serialized).await?;
            task::yield_now().await;
          }
        }
      }
}
pub trait WorldGenerator {
  fn generate(&self, data: &mut [BlockID], width: usize, height: usize, length: usize);
}

pub struct FlatWorldGenerator {
  height: usize,
  below: BlockID,
  surface: BlockID,
  above: BlockID,
}

impl FlatWorldGenerator {
  pub fn new(height: usize, below: BlockID, surface: BlockID, above: BlockID) -> Self {
    Self { height, below, surface, above }
  }
}
impl WorldGenerator for FlatWorldGenerator {
  fn generate(&self, data: &mut [BlockID], width: usize, height: usize, length: usize) {
    let area = width * length;
    for y in 0..height {
      let yi = area * y;
      if y < self.height - 1 {
        for i in 0..area { data[yi + i] = self.below; }
      } else if y < self.height {
        for i in 0..area { data[yi + i] = self.surface; }
      } else {
        for i in 0..area { data[yi + i] = self.above; }
      }
    }
  }
}