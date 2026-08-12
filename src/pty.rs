//! The PTY session: spawn a program in a pseudo-terminal, feed it input, and
//! capture its output for screen reconstruction.

use crate::screen::ScreenState;
use crate::{Result, TestError, DEFAULT_COLS, DEFAULT_ROWS};
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::time::{Duration, Instant};

/// A running program attached to a pseudo-terminal.
pub struct PtySession {
    master: std::fs::File,
    pid: libc::pid_t,
    out: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    rows: u16,
    cols: u16,
    exited: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl PtySession {
    /// Spawns `cmd` with `args` attached to a pty of the default size.
    pub fn spawn(cmd: &str, args: &[&str]) -> Result<PtySession> {
        PtySession::spawn_with_size(cmd, args, DEFAULT_ROWS, DEFAULT_COLS)
    }

    /// Spawns `cmd` with `args` attached to a pty of the given size.
    pub fn spawn_with_size(cmd: &str, args: &[&str], rows: u16, cols: u16) -> Result<PtySession> {
        let mut master: libc::c_int = 0;
        let mut ws = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            let pid = libc::forkpty(
                &mut master,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut ws,
            );
            if pid < 0 {
                return Err(TestError::Spawn(format!(
                    "forkpty failed: {}",
                    std::io::Error::last_os_error()
                )));
            }
            if pid == 0 {
                // Child: exec the command.
                let c_cmd = std::ffi::CString::new(cmd).unwrap();
                let mut c_args: Vec<std::ffi::CString> = vec![c_cmd.clone()];
                for a in args {
                    c_args.push(std::ffi::CString::new(*a).unwrap());
                }
                let mut argv: Vec<*const libc::c_char> =
                    c_args.iter().map(|a| a.as_ptr()).collect();
                argv.push(std::ptr::null());
                std::env::set_var("TERM", "xterm-256color");
                libc::execvp(c_cmd.as_ptr(), argv.as_mut_ptr());
                // If we get here, exec failed.
                let err = std::io::Error::last_os_error();
                let msg = format!("execvp failed: {err}\n");
                libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len());
                libc::_exit(127);
            }
            // Parent. The reader thread owns the original fd; the session
            // uses a dup for writing.
            let master_fd = master;
            let dup_fd = libc::dup(master_fd);
            if dup_fd < 0 {
                libc::close(master_fd);
                return Err(TestError::Spawn("dup failed".into()));
            }
            let master = std::fs::File::from_raw_fd(master_fd);
            let out = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let out2 = out.clone();
            let exited = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let exited2 = exited.clone();
            std::thread::spawn(move || {
                let mut master = master;
                let mut buf = [0u8; 8192];
                loop {
                    match master.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            out2.lock().unwrap().extend_from_slice(&buf[..n]);
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => {
                            eprintln!("DBG reader error: {e}");
                            break;
                        }
                    }
                }
                exited2.store(true, std::sync::atomic::Ordering::SeqCst);
            });

            Ok(PtySession {
                master: std::fs::File::from_raw_fd(dup_fd),
                pid,
                out,
                rows,
                cols,
                exited,
            })
        }
    }

    /// The captured raw output bytes so far.
    pub fn raw_output(&self) -> Vec<u8> {
        self.out.lock().unwrap().clone()
    }

    /// Reconstructs the current on-screen state by replaying the captured
    /// escape sequences.
    pub fn screen(&self) -> ScreenState {
        ScreenState::replay(&self.raw_output(), self.rows as usize, self.cols as usize)
    }

    /// Writes raw bytes to the pty (e.g. escape sequences).
    pub fn send(&self, bytes: &[u8]) -> Result<()> {
        let mut master = self.master.try_clone()?;
        master.write_all(bytes)?;
        master.flush()?;
        Ok(())
    }

    /// Types the given text literally (each character as a keypress).
    pub fn type_text(&self, text: &str) -> Result<()> {
        self.send(text.as_bytes())
    }

    /// Sends a named key ("enter", "tab", "esc", "up", "down", "left",
    /// "right", "backspace", "space", "q", "ctrl+c", ...). Multi-char literal
    /// keys (e.g. "q") are sent as-is; named keys map to their sequences.
    pub fn press(&self, key: &str) -> Result<()> {
        self.send(&crate::keys::key_sequence(key))
    }

    /// Resizes the terminal.
    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<()> {
        let ws = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let r = unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &ws) };
        if r < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        self.rows = rows;
        self.cols = cols;
        // Deliver SIGWINCH to the foreground process group.
        unsafe {
            libc::kill(-self.pid, libc::SIGWINCH);
        }
        Ok(())
    }

    /// Blocks until `pred` is true of the reconstructed screen, or the
    /// timeout elapses.
    pub fn wait_until(
        &self,
        timeout_ms: u64,
        pred: impl Fn(&ScreenState) -> bool,
    ) -> Result<ScreenState> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            if self.exited.load(std::sync::atomic::Ordering::SeqCst) {
                // Drain any final output.
            }
            let s = self.screen();
            if pred(&s) {
                return Ok(s);
            }
            if self.is_exited() {
                return Err(TestError::Exited(self.exit_status().unwrap_or(-1)));
            }
            if Instant::now() >= deadline {
                return Err(TestError::Timeout(format!(
                    "screen did not satisfy predicate; last screen:\n{}",
                    s
                )));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Waits until the given text appears anywhere on the screen.
    pub fn wait_for_text(&self, text: &str, timeout_ms: u64) -> Result<ScreenState> {
        self.wait_until(timeout_ms, |s| s.contains(text))
    }

    /// Waits until the given raw bytes appear in the captured output (for
    /// content that the screen reconstruction cannot represent, e.g.
    /// tab/backspace-based timer rendering).
    pub fn wait_for_raw(&self, text: &str, timeout_ms: u64) -> Result<()> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            if String::from_utf8_lossy(&self.raw_output()).contains(text) {
                return Ok(());
            }
            if self.is_exited() {
                return Err(TestError::Exited(self.exit_status().unwrap_or(-1)));
            }
            if Instant::now() >= deadline {
                return Err(TestError::Timeout(format!(
                    "raw output did not contain {text:?}"
                )));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Waits until the program exits, returning its exit status.
    pub fn wait_for_exit(&self, timeout_ms: u64) -> Result<i32> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        while Instant::now() < deadline {
            if self.is_exited() {
                return Ok(self.exit_status().unwrap_or(0));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Err(TestError::Timeout("program did not exit".into()))
    }

    /// Returns whether the child process has exited.
    pub fn is_exited(&self) -> bool {
        self.exited.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// The child's exit status (if it has exited).
    pub fn exit_status(&self) -> Option<i32> {
        let mut status = 0;
        let r = unsafe { libc::waitpid(self.pid, &mut status, libc::WNOHANG) };
        if r == self.pid {
            Some(libc::WEXITSTATUS(status))
        } else {
            None
        }
    }

    /// The pty master raw fd (for advanced use).
    pub fn master_fd(&self) -> RawFd {
        self.master.as_raw_fd()
    }

    /// Terminates the child (SIGKILL after a SIGTERM grace).
    pub fn kill(&self) {
        unsafe {
            libc::kill(self.pid, libc::SIGKILL);
        }
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.pid, libc::SIGKILL);
            libc::waitpid(self.pid, std::ptr::null_mut(), 0);
        }
    }
}

/// Reads one chunk from the master (blocking, small timeout).
pub fn read_chunk(master: &mut std::fs::File) -> std::io::Result<Vec<u8>> {
    let mut buf = [0u8; 8192];
    let n = master.read(&mut buf)?;
    Ok(buf[..n].to_vec())
}
