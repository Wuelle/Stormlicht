#![feature(iter_advance_by, associated_type_defaults)]

pub mod bytecode;
pub mod parser;
mod value;

pub use value::{Number, Value};
