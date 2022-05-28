use std::env;

use ggez::{
    conf::WindowSetup,
    graphics::{window, Color, DrawParam, Image, Rect},
    *,
};
use rgb::ComponentBytes;
use tac_cart::Cartridge;
use tac_runtime::{Screen, TAC70Runtime};

struct TAC {
    runtime: TAC70Runtime,
}

impl TAC {
    fn new(ctx: &mut Context, runtime: TAC70Runtime) -> Self {
        Self { runtime }
    }
}

impl event::EventHandler<GameError> for TAC {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        self.runtime.step();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let (width, height) = graphics::drawable_size(ctx);
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, width, height)).unwrap();

        graphics::clear(ctx, Color::from((0.0, 0.0, 0.0)));
        let screen = self.runtime.screen().to_rgba();
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

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {}
}

fn main() {
    let cartridge_path = env::args().nth(1).unwrap();
    let runtime = TAC70Runtime::new(Cartridge::load(cartridge_path).unwrap().into()).unwrap();

    let (mut ctx, event_loop) = ContextBuilder::new("TAC 70", "DMClVG")
        .default_conf(conf::Conf {
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
