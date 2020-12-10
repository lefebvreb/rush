use macroquad::prelude::*;

pub struct Assets {
    atlas: Texture2D,
}

#[derive(Copy, Clone)]
pub enum Texture {
    WhiteSquare,
    BlackSquare,
    LightSquare,
    DarkSquare,
    Select,
    Legal,
    Back,
    Restart,
    Invert,
}

impl Assets {
    fn param(x: i32, y: i32) -> DrawTextureParams {
        DrawTextureParams {
            dest_size: Some(Vec2::new(64.0, 64.0)),
            source: Some(Rect {
                x: x as f32 * 128.0,
                y: y as f32 * 128.0,
                w: 128.0,
                h: 128.0,
            }),
            ..Default::default()
        }        
    }

    pub async fn load() -> Assets {
        Assets {
            atlas: load_texture("assets.png").await,
        }
    }

    pub fn draw_piece(&self, color: chess::Color, piece: chess::Piece, x: f32, y: f32) {
        draw_texture_ex(
            self.atlas,
            x,
            y,
            WHITE,
            Assets::param(piece as i32, color as i32),
        )
    }

    pub fn draw_texture(&self, texture: Texture, x: f32, y: f32) {
        draw_texture_ex(
            self.atlas,
            x,
            y,
            WHITE,
            match texture {
                Texture::WhiteSquare => Assets::param(0, 2),
                Texture::BlackSquare => Assets::param(1, 2),
                Texture::LightSquare => Assets::param(2, 2),
                Texture::DarkSquare => Assets::param(3, 2),
                Texture::Select => Assets::param(4, 2),
                Texture::Legal => Assets::param(5, 2),
                Texture::Back => Assets::param(0, 3),
                Texture::Restart => Assets::param(1, 3),
                Texture::Invert => Assets::param(2, 3),
            },
        )
    }
}