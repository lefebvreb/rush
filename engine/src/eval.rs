use chess::{Color, Game};

pub(crate) fn eval(game: &Game) -> f32 {
    let mut score = 0.0;

    // TODO: eval

    match game.get_color() {
        Color::White =>  score,
        Color::Black => -score,
    }
}