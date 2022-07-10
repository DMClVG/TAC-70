use std::env;

use ggez::{
    conf::{WindowSetup, WindowMode},
    graphics::{Color, DrawParam, Rect},
    *, event::{KeyMods, KeyCode},
};
use rgb::ComponentBytes;
use tac_core::{Screen, PixBuf};
use tac_cart::Cartridge;
use tac_runtime::TAC70Runtime;

struct TAC {
    runtime: TAC70Runtime,
}

impl TAC {
    fn new(_ctx: &mut Context, runtime: TAC70Runtime) -> Self {
        Self { runtime }
    }
}

impl event::EventHandler<GameError> for TAC {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        
        self.runtime.step().map_err(|err| GameError::EventLoopError(err.to_string()))?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let tac = self.runtime.state();
        let (width, height) = graphics::drawable_size(ctx);
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, width, height)).unwrap();

        graphics::clear(ctx, Color::from((0.0, 0.0, 0.0)));
        let palette = tac.palette();
        let screen = tac.screen().to_rgba(&palette);
        let screen = screen.as_bytes();
        let mut screen_image =
            graphics::Image::from_rgba8(ctx, Screen::WIDTH as u16, Screen::HEIGHT as u16, &screen)?;
        screen_image.set_filter(graphics::FilterMode::Nearest);

        let upscale = height / screen_image.height() as f32;
        graphics::draw(
            ctx,
            &screen_image,
            DrawParam::new().scale([upscale, upscale]),
        )?;
        graphics::present(ctx)?;

        Ok(())
    }

    fn key_down_event( &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool) {
        if repeat { return }
        let tac = self.runtime.state();
        let gamepads = tac.gamepads();
        
        match keycode {
            KeyCode::Up => gamepads.player(0).set_btn(0, true),
            KeyCode::Down => gamepads.player(0).set_btn(1, true),
            KeyCode::Left => gamepads.player(0).set_btn(2, true),
            KeyCode::Right => gamepads.player(0).set_btn(3, true),
            KeyCode::Z => gamepads.player(0).set_btn(4, true),
            KeyCode::X => gamepads.player(0).set_btn(5, true),
            KeyCode::A => gamepads.player(0).set_btn(6, true),
            KeyCode::S => gamepads.player(0).set_btn(7, true),
            _ => {}
        }
    }

    fn key_up_event( &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods) {
        let tac = self.runtime.state();
        let gamepads = tac.gamepads();
        
        match keycode {
            KeyCode::Up => gamepads.player(0).set_btn(0, false),
            KeyCode::Down => gamepads.player(0).set_btn(1, false),
            KeyCode::Left => gamepads.player(0).set_btn(2, false),
            KeyCode::Right => gamepads.player(0).set_btn(3, false),
            KeyCode::Z => gamepads.player(0).set_btn(4, false),
            KeyCode::X => gamepads.player(0).set_btn(5, false),
            KeyCode::A => gamepads.player(0).set_btn(6, false),
            KeyCode::S => gamepads.player(0).set_btn(7, false),
            _ => {}
        }
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
}

fn main() {
    const SCALE: f32 = 3.0;

    let cartridge_path = env::args().nth(1).unwrap();
    println!("Loading {}..", &cartridge_path);
    let cart = Cartridge::load(cartridge_path).unwrap();
    dbg!(&cart);
    let runtime = TAC70Runtime::new(cart.into()).unwrap();

    let (mut ctx, event_loop) = ContextBuilder::new("TAC-70", "DMClVG")
        .default_conf(conf::Conf {
            window_mode: WindowMode {
                width: 240.0 * SCALE,
                height: 136.0 * SCALE,
                ..Default::default()
            },
            window_setup: WindowSetup {
                title: "TAC-70".to_string(),
                vsync: true,
                ..Default::default()
            }, ..Default::default()
        })
        .add_resource_path("./resources/")
        .build()
        .unwrap();

    let tac = TAC::new(&mut ctx, runtime);
    event::run(ctx, event_loop, tac);
}