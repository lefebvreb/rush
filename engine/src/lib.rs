#![allow(unused)]

mod params;
mod engine;
mod eval;
mod heuristics;
mod movepick;
mod search;
mod table;
mod utils;

/// The version of the engine.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Export the Engine struct.
pub use self::engine::Engine;