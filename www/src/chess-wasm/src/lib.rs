use js_sys::{Error as JsError, Set as JsSet};
use wasm_bindgen::prelude::*;

use std::fmt;

use chess::prelude::*;

// Use the wee_alloc allocator instead of the std one to save space.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Construct a javascript Error as a JsValue, from something that implements fmt::Display.
fn js_error<E: fmt::Display>(err: E) -> JsValue {
    JsError::new(err.to_string().as_str()).into()
}

/// The WasmChess struct, simply named "Chess" in JS is a class
/// representing a chess board, and wrapping some of it's functionnalities.
#[wasm_bindgen(js_name = Chess)]
#[derive(Debug)]
pub struct WasmChess {
    board: Board,
    buffer: Vec<Move>,
}

#[wasm_bindgen(js_class = Chess)]
impl WasmChess {
    /// Constructs a new WasmChess object, from it's fen representation.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmChess {
        // Initialize the chess lib, if not done already.
        chess::init();

        // Parses the game board.
        WasmChess {
            board: Board::new(DEFAULT_FEN).unwrap(),
            buffer: Vec::new(),
        }
    }

    // ================================ getters

    /// A getter to get the fen string of the current position.
    #[wasm_bindgen(method, js_name = getFen)]
    pub fn fen(&self) -> String {
        self.board.to_string()
    }

    // ================================ methods

    /// Parses and plays the given move, if valid.
    #[wasm_bindgen(method)]
    pub fn play(&mut self, mv: String) -> Result<(), JsValue> {
        let mv = self.board.parse_move(mv.as_str()).map_err(|e| js_error(e))?;
        self.board.do_move(mv);
        Ok(())
    }

    /// A setter for the current position, given by a fen string.
    #[wasm_bindgen(method, js_name = setFen)]
    pub fn set_fen(&mut self, fen: &str) -> Result<(), JsValue> {
        self.board = Board::new(fen).map_err(|e| js_error(e))?;
        Ok(())
    }

    /// Generates all legals move and returns them as a javascript set of strings.
    #[wasm_bindgen(method, js_name = getLegalMoves)]
    pub fn legals(&mut self) -> JsSet {
        self.buffer.clear();
        movegen::legals(&self.board, &mut self.buffer);

        let set = JsSet::new(&JsValue::UNDEFINED);
        for mv in self.buffer.iter() {
            set.add(&JsValue::from_str(mv.to_string().as_str()));
        }
        set
    }

    /// Prints self as debug.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}