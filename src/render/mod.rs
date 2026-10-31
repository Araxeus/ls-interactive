//! Drawing on the terminal: escape sequence handling and the screen differ.

pub mod ansi;
pub mod screen;
#[cfg(test)]
pub mod testing;

pub use screen::Screen;
