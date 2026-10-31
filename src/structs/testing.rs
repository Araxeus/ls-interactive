//! Helpers shared by the tests of the rendering code.

use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

/// A writer that remembers every `write` call separately, so tests can check that a frame is
/// written in one piece.
#[derive(Clone, Default)]
pub struct Sink {
    writes: Rc<RefCell<Vec<Vec<u8>>>>,
    /// Every byte ever written, never taken
    log: Rc<RefCell<Vec<u8>>>,
}

impl Sink {
    /// The writes since the last call.
    pub fn take(&self) -> Vec<Vec<u8>> {
        self.writes.take()
    }

    /// Every byte written so far.
    pub fn log(&self) -> Vec<u8> {
        self.log.borrow().clone()
    }
}

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writes.borrow_mut().push(buf.to_vec());
        self.log.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A terminal size for the emulator, which does not accept zero.
pub fn nz(n: u16) -> std::num::NonZeroU16 {
    std::num::NonZeroU16::new(n).expect("size must not be zero")
}
