#![cfg(windows)]

use rusty_testkit::{PtySession, TestError};

#[test]
fn spawn_reports_deferred_windows_pty_support() {
    let error = match PtySession::spawn("cmd.exe", &[]) {
        Ok(_) => panic!("Windows PTY spawning must remain explicitly unsupported"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        TestError::Unsupported(message)
            if message.contains("native Windows PTY support is deferred")
    ));
}

#[test]
fn sized_spawn_reports_the_same_stable_error() {
    let error = match PtySession::spawn_with_size("cmd.exe", &[], 24, 80) {
        Ok(_) => panic!("Windows PTY spawning must remain explicitly unsupported"),
        Err(error) => error,
    };

    assert_eq!(
        error.to_string(),
        "unsupported: PtySession requires a Unix pseudo-terminal; native Windows PTY support is deferred"
    );
}
