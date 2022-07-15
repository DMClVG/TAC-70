use rgb::{RGB8, RGBA8};
use std::cell::Cell;

#[derive(Clone)]
pub struct Sprite([Cell<u8>; 8 * 4]);

#[derive(Clone)]
pub struct FontChar { 
    mem: [Cell<u8>; 8],
    pub width: u32,
    pub padx: i32,
}

pub trait PixBuf {
    const WIDTH: usize;
    const HEIGHT: usize;
    const BPP: usize;
    const MASK: u8 = pix_mask(Self::BPP);

    // fn buf(&self) -> &[Cell<u8>];
    // fn buf_mut(&mut self) -> &mut [Cell<u8>];

    fn set_buf(&mut self, i: usize, to: u8);
    fn get_buf(&self, i: usize) -> u8;

    fn get_pix(&self, x: i32, y: i32) -> u8 {
        assert!(x < Self::WIDTH as i32 && x >= 0);
        assert!(y < Self::HEIGHT as i32 && y >= 0);
        let (x, y) = (x as usize, y as usize);
        let i = (x + y * Self::WIDTH) * Self::BPP;
        (self.get_buf(i / 8) >> (i % 8)) & Self::MASK
    }

    fn set_pix(&mut self, x: i32, y: i32, pix: u8) {
        if x < 0 || x >= Self::WIDTH as i32 || y < 0 || y >= Self::HEIGHT as i32 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        let bit = (x + y * Self::WIDTH) * 4;
        let off = bit % 8;
        let mut byte = self.get_buf(bit / 8);
        byte &= !(Self::MASK << off);
        byte |= pix << off;
        self.set_buf(bit / 8, byte);
    }

    fn blit<O: PixBuf>(
        &mut self,
        x: i32,
        y: i32,
        spr: &O,
        alpha: Option<u8>,
        hflip: bool,
        vflip: bool,
        scale: u32,
    ) {
        if scale == 0 {
            return;
        }
        let scale = scale as usize;
        for i in 0..O::WIDTH {
            for j in 0..O::HEIGHT {
                let spr_pix = spr.get_pix(
                    if hflip { O::WIDTH - i - 1 } else { i } as i32,
                    if vflip { O::HEIGHT - j - 1 } else { j } as i32,
                );

                if Some(spr_pix) == alpha {
                    continue;
                }

                for l in 0..scale {
                    for m in 0..scale {
                        self.set_pix(
                            x + (i * scale + l) as i32,
                            y + (j * scale + m) as i32,
                            spr_pix,
                        );
                    }
                }
            }
        }
    }

    fn clear(&mut self, pix: u8) {
        for i in 0..Self::WIDTH {
            for j in 0..Self::HEIGHT {
                self.set_pix(i as i32, j as i32, pix);
            }
        }
    }

    fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, pix: u8) {
        if w == 0 || h == 0 { return }
        for i in 0..w as i32 {
            for j in 0..h as i32 {
                self.set_pix(x + i, y + j, pix);
            }
        }
    }

    fn rectb(&mut self, x: i32, y: i32, w: u32, h: u32, pix: u8) {
        if w == 0 || h == 0 { return }
        let (w, h) = (w as i32, h as i32);
        for i in 0..w {
            self.set_pix(x + i, y, pix);
            self.set_pix(x + i, y + h - 1, pix);
        }
        for j in 0..h-1 {
            self.set_pix(x, y + j, pix);
            self.set_pix(x + w - 1, y + j, pix);
        }
    }

    fn line(&mut self, ax: i32, ay: i32, bx: i32, by: i32, pix: u8) {}

    ///
    /// Returns vec of size WIDTH*HEIGHT
    fn to_rgba<'p>(&self, palette: &Palette<'p>) -> Vec<RGBA8> {
        let mut out = Vec::with_capacity(Self::WIDTH * Self::HEIGHT);
        for j in 0..Self::HEIGHT {
            for i in 0..Self::WIDTH {
                let pix = self.get_pix(i as i32, j as i32);
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
    pub char_cache: [(u32, i32); Self::CHAR_COUNT]
}

impl TAC70 {
    const CHAR_COUNT: usize = 127 * 2;

    pub fn new(mem: &[u8], code: String) -> Self {

        let mut mem = mem.to_owned();
        mem[0x14604..0x14604+8*Self::CHAR_COUNT].copy_from_slice(include_bytes!("font.bin")); // Load font to memory

        let mem = mem
            .into_iter()
            .map(|b| Cell::new(b))
            .collect::<Vec<Cell<u8>>>();


        let mut tac = Self {
            mem: mem.try_into().unwrap(),
            code,
            char_cache: [(0,0); Self::CHAR_COUNT]
        };
        tac.update_font_data();

        tac
    }

    pub fn palette(&self) -> Palette {
        Palette {
            mem: &self.mem[0x3FC0..0x3FF0],
        }
    }

    pub fn screen(&self) -> Screen {
        let palette = self.palette().to_owned();
        Screen {
            pixels: &self.mem[0..Screen::PX_BUFFER_SIZE],
            palette,
        }
    }

    pub fn map(&self) -> Map {
        Map {
            tiles: &self.mem[0x8000..0x8000 + Map::WIDTH * Map::HEIGHT],
        }
    }

    pub fn sprite(&self, id: u16) -> Option<Sprite> {
        if id >= 512 {
            return None;
        }
        let off = id as usize * 8 * 4;
        let mut spr = vec![Cell::new(0); 32];
        for i in 0..32 {
            spr[i] = self.mem[0x4000 + off + i].clone();
        }
        Some(Sprite(spr.try_into().unwrap()))
    }

    pub fn gamepads(&self) -> Gamepads {
        Gamepads {
            mem: &self.mem[0x0FF80..0x0FF80 + 4],
        }
    }

    pub fn char(&self, c: char, alt: bool) -> Option<FontChar> {
        let c = c as usize;
        if (0..128).contains(&c) {
            let ccode = c as usize + if alt { 127 } else { 0 };
            let off = ccode * 8;
            let mut font = vec![Cell::new(0); 8];
            for i in 0..8 {
                font[i] = self.mem[0x14604 + off + i].clone();
            }
            Some(FontChar { mem: font.try_into().unwrap(), width: self.char_cache[ccode].0, padx: self.char_cache[ccode].1 } )
        } else {
            None
        }
    }

    pub fn mouse(&self) -> Mouse {
        Mouse { mem: self.mem[0x0FF84..0x0FF84+4].try_into().unwrap() }
    }

    pub fn set_sprite(&mut self, id: u16, spr: Sprite) {
        assert!(id < 512);
        let off = id as usize * 8 * 4;
        self.mem[0x400 + off..0x4000 + off * 8 * 4].clone_from_slice(&spr.0);
    }

    pub fn update_font_data(&mut self) {
        for c in 0..127 {
            for alt in [false, true] {
                let ccode = c as usize + if !alt { 0 } else { 127 };
                if c == ' ' as u8 {
                    self.char_cache[ccode] = (if alt { 1 } else { 3 }, 0);
                    continue;
                }

                let fchar = self.char(c as u8 as char, alt).unwrap();

                let mut padr = 8;
                'col1: for i in (0..FontChar::WIDTH as i32).rev() {
                    for j in 0..FontChar::HEIGHT as i32 {
                        if fchar.get_pix(i, j) != 0 {
                            padr = 7-i;
                            break 'col1;
                        }
                    }
                }
                let mut padl = 0;
                'col2: for i in 0..FontChar::WIDTH as i32 {
                    for j in 0..FontChar::HEIGHT as i32 {
                        if fchar.get_pix(i, j) != 0 {
                            padl = i;
                            break 'col2;
                        }
                    }
                }
                let width = 8 - padr - padl;
                self.char_cache[ccode] = (width.max(0) as u32, padl);
            } 
        }
    }
}

pub struct Gamepads<'a> {
    mem: &'a [Cell<u8>],
}

pub struct Gamepad<'a> {
    byte: &'a Cell<u8>,
}

pub struct Mouse<'a> {
    mem: &'a [Cell<u8>; 4]
}

impl Mouse<'_> {
    pub fn set(&self, mx: u8, my: u8, ml: bool, mm: bool, mr: bool, scrollx: i8, scrolly: i8) {
        assert!(scrollx > -33 && scrollx < 32);
        assert!(scrolly > -33 && scrolly < 32);

        let mut dword: u32 = (mx as u32) | (my as u32) << 8;
        if ml {
            dword |= 0b1 << 16;
        }
        if mm {
            dword |= 0b10 << 16;
        }
        if mr {
            dword |= 0b100 << 16;
        }
        dword |= (scrollx as u8 as u32 & 0b111111) << 19;
        dword |= (scrolly as u8 as u32 & 0b111111) << 25;

        let m = &self.mem;
        m[0].set((dword & 0xFF) as u8);
        m[1].set(((dword >> 8) & 0xFF) as u8);
        m[2].set(((dword >> 16) & 0xFF) as u8);
        m[3].set(((dword >> 24) & 0xFF) as u8);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.mem[0].get() as i32, self.mem[1].get() as i32)
    }

    pub fn buttons(&self) -> (bool, bool, bool) {
        let buttons = self.mem[2].get();
        (
            buttons & 0b1 != 0,
            buttons & 0b10 != 0,
            buttons & 0b100 != 0
        )
    }

    pub fn scrolly(&self) -> i32 {
        (((((self.mem[3].get() >> 1) & 0b111111) << 2) as i8) >> 2) as i32
    }

    pub fn scrollx(&self) -> i32 {
        let i6 = ((self.mem[2].get() >> 3) & 0b11111) | (self.mem[3].get() & 0b1) << 5;
        (((i6 << 2) as i8) >> 2) as i32
    }
}

impl<'a> Gamepads<'a> {
    pub fn player(&self, id: u8) -> Gamepad {
        Gamepad {
            byte: &self.mem[id as usize],
        }
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
    mem: &'a [Cell<u8>],
}

impl Palette<'_> {
    pub const fn bpp(&self) -> usize {
        4
    }

    pub fn get(&self, idx: u8) -> Option<RGB8> {
        Some(RGB8::new(
            self.mem[idx as usize * 3 + 0].get(),
            self.mem[idx as usize * 3 + 1].get(),
            self.mem[idx as usize * 3 + 2].get(),
        ))
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

    fn set_buf(&mut self, i: usize, to: u8) {
        self.pixels[i].set(to)
    }
    fn get_buf(&self, i: usize) -> u8 {
        self.pixels[i].get()
    }
}

impl PixBuf for Sprite {
    const WIDTH: usize = 8;
    const HEIGHT: usize = 8;
    const BPP: usize = 4;

    fn set_buf(&mut self, i: usize, to: u8) {
        self.0[i].set(to)
    }
    fn get_buf(&self, i: usize) -> u8 {
        self.0[i].get()
    }
}

impl PixBuf for FontChar {
    const WIDTH: usize = 8;
    const HEIGHT: usize = 8;
    const BPP: usize = 1;

    fn set_buf(&mut self, i: usize, to: u8) {
        self.mem[i].set(to)
    }
    fn get_buf(&self, i: usize) -> u8 {
        self.mem[i].get()
    }
}

pub struct Colorized<T: PixBuf>(pub u8, pub T);

impl<T: PixBuf> PixBuf for Colorized<T> {
    const WIDTH: usize = T::WIDTH;
    const HEIGHT: usize = T::HEIGHT;

    const BPP: usize = T::BPP;

    const MASK: u8 = pix_mask(Self::BPP);

    fn get_pix(&self, x: i32, y: i32) -> u8 {
        match self.1.get_pix(x, y) {
            0 => 0,
            _ => self.0
        } 
    }

    fn set_pix(&mut self, _x: i32, _y: i32, _pix: u8) {
        unimplemented!()
    }

    fn set_buf(&mut self, _i: usize, _to: u8) {
        unimplemented!()
    }

    fn get_buf(&self, _i: usize) -> u8 {
        unimplemented!()
    }
}

impl Screen<'_> {
    const PX_BUFFER_SIZE: usize = (Self::WIDTH * Self::HEIGHT) / 2;
}
