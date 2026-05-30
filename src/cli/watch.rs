//! Watch loop for list-style commands.
//!
//! Encapsulates the kubectl-style `-w` behaviour as a single helper so
//! individual commands stay declarative: provide an async closure that
//! produces the next render, the loop handles timing, screen-clearing,
//! and Ctrl+C teardown.

use crate::error::Result;
use std::future::Future;
use std::time::Duration;
use tokio::signal;
use tokio::time::interval;

/// Run `tick` repeatedly every `period`; clear the terminal between frames
/// when stdout is a TTY. Returns when Ctrl+C is received or `tick` errors.
pub async fn run_loop<F, Fut>(period: Duration, mut tick: F) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let is_tty = atty_stdout();
    let mut ticker = interval(period);
    // First tick fires immediately; we want explicit control.
    ticker.tick().await;

    loop {
        if is_tty {
            // ANSI: clear screen + move cursor home.
            print!("\x1b[2J\x1b[H");
        }
        tokio::select! {
            res = tick() => res?,
            _ = signal::ctrl_c() => return Ok(()),
        }
        tokio::select! {
            _ = ticker.tick() => {},
            _ = signal::ctrl_c() => return Ok(()),
        }
    }
}

/// Best-effort TTY detection without pulling another crate.
fn atty_stdout() -> bool {
    // SAFETY: isatty is a libc syscall that only reads the fd table.
    #[cfg(unix)]
    unsafe {
        extern "C" {
            fn isatty(fd: i32) -> i32;
        }
        isatty(1) != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}
