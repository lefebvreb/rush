use js_sys::{Array as JsArray, Error as JsError, Set as JsSet};
use wasm_bindgen::prelude::*;

use std::fmt;

use chess::prelude::*;

// Use the wee_alloc allocator instead of the std one to save space.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Construct a javascript Error as a JsValue, from something that implements fmt::Display.
fn js_error<E: fmt::Display>(err: E) -> JsValue {
    JsError::new(err.to_string().as_str()).into()
}

/// The WasmChess struct, simply named "Chess" in JS is a class
/// representing a chess board, and wrapping some of it's functionnalities.
/// It also contains a move history for undoing moves if needed.
#[wasm_bindgen(js_name = Chess)]
#[derive(Debug)]
pub struct WasmChess {
    board: Board,
    history: Vec<Move>,
}

#[wasm_bindgen(js_class = Chess)]
impl WasmChess {
    /// Constructs a new WasmChess object, from it's fen representation.
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Result<WasmChess, JsValue> {
        match Board::new(fen) {
            Ok(board) => Ok(WasmChess {
                board,
                history: Vec::new(),
            }),
            Err(err) => Err(js_error(err)),
        }
    }

    // ================================ getters

    /// A getter to return the move history, as a list of js strings.
    #[wasm_bindgen(method, getter)]
    pub fn history(&self) -> JsArray {
        let arr = JsArray::new_with_length(self.history.len() as u32);
        for (i, mv) in self.history.iter().enumerate() {
            arr.set(i as u32, JsValue::from_str(mv.to_string().as_str()));
        }
        arr
    }

    /// A getter to get the fen string of the current position.
    #[wasm_bindgen(method, getter)]
    pub fn fen(&self) -> String {
        self.board.to_string()
    }

    // ================================ methods

    /// Parses and plays the given move, if valid.
    #[wasm_bindgen(method)]
    pub fn play(&mut self, mv: String) -> Result<(), JsValue> {
        match self.board.parse_move(mv.as_str()) {
            Ok(mv) => {
                self.board.do_move(mv);
                self.history.push(mv);
                Ok(())
            },
            Err(err) => Err(js_error(err)),
        }
    }

    /// A setter for the current position, given by a fen string.
    #[wasm_bindgen(method, js_name = setFen)]
    pub fn set_fen(&mut self, fen: &str) -> Result<(), JsValue> {
        match Board::new(fen) {
            Ok(board) => {
                self.board = board;
                self.history.clear();
                Ok(())
            },
            Err(err) => Err(js_error(err)),
        }
    }

    /// Generates all legals move and returns them as a javascript set of strings.
    #[wasm_bindgen(method, js_name = getLegalMoves)]
    pub fn legals(&mut self) -> JsSet {
        let mut buffer = Vec::new();
        movegen::legals(&self.board, &mut buffer);

        let set = JsSet::new(&JsValue::UNDEFINED);
        for mv in buffer.iter() {
            set.add(&JsValue::from_str(mv.to_string().as_str()));
        }
        set
    }

    /// Undoes the last move, if there is one.
    #[wasm_bindgen(method)]
    pub fn back(&mut self) -> Result<(), JsValue> {
        if self.history.is_empty() {
            Err(js_error("There are no moves to undo."))
        } else {
            let mv = self.history.pop().unwrap();
            self.board.undo_move(mv);
            Ok(())
        }
    }

    /// Prints self as debug.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// The start function intializes the chess lib.
#[wasm_bindgen]
pub fn start() {
    chess::init();
}   