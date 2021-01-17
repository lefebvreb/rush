#![allow(dead_code, unused_variables, unused_macros)]

use actix::{Actor, Context};

struct Engine;

impl Actor for Engine {
    type Context = Context<Self>;
}