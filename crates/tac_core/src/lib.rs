use std::cell::Cell;
use rgb::{RGBA8, RGB8};


#[derive(Clone)]
pub struct Sprite([Cell<u8>; 8 * 4]);

#[derive(Clone)]
pub struct Font([Cell<u8>; 8]);

pub trait PixBuf {
    const WIDTH: usize;
    const HEIGHT: usize;
    const BPP: usize;
    const MASK: u8 = pix_mask(Self::BPP);

    // fn buf(&self) -> &[Cell<u8>];
    // fn buf_mut(&mut self) -> &mut [Cell<u8>];

    fn set_buf(&mut self, i: usize, to: u8);
    fn get_buf(&self, i: usize) -> u8;

    fn get_pix(&self, x: usize, y: usize) -> u8 {
        assert!(x < Self::WIDTH);
        assert!(y < Self::HEIGHT);
        let i = (x + y * Self::WIDTH) * Self::BPP;
        (self.get_buf(i / 8) >> (i % 8)) & Self::MASK
    }

    fn set_pix(&mut self, x: usize, y: usize, pix: u8) {
        assert!(x < Self::WIDTH);
        assert!(y < Self::HEIGHT);

        const MASK: u8 = pix_mask(4);
        let bit = (x + y * Self::WIDTH) * 4;
        let off = bit % 8;
        let mut byte = self.get_buf(bit / 8);
        byte &= !(MASK << off);
        byte |= pix << off;
        self.set_buf(bit / 8, byte);
    }

    fn blit<O: PixBuf>(&mut self, x: i32, y: i32, spr: &O, alpha: Option<u8>) {
        for i in x.min(0).abs() as usize..O::WIDTH.min((Self::WIDTH as isize - x.max(0) as isize).max(0) as usize) {
            for j in y.min(0).abs() as usize..O::HEIGHT.min((Self::HEIGHT as isize - y.max(0) as isize).max(0)  as usize) {
                let spr_pix = spr.get_pix(i,j);

                if Some(spr_pix) == alpha { continue; }

                self.set_pix((x+i as i32).max(0) as usize, (y+j as i32).max(0) as usize, spr_pix);
            }
        }
    }

    fn clear(&mut self, pix: u8) {
        for i in 0..Self::WIDTH {
            for j in 0..Self::HEIGHT {
                self.set_pix(i, j, pix);
            }
        }
    }

    ///
    /// Returns vec of size WIDTH*HEIGHT
    fn to_rgba<'p>(&self, palette: &Palette<'p>) -> Vec<RGBA8> {
        let mut out = Vec::with_capacity(Self::WIDTH * Self::HEIGHT);
        for j in 0..Self::HEIGHT {
            for i in 0..Self::WIDTH {
                let pix = self.get_pix(i, j);
                out.push(palette.get(pix).unwrap().alpha(255));
            }
        }

        out
    }
}

const fn pix_mask(bpp: usize) -> u8 {
    u8::MAX >> (8_u8 - bpp as u8)
}

#[derive(Clone)]
pub struct TAC70 {
    pub mem: [Cell<u8>; 0x18000],
    pub code: String,
}

impl TAC70 {
    pub fn new(mem: &[u8], code: String) -> Self {
        let mem = mem.into_iter().cloned().map(|b| Cell::new(b)).collect::<Vec<Cell<u8>>>();
        
        Self {
            mem: mem.try_into().unwrap(),
            code,
        }
    }

    pub fn palette(&self) -> Palette {
        Palette { mem: &self.mem[0x3FC0..0x3FF0] }
    }

    pub fn screen(&self) -> Screen {
        let palette = self.palette().to_owned();
        Screen { pixels: &self.mem[0.. Screen::PX_BUFFER_SIZE], palette }
    }

    pub fn map(&self) -> Map {
        Map { tiles: &self.mem[0x8000.. 0x8000+Map::WIDTH*Map::HEIGHT] }
    }

    pub fn sprite(&self, id: u16) -> Option<Sprite> {
        if id >= 512 { return None; } 
        let off = id as usize * 8 * 4;
        let mut spr = vec![Cell::new(0); 32];
        for i in 0..32 {
            spr[i] = self.mem[0x4000 + off + i].clone(); 
        }
        Some(Sprite(spr.try_into().unwrap()))
    }

    pub fn gamepads(&self) -> Gamepads {
        Gamepads { mem: &self.mem[0x0FF80..0x0FF80+4] }
    }

    pub fn set_sprite(&mut self, id: u16, spr: Sprite) {
        assert!(id < 512);
        let off = id as usize * 8 * 4;
        self.mem[0x400 + off..0x4000 + off * 8 * 4].clone_from_slice(&spr.0);
    }
}

pub struct Gamepads<'a> {
    mem: &'a [Cell<u8>]
}

pub struct Gamepad<'a> {
    byte: &'a Cell<u8>
}

impl<'a> Gamepads<'a> {
    pub fn player(&self, id: u8) -> Gamepad {
        Gamepad { byte:  &self.mem[id as usize] }
    }
}

impl<'a> Gamepad<'a> {
    pub fn btn(&self, btn: u8) -> bool {
        self.byte.get() & (1 << btn) != 0
    }

    pub fn set_btn(&self, btn: u8, pressed: bool) {
        if pressed {
            self.byte.set(self.byte.get() | (1 << btn))
        } else {
            self.byte.set(self.byte.get() & !(1 << btn))
        }
    }
}

// pub enum Palette {
//     OneBPP([rgb::RGB8; 2]),
//     TwoBPP([rgb::RGB8; 4]),
//     FourBPP([rgb::RGB8; 16]),
// }

#[derive(Clone)]
pub struct Palette<'a> {
    mem: &'a [Cell<u8>]
}

impl Palette<'_> {
    pub const fn bpp(&self) -> usize {
        4
    }

    pub fn get(&self, idx: u8) -> Option<RGB8> {
        Some(RGB8::new(self.mem[idx as usize*3+0].get(),self.mem[idx as usize*3+1].get(),self.mem[idx as usize*3+2].get()))
    }
}

pub struct Map<'a> {
    pub tiles: &'a [Cell<u8>],
}

impl Map<'_> {
    pub const WIDTH: usize = 240;
    pub const HEIGHT: usize = 136;

    pub fn get(&self, x: i32, y: i32) -> Option<u8> {
        if (0..Map::WIDTH).contains(&(x as usize)) && (0..Map::HEIGHT).contains(&(y as usize)) {
            Some(self.tiles[x as usize + y as usize * Map::WIDTH].get())
        } else {
            None
        }
    }

    pub fn set(&mut self, x: i32, y: i32, id: u8) {
        if (0..Map::WIDTH).contains(&(x as usize)) && (0..Map::HEIGHT).contains(&(y as usize)) {
            self.tiles[x as usize + y as usize * Map::WIDTH].set(id);
        }
    }
}

pub struct Screen<'a> {
    pub pixels: &'a [Cell<u8>],
    pub palette: Palette<'a>,
}

impl PixBuf for Screen<'_> {
    const WIDTH: usize = 240;
    const HEIGHT: usize = 136;
    const BPP: usize = 4;

    fn set_buf(&mut self, i: usize, to: u8) { self.pixels[i].set(to) }
    fn get_buf(&self, i: usize) -> u8 { self.pixels[i].get() }
}

impl PixBuf for Sprite {
    const WIDTH: usize = 8;
    const HEIGHT: usize = 8;
    const BPP: usize = 4;
    
    fn set_buf(&mut self, i: usize, to: u8) { self.0[i].set(to) }
    fn get_buf(&self, i: usize) -> u8 { self.0[i].get() }
}

impl Screen<'_> {
    const PX_BUFFER_SIZE: usize = (Self::WIDTH * Self::HEIGHT) / 2;
}
