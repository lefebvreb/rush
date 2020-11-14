#![allow(dead_code, unused_variables, unused_macros)]

use miniquad::conf::Conf;
use miniquad::UserData;

mod state;
use state::State;

const NAME: &'static str = concat!("Benji's chess engine v", env!("CARGO_PKG_VERSION"));
const WIDTH: i32 = 720;
const HEIGHT: i32 = 480;

fn main() {
    let conf = Conf {
        window_title: NAME.to_string(),
        window_width: WIDTH,
        window_height: HEIGHT,
        ..Default::default()
    };

    miniquad::start(conf, |ctx| UserData::owning(State::default(), ctx));
}