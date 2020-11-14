use miniquad::{Context, EventHandler, MouseButton};

#[derive(Default, Debug)]
pub struct State {
    need_redraw: bool,
}

impl EventHandler for State {
    fn mouse_button_up_event(&mut self, ctx: &mut Context, _: MouseButton, x: f32, y: f32) {
        println!("click: ({}, {})", x, y);
    }

    fn update(&mut self, _: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear(Some((1., 1., 1., 1.)), None, None);
    }
}