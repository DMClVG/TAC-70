use std::{error::Error, fmt::Debug, io::Cursor, path::Path, u8, slice::Iter};

use binread::prelude::*;
use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 5]
pub enum ChunkType {
    Dummy = 0,
    Tiles = 1,
    Sprites = 2,

    Map = 4,
    Code = 5,
    Flags = 6,
    Samples = 9,
    Waveform = 10,
    Palette = 12,
    Music = 14,
    Patterns = 15,
    Default = 17,
    Screen = 18,
    Binary = 19,

    #[deprecated]
    CoverDep = 3,
    #[deprecated]
    CodeZip = 16,
    #[deprecated]
    PatternsDep = 13,
}

#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct ChunkInfo {
    pub chunk_type: ChunkType,
    bank: B3,
}

#[derive(BinRead, Debug)]
pub struct Chunk {
    pub info: ChunkInfo,

    #[br(little)]
    pub size: u16,

    _reserved: u8,

    #[br(count=size)]
    pub data: Vec<u8>,
}

pub struct Cartridge {
    pub title: String,
    chunks: Vec<Chunk>,
}

impl Cartridge {
    pub fn load(path: impl AsRef<Path>) -> Result<Cartridge, Box<dyn Error>> {
        let bytes = std::fs::read(path)?;
        Ok(Self::from_bytes(&bytes)?)
    }

    pub fn chunks(&self) -> Iter<Chunk> {
        self.chunks.iter()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Cartridge, Box<dyn Error>> {
        let mut cursor = Cursor::new(bytes);
        let mut cart = Cartridge {
            chunks: vec![],
            title: "cart.tic".to_string(),
        };

        while (cursor.position() as usize) < bytes.len() {
            let chunk: Chunk = cursor.read_ne()?;
            cart.chunks.push(chunk);
        }
        Ok(cart)
    }
}

impl Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== CARTRIDGE {} ===", self.title)?;
        for chunk in &self.chunks {
            writeln!(
                f,
                "BANK: {} SIZE: {} TYPE: {:?}",
                chunk.info.bank(),
                chunk.size,
                chunk.info.chunk_type()
            )?;
        }
        Ok(())
    }
}
