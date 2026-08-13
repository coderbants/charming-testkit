//! Cleanroom test harness for the `rusty-*` TUI crates.
//!
//! <public-docs>
//! A Playwright-style integration harness for terminal programs: spawn an
//! example binary in a pseudo-terminal, send keys and mouse events, resize
//! the terminal, and assert on the reconstructed on-screen state.
//!
//! ```no_run
//! use rusty_testkit::PtySession;
//!
//! let mut pty = PtySession::spawn("target/debug/examples/textinput", &[])?;
//! pty.wait_for_text("Type something", 5000)?;
//! pty.type_text("hello");
//! pty.wait_until(5000, |s| s.contains("hello"))?;
//! pty.press("enter");
//! pty.press("q");
//! pty.wait_for_exit(3000)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//! </public-docs>

pub mod keys;
pub mod mouse;
pub mod pty;
pub mod screen;

pub use pty::PtySession;
pub use screen::ScreenState;

/// The default terminal size used when spawning examples.
pub const DEFAULT_ROWS: u16 = 24;
/// The default terminal size used when spawning examples.
pub const DEFAULT_COLS: u16 = 80;

/// A convenience error type for the harness.
#[derive(Debug)]
pub enum TestError {
    /// The program did not satisfy a predicate within the timeout.
    Timeout(String),
    /// The program exited unexpectedly.
    Exited(i32),
    /// An I/O error occurred.
    Io(std::io::Error),
    /// The spawn failed.
    Spawn(String),
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::Timeout(msg) => write!(f, "timeout: {msg}"),
            TestError::Exited(code) => write!(f, "program exited with status {code}"),
            TestError::Io(e) => write!(f, "io error: {e}"),
            TestError::Spawn(msg) => write!(f, "spawn failed: {msg}"),
        }
    }
}

impl std::error::Error for TestError {}

impl From<std::io::Error> for TestError {
    fn from(e: std::io::Error) -> Self {
        TestError::Io(e)
    }
}

impl From<String> for TestError {
    fn from(msg: String) -> Self {
        TestError::Spawn(msg)
    }
}

/// Result alias for the harness.
pub type Result<T> = std::result::Result<T, TestError>;
