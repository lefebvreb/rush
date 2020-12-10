use macroquad::prelude::Rect;

pub const BOARD: Rect = Rect {
    x: 64.0,
    y: 64.0,
    w: 512.0,
    h: 512.0,
};

pub const BACK_BUTTON: Rect = Rect {
    x: 608.0,
    y: 96.0,
    w: 64.0, 
    h: 64.0
};

pub const RESTART_BUTTON: Rect = Rect {
    x: 608.0,
    y: 192.0,
    w: 64.0, 
    h: 64.0
};

pub const INVERT_BUTTON: Rect = Rect {
    x: 608.0,
    y: 288.0,
    w: 64.0, 
    h: 64.0
};

pub const ROOK_PROMOTE: Rect = Rect {
    x: 608.0,
    y: 384.0,
    w: 64.0,
    h: 64.0,
};

pub const KNIGHT_PROMOTE: Rect = Rect {
    x: 608.0,
    y: 448.0,
    w: 64.0,
    h: 64.0,
};

pub const BISHOP_PROMOTE: Rect = Rect {
    x: 672.0,
    y: 448.0,
    w: 64.0,
    h: 64.0,
};

pub const QUEEN_PROMOTE: Rect = Rect {
    x: 672.0,
    y: 384.0,
    w: 64.0,
    h: 64.0,
};