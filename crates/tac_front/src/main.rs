use std::env;

use rgb::ComponentBytes;
use tac_cart::Cartridge;
use tac_core::{PixBuf, Screen};
use tac_runtime::TAC70Runtime;

use macroquad::prelude::*;

#[macroquad::main("TAC-70")]
async fn main() {
    let cartridge_path = env::args().nth(1).unwrap();
    println!("Loading {}..", &cartridge_path);
    let cart = Cartridge::load(cartridge_path).unwrap();
    dbg!(&cart);
    let mut runtime = TAC70Runtime::new(cart.into()).unwrap();

    runtime.boot().unwrap();
    loop {
        runtime.step().unwrap();

        let (width, height) = (screen_width(), screen_height());

        let upscale = (height / Screen::HEIGHT as f32)
            .min(width / Screen::WIDTH as f32)
            .floor()
            .max(1.0);

        let (offx, offy) = (
            ((width - Screen::WIDTH as f32 * upscale) / 2.0).ceil(),
            ((height - Screen::HEIGHT as f32 * upscale) / 2.0).ceil(),
        );

        let state = runtime.state();
        let gamepads = state.gamepads();

        gamepads.player(0).set_btn(0, is_key_down(KeyCode::Up));
        gamepads.player(0).set_btn(1, is_key_down(KeyCode::Down));
        gamepads.player(0).set_btn(2, is_key_down(KeyCode::Left));
        gamepads.player(0).set_btn(3, is_key_down(KeyCode::Right));
        gamepads.player(0).set_btn(4, is_key_down(KeyCode::Z));
        gamepads.player(0).set_btn(5, is_key_down(KeyCode::X));
        gamepads.player(0).set_btn(6, is_key_down(KeyCode::A));
        gamepads.player(0).set_btn(7, is_key_down(KeyCode::S));

        let (mx, my) = mouse_position();
        let (ml, mm, mr) = (
            is_mouse_button_down(MouseButton::Left),
            is_mouse_button_down(MouseButton::Middle),
            is_mouse_button_down(MouseButton::Right),
        );
        let (scrollx, scrolly) = mouse_wheel();
        let (mx, my) = (
            ((mx - offx) / upscale).max(0.0) as u8,
            ((my - offy) / upscale).max(0.0) as u8,
        );

        state.mouse().set(
            mx,
            my,
            ml,
            mm,
            mr,
            scrollx.round() as i8,
            scrolly.round() as i8,
        );

        // ==== DRAW ====
        clear_background(BLACK);

        let screen = state.screen().to_rgba(&state.palette());
        let screen = screen.as_bytes();

        let tex = Texture2D::from_rgba8(Screen::WIDTH as u16, Screen::HEIGHT as u16, screen);
        tex.set_filter(FilterMode::Nearest);
        draw_texture_ex(
            tex,
            offx,
            offy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    Screen::WIDTH as f32 * upscale,
                    Screen::HEIGHT as f32 * upscale,
                )),
                ..Default::default()
            },
        );

        next_frame().await
    }
}
