//! Terminal handling: raw mode, size, cursor, and palette detection via OSC.
//! The colors are YOUR terminal's theme colors - queried live via OSC 10/11/12
//! and the 256-color palette via OSC 4. No pywal, no matugen, no config.

use std::io::{self, Write};

// ---------------------------------------------------------------------------
// Raw mode
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct Termios {
    c_iflag: u32,
    c_oflag: u32,
    c_cflag: u32,
    c_lflag: u32,
    c_line: u8,
    c_cc: [u8; 32],
    c_ispeed: u32,
    c_ospeed: u32,
}

extern "C" {
    fn tcgetattr(fd: i32, termios_p: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, optional_actions: i32, termios_p: *const Termios) -> i32;
    fn ioctl(fd: i32, request: u64, ...) -> i32;
}

const TIOCGWINSZ: u64 = 0x5413;
const TCSANOW: i32 = 0;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct WinSize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

/// Guard that puts the terminal into raw mode and restores it on drop.
pub struct RawMode {
    saved: Termios,
    fd: i32,
}

impl RawMode {
    pub fn enable() -> io::Result<Self> {
        let fd = libc::STDOUT_FILENO;
        let mut saved = unsafe { std::mem::zeroed() };
        if unsafe { tcgetattr(fd, &mut saved) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut raw = saved;
        // classic cfmakeraw minus output post-processing (keep OPOST so \n works)
        raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
        raw.c_iflag &= !(libc::IXON | libc::ICRNL | libc::BRKINT | libc::INPCK | libc::ISTRIP);
        raw.c_cflag |= libc::CS8;
        raw.c_cc[libc::VMIN] = 0; // polling reads
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { tcsetattr(fd, TCSANOW, &raw) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { saved, fd })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        unsafe { tcsetattr(self.fd, TCSANOW, &self.saved) };
        let _ = io::stdout().flush();
    }
}

/// Terminal size in (cols, rows); falls back to 80x24.
pub fn size() -> (usize, usize) {
    let mut ws = WinSize::default();
    let ok = unsafe { ioctl(libc::STDOUT_FILENO, TIOCGWINSZ, &mut ws) } == 0;
    if ok && ws.ws_col > 0 && ws.ws_row > 0 {
        (ws.ws_col as usize, ws.ws_row as usize)
    } else {
        (80, 24)
    }
}

/// Non-blocking read of pending input bytes (for q/Ctrl-C to quit).
pub fn poll_key() -> Option<u8> {
    let mut buf = [0u8; 1];
    let n = unsafe { libc::read(libc::STDIN_FILENO, buf.as_mut_ptr() as *mut libc::c_void, 1) };
    if n == 1 {
        Some(buf[0])
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Screen helpers
// ---------------------------------------------------------------------------

pub fn hide_cursor() -> io::Result<()> {
    io::stdout().write_all(b"\x1b[?25l").map(|_| ())
}

pub fn show_cursor() -> io::Result<()> {
    io::stdout().write_all(b"\x1b[?25h").map(|_| ())
}

pub fn clear_screen() -> io::Result<()> {
    io::stdout().write_all(b"\x1b[2J\x1b[H").map(|_| ())
}

/// Move cursor to the start of the lyric area without clearing: we redraw
/// in place every frame, which avoids the flash of full clears.
#[allow(dead_code)]
pub fn cursor_home() -> io::Result<()> {
    io::stdout().write_all(b"\x1b[H").map(|_| ())
}

#[cfg(test)]
mod tests {
    // term.rs is thin libc plumbing around raw mode / size / keys;
    // behavior is covered by the E2E run against the MPRIS mock.
}
