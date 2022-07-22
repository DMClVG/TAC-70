use std::{error::Error, fmt::Debug, io::Cursor, path::Path, u8};

use binread::prelude::*;
use modular_bitfield::prelude::*;
use tac_core::TAC70;

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

    _reserved: u8,

    #[br(count=size)]
    data: Vec<u8>,
}

pub struct Cartridge {
    pub title: String,
    chunks: Vec<Chunk>,
}

impl From<Cartridge> for TAC70 {
    fn from(cart: Cartridge) -> Self {
        let mut mem = Box::new([0u8; 0x18000]);
        let mut code = None;

        for chunk in cart.chunks {
            use ChunkType::*;
            match chunk.info.c_type() {
                Tiles => {
                    mem[0x4000..=0x5FFF][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Sprites => {
                    mem[0x6000..=0x7FFF][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Map => {
                    mem[0x8000..=0xFF7F][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Samples => {
                    mem[0x100E4..=0x11163][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Waveform => {
                    mem[0x0FFE4..=0x100E3][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Flags => {
                    mem[0x14404..=0x14603][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Music => {
                    mem[0x13E64..=0x13FFB][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Patterns => {
                    mem[0x11164..=0x13E63][..chunk.data.len()].copy_from_slice(&chunk.data);
                }
                Palette => {
                    mem[0x3FC0..=0x3FEF].copy_from_slice(&chunk.data[0..48]);
                    if chunk.data.len() == 96 {
                        // TODO: OVR PALETTE
                    }
                }
                Code => {
                    code = Some(std::str::from_utf8(&chunk.data).unwrap().to_string());
                }
                Screen => {} // dunno??
                Default => {}
                _ => unimplemented!(),
            }
        }
        TAC70::new(mem.as_ref(), code.unwrap())
    }
}

impl Cartridge {
    pub fn load(path: impl AsRef<Path>) -> Result<Cartridge, Box<dyn Error>> {
        let bytes = std::fs::read(path)?;
        Ok(Self::from_bytes(&bytes)?)
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
                chunk.info.c_type()
            )?;
        }
        Ok(())
    }
}
