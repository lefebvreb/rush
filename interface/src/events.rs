use macroquad::prelude::*;

use crate::layout::*;

#[derive(Debug)]
pub enum Click {
    Square(chess::Square),
    Back,
    Restart,
    Invert,
    Rook,
    Knight,
    Bishop,
    Queen,
}

fn contains(x: f32, y: f32, rect: Rect) -> bool {
    rect.x <= x && x <= rect.x + rect.w && rect.y <= y && y <= rect.y + rect.h
}

pub fn get_click(reversed: bool) -> Option<Click> {
    static mut CLICK: bool = false;

    if unsafe {CLICK} {
        if !is_mouse_button_down(MouseButton::Left) {
            unsafe {CLICK = false}

            let (x, y) = mouse_position();
            let (sx, sy) = ((x / 64.0) as u32, (y / 64.0) as u32);
    
            if 0 < sx && sx < 9 && 0 < sy && sy < 9 {
                Some(Click::Square(if reversed {
                    chess::Square::from((sx-1, sy-1))
                } else {
                    chess::Square::from((sx-1, 8-sy))
                }))
            } else if contains(x, y, BACK_BUTTON) {
                Some(Click::Back)
            } else if contains(x, y, RESTART_BUTTON) {
                Some(Click::Restart)
            } else if contains(x, y, INVERT_BUTTON) {
                Some(Click::Invert)
            } else if contains(x, y, ROOK_PROMOTE) {
                Some(Click::Rook)
            } else if contains(x, y, KNIGHT_PROMOTE) {
                Some(Click::Knight)
            } else if contains(x, y, BISHOP_PROMOTE) {
                Some(Click::Bishop)
            } else if contains(x, y, QUEEN_PROMOTE) {
                Some(Click::Queen)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        if is_mouse_button_down(MouseButton::Left) {
            unsafe {CLICK = true}
        }
        None
    }
}