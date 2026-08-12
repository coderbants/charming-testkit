use charming_testkit::PtySession;

#[test]
fn spawn_and_capture() {
    let pty = PtySession::spawn("sh", &["-c", "echo hello; sleep 1"]).expect("spawn");
    let s = pty.wait_for_text("hello", 5000).expect("text");
    assert!(s.contains("hello"));
    pty.kill();
}

#[test]
fn screen_grid() {
    let pty = PtySession::spawn("printf", &["AB\\nCD"]).expect("spawn");
    std::thread::sleep(std::time::Duration::from_millis(300));
    let s = pty.screen();
    assert_eq!(s.cell(0, 0), 'A');
    assert_eq!(s.cell(1, 0), 'B');
    assert_eq!(s.cell(0, 1), 'C');
    assert_eq!(s.cell(1, 1), 'D');
    assert!(s.contains_at(0, 0, "AB"));
    pty.kill();
}

#[test]
fn type_and_echo() {
    let pty = PtySession::spawn("sh", &["-c", "read x; echo got:$x; sleep 1"]).expect("spawn");
    std::thread::sleep(std::time::Duration::from_millis(200));
    pty.type_text("hello").expect("type");
    pty.press("enter").expect("enter");
    pty.wait_for_text("got:hello", 5000).expect("text");
    pty.kill();
}
