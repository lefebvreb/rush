use js_sys::{Error as JsError, Set as JsSet};
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
    pub fn set_position(&mut self, fen: &str, draw: bool) -> Result<(), JsValue> {
        self.board = Board::new(fen).map_err(|_| js_error("Invalid fen literal."))?;

        self.legals.clear();
        if !draw {
            movegen::legals(&self.board, &mut self.legals);
        }

        Ok(())
    }

    /// Returns true if the given move is legal.
    #[wasm_bindgen(method, js_name = isLegal)]
    pub fn is_legal(&mut self, mv: String) -> Result<bool, JsValue> {
        let mv = self.board.parse_move(mv.as_str()).map_err(|_|"Invalid move literal.")?;
        Ok(self.legals.contains(&mv))
    }

    /// Returns the set of squares that lead to legal moves from a given square.
    #[wasm_bindgen(method, js_name = getLegalsFrom)]
    pub fn get_legals_from(&self, from: String) -> Result<JsSet, JsValue> {
        let from = Square::from_str(&from).map_err(|_| "Invalid square literal.")?;

        let set = JsSet::new(&JsValue::UNDEFINED);
        for mv in self.legals.iter().filter(|mv| mv.from() == from) {
            set.add(&JsValue::from_str(mv.to_string().as_str()));
        }

        Ok(set)
    }

    /// Prints self as debug.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}