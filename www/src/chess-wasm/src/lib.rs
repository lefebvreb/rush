use chess::piece::Piece;
use js_sys::{Error as JsError};
use wasm_bindgen::prelude::*;

use std::str::FromStr;

use chess::prelude::*;

// Use the wee_alloc allocator instead of the std one to save space.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// The default fen position, used to initialize the board.
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Construct a javascript Error as a JsValue, from something that implements fmt::Display.
fn js_error(msg: &str) -> JsValue {
    JsError::new(msg).into()
}

// Tries to parse a square from a String.
fn parse_square(sq: &str) -> Result<Square, JsValue> {
    Square::from_str(sq).map_err(|_| js_error("Invalid square literal."))
}

/// The WasmChess struct, simply named "Chess" in JS is a class
/// representing a chess board, and wrapping some of it's functionnalities.
#[wasm_bindgen(js_name = Chess)]
#[derive(Debug)]
pub struct WasmChess {
    board: Board,
    legals: Vec<Move>,
}

#[wasm_bindgen(js_class = Chess)]
impl WasmChess {
    /// Constructs a new WasmChess object, from it's fen representation.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmChess {
        // Initialize the chess lib, if not done already.
        chess::init();

        let board = Board::new(DEFAULT_FEN).unwrap();
        let mut legals = Vec::new();
        movegen::legals(&board, &mut legals);

        WasmChess {board, legals}
    }

    /// A setter for the current position, given by a fen string.
    #[wasm_bindgen(method, js_name = setPosition)]
    pub fn set_position(&mut self, fen: &str, end: bool) -> Result<(), JsValue> {
        self.board = Board::new(fen).map_err(|_| js_error("Invalid fen literal."))?;

        self.legals.clear();
        if !end {
            movegen::legals(&self.board, &mut self.legals);
        }

        Ok(())
    }

    /// Returns true if the given move is legal.
    #[wasm_bindgen(method, js_name = isLegal)]
    pub fn is_legal(&self, from: String, to: String) -> Result<bool, JsValue> {
        let from = parse_square(&from)?;
        let to = parse_square(&to)?;

        Ok(self.legals.iter().any(|mv| mv.from() == from && mv.to() == to))
    }

    /// Returns true if the given move is a promotion. 
    #[wasm_bindgen(method, js_name = isPromotion)]
    pub fn is_promotion(&self, from: String, to: String) -> Result<bool, JsValue> {
        let from = parse_square(&from)?;
        let to = parse_square(&to)?;
        let (color, piece) = self.board.get_piece(from).ok_or(js_error("Invalid move from square."))?;
        
        Ok(match color {
            Color::White => to.y() == 7,
            Color::Black => to.y() == 0,
        } && piece == Piece::Pawn)
    }

    /// Returns true if the king is in check in this position.
    #[wasm_bindgen(method, js_name = isInCheck)]
    pub fn is_in_check(&self) -> bool {
        self.board.get_checkers().not_empty()
    }

    /// Returns true if the side to move is white.
    #[wasm_bindgen(method, js_name = isWhiteToMove)]
    pub fn is_white_to_move(&self) -> bool {
        self.board.get_side_to_move() == Color::White
    }

    /*/// Returns the set of squares that lead to legal moves from a given square.
    #[wasm_bindgen(method, js_name = getLegalsFrom)]
    pub fn get_legals_from(&self, from: String) -> Result<JsSet, JsValue> {
        let from = Square::from_str(&from).map_err(|_| "Invalid square literal.")?;

        let set = JsSet::new(&JsValue::UNDEFINED);
        for mv in self.legals.iter().filter(|mv| mv.from() == from) {
            set.add(&JsValue::from_str(mv.to_string().as_str()));
        }

        Ok(set)
    }*/

    // Compile only when in debug mode to save up some bytes.
    /// Prints self, using rust debug's format.
    #[cfg(debug_assertions)]
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}