use std::{error::Error, time::Instant};

use mlua::prelude::*;
use tac_core::{Colorized, PixBuf, TAC70};

pub struct TAC70Runtime {
    pub lua_ctx: Lua,
}

impl TAC70Runtime {
    pub fn new(tac: TAC70) -> Result<Self, Box<dyn Error>> {
        let lua = Lua::new_with(
            LuaStdLib::NONE
                | LuaStdLib::TABLE
                | LuaStdLib::STRING
                | LuaStdLib::MATH
                | LuaStdLib::UTF8,
            LuaOptions::new(),
        )?;

        let globals = lua.globals();

        let trace = lua.create_function(|_, msg: String| {
            println!("TRACE: {}", msg);
            Ok(())
        })?;

        let mget = lua.create_function(|ctx, (x, y): (i32, i32)| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.map().get(x, y).unwrap_or(0))
        })?;

        let mset = lua.create_function(|ctx, (x, y, id): (i32, i32, u8)| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.map().set(x, y, id))
        })?;

        let cls = lua.create_function(|ctx, pix: u8| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.screen().clear(pix))
        })?;

        let rect = lua.create_function(|ctx, (x, y, w, h, pix): (i32, i32, u32, u32, u8)| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            tac.screen().rect(x, y, w, h, pix);
            Ok(())
        })?;

        let rectb = lua.create_function(|ctx, (x, y, w, h, pix): (i32, i32, u32, u32, u8)| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            tac.screen().rectb(x, y, w, h, pix);
            Ok(())
        })?;

        let print = lua.create_function(
            |ctx,
             (s, x, y, pix, fixed, scale, smallfont): (
                String,
                i32,
                i32,
                Option<u8>,
                Option<bool>,
                Option<u32>,
                Option<bool>,
            )| {
                let tac = ctx.app_data_ref::<TAC70>().unwrap();
                let (pix, fixed, scale, smallfont) = (
                    pix.unwrap_or(15),
                    fixed.unwrap_or(false),
                    scale.unwrap_or(1),
                    smallfont.unwrap_or(false),
                );

                
                let mut cursor = 0;
                let fixedw = if smallfont { 3 } else { 5 }; 
                for c in s.chars() {
                    let fchar = tac.char(c, smallfont).unwrap();
                    let advance = if !fixed { fchar.width as i32 } else { fixedw } + 1;
                    tac.screen().blit(
                        x + (cursor - if !fixed { fchar.padx } else { 0 }) * scale as i32,
                        y,
                        &Colorized(pix, fchar),
                        Some(0),
                        false,
                        false,
                        scale,
                    );
                    cursor += advance;
                }
                Ok(cursor) // return width
            },
        )?;

        let spr = lua.create_function(
            |ctx,
             (id, x, y, alpha, scale, flip, rot, w, h): (
                u16,
                i32,
                i32,
                Option<u8>,
                Option<u32>,
                LuaValue,
                Option<u32>,
                Option<u32>,
                Option<u32>,
            )| {
                let tac = ctx.app_data_ref::<TAC70>().unwrap();
                let (scale, _rot, w, h) = (
                    scale.unwrap_or(1),
                    rot.unwrap_or(0),
                    w.unwrap_or(1),
                    h.unwrap_or(1),
                );
                let flip = match flip {
                    LuaValue::Boolean(b) if b => 1,
                    LuaValue::Integer(n) => n,
                    _ => 0,
                };
                let (hflip, vflip) = (flip & 0b1 != 0, flip & 0b10 != 0);
                for i in 0..w {
                    for j in 0..h {
                        let px = if hflip { (w - i - 1) * 8 } else { i * 8 } * scale;
                        let py = if vflip { (h - j - 1) * 8 } else { j * 8 } * scale;
                        tac.screen().blit(
                            x + px as i32,
                            y + py as i32,
                            &tac.sprite(id + (i + j * 16) as u16).unwrap(),
                            alpha,
                            hflip,
                            vflip,
                            scale,
                        );
                    }
                }
                Ok(())
            },
        )?;

        let btn = lua.create_function(|ctx, btn: u8| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.gamepads().player(btn / 8).btn(btn % 8))
        })?;

        let pix = lua.create_function(
            |ctx, (x, y, pix): (i32, i32, Option<u8>)| -> LuaResult<Option<u8>> {
                let tac = ctx.app_data_ref::<TAC70>().unwrap();
                match pix {
                    Some(pix) => {
                        tac.screen().set_pix(x, y, pix);
                        Ok(None)
                    }
                    None => Ok(Some(tac.screen().get_pix(x, y))),
                }
            },
        )?;

        let map = lua.create_function(
            |ctx,
             (x, y, w, h, sx, sy, alpha, scale, remap): (
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<u8>,
                Option<u32>,
                Option<LuaFunction>,
            )| {
                let tac = ctx.app_data_ref::<TAC70>().unwrap();
                let (x, y, w, h, sx, sy) = (
                    x.unwrap_or(0),
                    y.unwrap_or(0),
                    w.unwrap_or(30),
                    h.unwrap_or(17),
                    sx.unwrap_or(0),
                    sy.unwrap_or(0),
                );
                let scale = scale.unwrap_or(1); // TODO: use scale
                for i in 0..w {
                    for j in 0..h {
                        let (spr_id, flip, _rotate) = {
                            match &remap {
                                None => (
                                    tac.map().get(x + i, y + j).unwrap() as u16,
                                    LuaValue::Nil,
                                    Option::<i32>::None,
                                ),
                                Some(f) => f.call::<_, _>((
                                    tac.map().get(x + i, y + j).unwrap() as u16,
                                    x + i,
                                    y + j,
                                ))?,
                            }
                        };

                        let flip = match flip {
                            LuaValue::Boolean(b) if b => 1,
                            LuaValue::Integer(n) => n,
                            _ => 0,
                        };
                        tac.screen().blit(
                            sx + i * 8 * scale as i32,
                            sy + j * 8 * scale as i32,
                            &tac.sprite(spr_id).unwrap(),
                            alpha,
                            flip & 0b1 != 0,
                            flip & 0b10 != 0,
                            scale,
                        );
                    }
                }
                Ok(())
            },
        )?;

        let start_time = Instant::now();
        let time =
            lua.create_function(move |_, _: ()| Ok(start_time.elapsed().as_secs_f64() * 1000.0))?;

        globals.set("trace", trace)?;
        globals.set("mset", mset)?;
        globals.set("mget", mget)?;
        globals.set("cls", cls)?;
        globals.set("spr", spr)?;
        globals.set("btn", btn)?;
        globals.set("pix", pix)?;
        globals.set("time", time)?;
        globals.set("map", map)?;
        globals.set("rect", rect)?;
        globals.set("rectb", rectb)?;
        globals.set("print", print)?;

        drop(globals);

        let code = tac.code.clone();
        lua.set_app_data(tac);
        lua.load(&code).exec()?;

        Ok(Self { lua_ctx: lua })
    }

    pub fn boot(&mut self) -> LuaResult<()> {
        if let Ok(f) = self.lua_ctx.globals().get::<_, LuaFunction>("BOOT") {
            f.call::<_, _>(())?;
        }
        Ok(())
    }

    pub fn step(&mut self) -> LuaResult<()> {
        self.lua_ctx
            .globals()
            .get::<_, LuaFunction>("TIC")
            .unwrap()
            .call::<_, ()>(())?;
        Ok(())
    }

    pub fn state(&mut self) -> std::cell::RefMut<TAC70> {
        self.lua_ctx.app_data_mut().unwrap()
    }
}

#[cfg(test)]
mod test {}
