//! The terminal side of the prompt: size, raw mode, mouse reporting, the cursor position and
//! input events, behind a trait so the prompt can be tested with a scripted terminal.

use std::{
    io::{self, Write},
    time::Duration,
};

use crossterm::{
    event::{self, DisableMouseCapture, Event},
    execute, terminal,
};

/// The terminal side of the prompt: its size, raw mode, the cursor and input events.
pub trait Backend {
    fn size(&self) -> io::Result<(u16, u16)>;
    fn set_raw_mode(&mut self, enabled: bool) -> io::Result<()>;
    /// Turns mouse reporting on or off. While it is on, clicks arrive as events instead of being
    /// handled by the terminal, which would otherwise turn some of them into arrow keys.
    fn set_mouse_capture(&mut self, enabled: bool) -> io::Result<()>;
    /// The cursor position as (column, row), or `None` if the terminal did not say.
    fn cursor_position(&mut self) -> io::Result<Option<(u16, u16)>>;
    /// Blocks until the next event.
    fn read(&mut self) -> io::Result<Event>;
    /// Whether [`read`](Self::read) would return without waiting.
    fn has_pending(&mut self) -> io::Result<bool>;
}

/// The real terminal.
pub struct Tty;

impl Backend for Tty {
    fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    fn set_raw_mode(&mut self, enabled: bool) -> io::Result<()> {
        if enabled {
            terminal::enable_raw_mode()
        } else {
            terminal::disable_raw_mode()
        }
    }

    fn set_mouse_capture(&mut self, enabled: bool) -> io::Result<()> {
        let mut stderr = io::stderr();
        if !enabled {
            return execute!(stderr, DisableMouseCapture);
        }

        // Windows consoles report the mouse through the console input mode
        #[cfg(windows)]
        return execute!(stderr, crossterm::event::EnableMouseCapture);

        // Presses and releases in the SGR format, without the motion reports that would flood a
        // slow connection
        #[cfg(not(windows))]
        {
            stderr.write_all(b"\x1b[?1000h\x1b[?1006h")?;
            stderr.flush()
        }
    }

    #[cfg(unix)]
    fn cursor_position(&mut self) -> io::Result<Option<(u16, u16)>> {
        Ok(query_cursor_position(Duration::from_secs(1)))
    }

    #[cfg(windows)]
    fn cursor_position(&mut self) -> io::Result<Option<(u16, u16)>> {
        // The console answers directly, no round trip through the terminal
        crossterm::cursor::position().map(Some)
    }

    fn read(&mut self) -> io::Result<Event> {
        event::read()
    }

    fn has_pending(&mut self) -> io::Result<bool> {
        event::poll(Duration::ZERO)
    }
}

/// Asks the terminal where the cursor is (`ESC [ 6 n`) and reads the answer, giving up after
/// `timeout`. Both go through `/dev/tty`, so this works while the shell captures stdout, and
/// cannot hang when there is no terminal. Over SSH the answer takes one round trip. Raw mode
/// must be on.
#[cfg(unix)]
fn query_cursor_position(timeout: Duration) -> Option<(u16, u16)> {
    use std::{fs::File, io::Read, os::fd::AsRawFd, time::Instant};

    let mut tty = File::options()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .ok()?;
    tty.write_all(b"\x1b[6n").ok()?;
    tty.flush().ok()?;

    let deadline = Instant::now() + timeout;
    let mut reply = Vec::new();
    let mut byte = [0];
    while reply.len() < 32 {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let mut poll = libc::pollfd {
            fd: tty.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let millis = libc::c_int::try_from(remaining.as_millis()).unwrap_or(libc::c_int::MAX);
        // SAFETY: polls one valid descriptor, owned by `tty` for the whole call
        if unsafe { libc::poll(&raw mut poll, 1, millis) } <= 0 {
            return None;
        }
        if tty.read(&mut byte).ok()? == 0 {
            return None;
        }
        reply.push(byte[0]);
        if byte[0] == b'R' {
            return parse_cursor_reply(&reply);
        }
    }
    None
}

/// Parses the terminal's answer `ESC [ row ; column R` (counted from 1) into a (column, row)
/// counted from 0. Anything typed just before the answer is ignored.
#[cfg(any(unix, test))]
fn parse_cursor_reply(reply: &[u8]) -> Option<(u16, u16)> {
    let reply = std::str::from_utf8(reply).ok()?;
    let start = reply.rfind("\u{1b}[")?;
    let (row, column) = reply[start + 2..].strip_suffix('R')?.split_once(';')?;
    Some((
        column.parse::<u16>().ok()?.checked_sub(1)?,
        row.parse::<u16>().ok()?.checked_sub(1)?,
    ))
}

/// Windows consoles only understand escape sequences once virtual terminal processing is turned
/// on for the handle that is written to. The prompt draws on stderr, since stdout carries the
/// result.
#[cfg(windows)]
pub fn enable_ansi_output() {
    use windows::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_ERROR_HANDLE,
    };

    // SAFETY: plain Win32 calls on the process's own stderr handle
    unsafe {
        if let Ok(handle) = GetStdHandle(STD_ERROR_HANDLE) {
            let mut mode = CONSOLE_MODE::default();
            if GetConsoleMode(handle, &raw mut mode).is_ok() {
                let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

/// Puts the terminal back in a usable state. For panic hooks, where the [`Ui`](super::Ui) cannot do it.
pub fn restore_terminal() {
    let mut stderr = io::stderr();
    let _ = execute!(stderr, DisableMouseCapture);
    let _ = terminal::disable_raw_mode();
    let _ = stderr.write_all(b"\x1b[?25h");
    let _ = stderr.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_replies_are_parsed() {
        assert_eq!(parse_cursor_reply(b"\x1b[5;1R"), Some((0, 4)));
        assert_eq!(parse_cursor_reply(b"typed\x1b[2;3R"), Some((2, 1)));
        assert_eq!(parse_cursor_reply(b"\x1b[0;1R"), None);
        assert_eq!(parse_cursor_reply(b"garbage"), None);
    }
}
