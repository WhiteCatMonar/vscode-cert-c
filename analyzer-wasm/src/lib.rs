//! CERT-Cチェック用の最小Rust/WASM解析コア。
//!
//! Cソースを軽量な字句解析で処理し、ルール別パターン検出を提供する。
//!
//! TODO: C99 Parser、AST、CFG、Data Flowへ段階的に置き換える。

mod analyzer;
mod diagnostic;
mod lexer;
mod rules;
mod utils;
mod wasm_api;

pub use wasm_api::{alloc, analyze_source, dealloc};
