// temporary
#![allow(dead_code, unused_variables, unused_macros)]

extern crate chess;

mod params;
mod engine;
mod errors;
mod eval;
mod search;
mod table;

mod prelude {
    pub use crate::engine::Engine;
}