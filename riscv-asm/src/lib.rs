#![feature(trim_prefix_suffix)]
pub mod assembler;
pub mod error;
pub mod tokenizer;
pub use assembler::assemble;
