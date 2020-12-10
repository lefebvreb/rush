#![allow(dead_code, unused_variables, unused_macros)]

use macroquad::prelude::*;
use macroquad::rand::srand;

use std::time::SystemTime;

mod assets;
use assets::Assets;

mod events;

mod layout;

mod render;

mod state;
use state::App;

fn window_conf() -> Conf {
    Conf {
        window_title: format!("Benji's chess engine v{}", env!("CARGO_PKG_VERSION")),
        window_width: 768,
        window_height: 640,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    srand(
        SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
    );

    let assets = Assets::load().await;
    let mut state = App::default();

    loop {
        clear_background(WHITE);
        state.act(&assets);
        next_frame().await
    }
}