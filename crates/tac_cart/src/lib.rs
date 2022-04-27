use std::{u8, io::Cursor, error::Error, path::{PathBuf, Path}, fmt::Debug};

use binread::prelude::*;
use modular_bitfield::prelude::*;


#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 5]
enum ChunkType {
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
struct ChunkInfo {
    c_type: ChunkType,
    bank: B3,
}

#[derive(BinRead, Debug)]
struct Chunk {
    info: ChunkInfo,

    #[br(little)]
    size: u16,

    reserved: u8,

    #[br(count=size)]
    data: Vec<u8>,
}

pub struct Cartridge {
    chunks: Vec<Chunk>,
    title: String,
}

impl Cartridge {
    pub fn load(path: impl AsRef<Path>) -> Result<Cartridge, Box<dyn Error>> {
        let bytes = std::fs::read(path)?;
        Ok(Self::from_bytes(&bytes)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Cartridge, Box<dyn Error>> {
        let mut cursor = Cursor::new(bytes);
        let mut cart = Cartridge { chunks: vec![], title: "cart.tic".to_string() };

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
            writeln!(f, "BANK: {} SIZE: {} TYPE: {:?}", chunk.info.bank(), chunk.size, chunk.info.c_type())?;
        }
        Ok(())
    }
}