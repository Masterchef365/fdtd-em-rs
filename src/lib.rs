#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::FdtdApp;
pub mod common;
pub mod field_vis;
pub mod sim;
pub mod streamers;
pub mod wire_editor_3d;
mod circuit_editor;
