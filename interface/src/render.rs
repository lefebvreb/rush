use macroquad::prelude::*;

use crate::assets::{Assets, Texture};
use crate::layout::*;

fn is_black(square: chess::Square) -> bool {
    (square.x() + square.y()) & 1 == 1
}

fn get_xy(square: chess::Square, reversed: bool) -> (f32, f32) {
    (
        (square.x() + 1) as f32 * 64.0,
        if reversed {
            square.y() + 1
        } else {
            8 - square.y()
        } as f32 * 64.0,
    )
}

fn make_border(rect: Rect) {
    draw_line(rect.x, rect.y, rect.x + rect.w, rect.y, 3.0, BLACK);
    draw_line(rect.x, rect.y, rect.x, rect.y + rect.h, 3.0, BLACK);
    draw_line(rect.x + rect.w, rect.y, rect.x + rect.w, rect.y + rect.h, 3.0, BLACK);
    draw_line(rect.x, rect.y + rect.h, rect.x + rect.w, rect.y + rect.h, 3.0, BLACK);
}

pub fn render_squares(assets: &Assets, reversed: bool) {
    make_border(BOARD);
    for square in &chess::Square::SQUARES {
        let xy = get_xy(*square, reversed);
        assets.draw_texture(if is_black(*square) {
                Texture::BlackSquare
            } else {
                Texture::WhiteSquare
            },
            xy.0,
            xy.1,
        )
    }
}

pub fn render_pieces(assets: &Assets, board: &chess::Board, reversed: bool) {
    for square in &chess::Square::SQUARES {
        if let Some((color, piece)) = board.get_piece(*square) {
            assets.draw_piece(
                color,
                piece,
                (square.x() + 1) as f32 * 64.0,
                (if reversed {
                    square.y()
                } else {
                    7 - square.y()
                } + 1 ) as f32 * 64.0,
            )
        }
    }
}

pub fn render_buttons(assets: &Assets) {
    for (rect, texture) in [BACK_BUTTON, RESTART_BUTTON, INVERT_BUTTON].iter()
        .zip([Texture::Back, Texture::Restart, Texture::Invert].iter())
    {
        make_border(*rect);
        assets.draw_texture(*texture, rect.x, rect.y);
    }
}

pub fn render_promotion(assets: &Assets, color: chess::Color) {
    for (rect, piece) in [ROOK_PROMOTE, KNIGHT_PROMOTE, BISHOP_PROMOTE, QUEEN_PROMOTE].iter()
        .zip([chess::Piece::Rook, chess::Piece::Knight, chess::Piece::Bishop, chess::Piece::Queen].iter())
    {
        make_border(*rect);
        assets.draw_piece(color, *piece, rect.x, rect.y,);
    }
}

pub fn render_last_move(assets: &Assets, from: chess::Square, to: chess::Square, reversed: bool) {
    for square in &[from, to] {
        let xy = get_xy(*square, reversed);
        assets.draw_texture(if is_black(*square) {
                Texture::DarkSquare
            } else {
                Texture::LightSquare
            },
            xy.0,
            xy.1,
        )
    }
}

pub fn render_moves(assets: &Assets, select: chess::Square, legals: impl Iterator<Item = chess::Square>, reversed: bool) {
    let xy = get_xy(select, reversed);
    assets.draw_texture(
        Texture::Select,
        xy.0,
        xy.1,
    );

    for square in legals {
        let xy = get_xy(square, reversed);
        assets.draw_texture(
            Texture::Legal,
            xy.0,
            xy.1,
        );
    }
}