use std::{error::Error, fmt::Debug, io::Cursor, path::Path, u8, slice::Iter};

use binread::prelude::*;
use binwrite::*;
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
#[derive(BinRead, BinWrite, Debug, Clone)]
#[br(map = Self::from_bytes)]
pub struct ChunkInfo {
    pub chunk_type: ChunkType,
    pub bank: B3, // specifies in which bank the chunk lives
}


#[derive(BinRead, BinWrite, Debug, Clone)]
pub struct Chunk {
    pub info: ChunkInfo,

    #[br(little)]
    pub size: u16,

    pub reserved: u8,

    #[br(count=size)]
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct Cartridge {
    pub title: String,
    pub chunks: Vec<Chunk>,
}

impl TryFrom<&[u8]> for Cartridge {
    type Error = Box<dyn Error>;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
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

impl TryInto<Vec<u8>> for Cartridge {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let mut out = Vec::new();
        for chunk in self.chunks {
            chunk.write(&mut out)?;
        }
        Ok(out)
    }
}

impl Cartridge {
    pub fn load(path: impl AsRef<Path>) -> Result<Cartridge, Box<dyn Error>> {
        let bytes = std::fs::read(path)?;
        Ok(Cartridge::try_from(bytes.as_slice())?)
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
