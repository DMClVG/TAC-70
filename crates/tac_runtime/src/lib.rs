use std::error::Error;

use mlua::prelude::*;
use tac_core::{TAC70, PixBuf};

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
            tac.map().get(x, y).ok_or(LuaError::RuntimeError("MGET outside map".to_string()))
        })?;

        let mset = lua.create_function(|ctx, (x, y, id): (i32, i32, u8)| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.map().set(x, y, id))
        })?;

        let cls = lua.create_function(|ctx, pix: u8| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.screen().clear(pix))
        })?;

        let spr = lua.create_function(|ctx, (id, x, y, alpha): (u16, i32, i32, Option<u8>)| {
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.screen().blit(x, y, &tac.sprite(id).unwrap(), alpha))
        })?;
        
        let btn = lua.create_function(|ctx, btn: u8|{
            let tac = ctx.app_data_ref::<TAC70>().unwrap();
            Ok(tac.gamepads().player(btn / 8).btn(btn % 8))
        })?;

        globals.set("trace", trace)?;
        globals.set("mset", mset)?;
        globals.set("mget", mget)?;
        globals.set("cls", cls)?;
        globals.set("spr", spr)?;
        globals.set("btn", btn)?;

        drop(globals);

        lua.load(&tac.code).exec()?;
        lua.set_app_data(tac);

        Ok(Self {
            lua_ctx: lua,
        })
    }

    pub fn step(&mut self) -> LuaResult<()> {
        self.lua_ctx.globals().get::<_, LuaFunction>("TIC").unwrap().call::<_, ()>(())?;
        Ok(())
    }

    pub fn state(&mut self) -> std::cell::RefMut<TAC70> {
        self.lua_ctx.app_data_mut().unwrap()
    }
}




#[cfg(test)]
mod test {}
