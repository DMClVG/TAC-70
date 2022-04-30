use std::{error::Error, pin::Pin, cell::RefCell};

use rgb::{Zeroable, RGB8, RGBA8};
use rlua::prelude::*;
use tac_core::TAC70;

const MAP_W: usize = 240;
const MAP_H: usize = 136;

#[derive(Clone, Copy)]
pub struct Sprite([u8; 8 * 4]);

#[derive(Clone, Copy)]
pub struct Font([u8; 8]);

pub struct TAC70Runtime {
    pub tac: TAC70,
    lua: Lua,
}

impl TAC70Runtime {
    pub fn new(tac: TAC70) -> Result<Self, Box<dyn Error>> {
        let lua = Lua::new();

        Ok(Self {
            tac,
            lua,
        })
    }

    pub fn step(&mut self) {
        static mut x: f32 = 0.0;

        let a = self.get_sprite(1);
        let b = self.get_sprite(2);
        let c = self.get_sprite(1+16);
        let d = self.get_sprite(2+16);
        unsafe {
            x += 0.2;
        }
        let mut screen = self.get_screen();
        screen.clear(0);
        screen.blit(unsafe { x as u8 }, unsafe { x as u8 }, &a, None);
        screen.blit(unsafe { x as u8 }.wrapping_add(8), unsafe { x as u8 }, &b, None);
        screen.blit(unsafe { x as u8 }, unsafe { x as u8 }.wrapping_add(8), &c, None);
        screen.blit(unsafe { x as u8 }.wrapping_add(8), unsafe { x as u8 }.wrapping_add(8), &d, None);
    }

    pub fn get_screen<'a>(&'a mut self) -> Screen<'a> {
        Screen { pixels: &mut self.tac.mem[0x0000..Screen::PX_BUFFER_SIZE], palette: Palette::default() }
    }

    pub fn get_sprite(&self, id: u8) -> Sprite {
        let off = id as usize * 8 * 4;
        Sprite(self.tac.mem[0x4000 + off..0x4000 + off + 8 * 4].try_into().unwrap())
    }
}



pub enum Palette {
    OneBPP([rgb::RGB8; 2]),
    TwoBPP([rgb::RGB8; 4]),
    FourBPP([rgb::RGB8; 16]),
}

impl Default for Palette {
    fn default() -> Self {
        Self::FourBPP([
            RGB8::new(0x1a, 0x1c, 0x2c),
            RGB8::new(0x5d, 0x27, 0x5d),
            RGB8::new(0xb1, 0x3e, 0x53),
            RGB8::new(0xef, 0x7d, 0x57),
            RGB8::new(0xff, 0xcd, 0x75),
            RGB8::new(0xa7, 0xf0, 0x70),
            RGB8::new(0x38, 0xb7, 0x64),
            RGB8::new(0x25, 0x71, 0x79),
            RGB8::new(0x29, 0x36, 0x6f),
            RGB8::new(0x3b, 0x5d, 0xc9),
            RGB8::new(0x41, 0xa6, 0xf6),
            RGB8::new(0x73, 0xef, 0xf7),
            RGB8::new(0xf4, 0xf4, 0xf4),
            RGB8::new(0x94, 0xb0, 0xc2),
            RGB8::new(0x56, 0x6c, 0x86),
            RGB8::new(0x33, 0x3c, 0x57),
        ])
        // Self::OneBPP([
        //     RGB8::new(0x1a, 0x1c, 0x2c),
        //     RGB8::new(0x33, 0xff, 0x57),
        // ])
    }
}

impl Palette {
    pub const fn bpp(&self) -> usize {
        match self {
            Self::OneBPP(_) => 1,
            Self::TwoBPP(_) => 2,
            Self::FourBPP(_) => 4,
        }
    }

    pub const fn mask(&self) -> u8 {
        u8::MAX >> (8_u8 - self.bpp() as u8)
    }

    pub fn get(&self, idx: u8) -> Option<RGB8> {
        let idx = idx as usize;
        match self {
            Self::OneBPP(colors) if idx < 2 => Some(colors[idx]),
            Self::TwoBPP(colors) if idx < 4 => Some(colors[idx]),
            Self::FourBPP(colors) if idx < 16 => Some(colors[idx]),
            _ => None,
        }
    }
}

pub struct Screen<'a> {
    pub pixels: &'a mut [u8],
    pub palette: Palette,
}

impl Screen<'_> {
    pub const WIDTH: usize = 240;
    pub const HEIGHT: usize = 136;
    const PX_BUFFER_SIZE: usize = (Self::WIDTH * Self::HEIGHT) / 2;

    pub fn clear(&mut self, pix: u8) {
        let mut byte = 0;
        for i in (0..8).step_by(self.palette.bpp()) {
            byte |= (pix & self.palette.mask()) << i;
        }
        self.pixels.copy_from_slice(&[byte; Screen::PX_BUFFER_SIZE]);
    }

    pub fn to_rgba(&self) -> [RGBA8; Screen::WIDTH * Screen::HEIGHT] {
        let mut out = [RGBA8::zeroed(); Self::WIDTH * Self::HEIGHT];
        let palette = &self.palette;
        let mask = palette.mask();
        for i in 0..Self::WIDTH * Self::HEIGHT {
            let bits = i * palette.bpp();
            let boff = bits % 8;

            let pix = (self.pixels[bits / 8] >> boff) & mask;
            out[i] = palette.get(pix).unwrap().alpha(255);
        }

        out
    }

    pub fn blit(&mut self, x: u8, y: u8, spr: &Sprite, alpha: Option<u8>) {
        let (x, y) = (x as usize, y as usize);
        let mask = self.palette.mask();
        for j in 0..8.min((Self::HEIGHT as isize - y as isize).max(0) as usize) {
            for i in 0..8.min((Self::WIDTH as isize - x as isize).max(0) as usize) {
                let spr_pix = {
                    let bits = (i * 4) + j * 4 * 8;
                    let boff = bits % 8;
                    (spr.0[bits / 8] >> boff) & mask
                } as u8;

                if alpha.is_some() && spr_pix == alpha.unwrap() { continue; }

                let bits = ((x + i) + (y + j) * Self::WIDTH) * self.palette.bpp();
                let boff = bits % 8;

                self.pixels[bits / 8] &= !(mask << boff);
                self.pixels[bits / 8] |= spr_pix << boff;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn test_screen_to_rgb() {
    //     let screen = Screen::default();
    //     let rgba = screen.to_rgba();
    // }
}