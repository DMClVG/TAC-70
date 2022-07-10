use std::{error::Error, time::Instant};

use mlua::prelude::*;
use tac_core::{PixBuf, TAC70};

pub struct TAC70Runtime {
    pub lua_ctx: Lua,
}

impl TAC70Runtime {
    pub fn new(tac: TAC70) -> Result<Self, Box<dyn Error>> {
        let lua = Lua::new();

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

        let spr = lua.create_function(
            |ctx,
             (id, x, y, alpha, scale, flip, rot, w, h): (
                u16,
                i32,
                i32,
                Option<u8>,
                Option<u32>,
                Option<LuaValue>,
                Option<u32>,
                Option<u32>,
                Option<u32>,
            )| {
                let tac = ctx.app_data_ref::<TAC70>().unwrap();
                let (scale, flip, _rot, w, h) = (
                    scale.unwrap_or(1),
                    flip.unwrap_or(LuaValue::Boolean(false)),
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
                Option<i32>,
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
                let _scale = scale.unwrap_or(1); // TODO: use scale
                for i in 0..w {
                    for j in 0..h {
                        let (spr_id, flip, _rotate) = {
                            match &remap {
                                None => (
                                    tac.map().get(x + i, y + j).unwrap() as u16,
                                    Option::<i32>::None,
                                    Option::<i32>::None,
                                ),
                                Some(f) => f.call::<_, _>((
                                    tac.map().get(x + i, y + j).unwrap() as u16,
                                    x + i,
                                    y + j,
                                ))?,
                            }
                        };
                        tac.screen().blit(
                            sx + i * 8,
                            sy + j * 8,
                            &tac.sprite(spr_id).unwrap(),
                            alpha,
                            flip.unwrap_or(0) & 0b1 != 0,
                            flip.unwrap_or(0) & 0b10 != 0,
                            1,
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

        drop(globals);

        let code = tac.code.clone();
        lua.set_app_data(tac);
        lua.load(&code).exec()?;

        Ok(Self { lua_ctx: lua })
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
