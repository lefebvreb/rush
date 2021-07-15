extern crate wasm_bindgen;
extern crate wee_alloc;

extern crate chess;
extern crate engine;

use wasm_bindgen::prelude::*;

// The JsChess type.
mod jschess;

// Use the wee_alloc allocator instead of the std one to save space.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// The start function intializes the chess lib.
#[wasm_bindgen]
pub fn start() {
    chess::init();
}   