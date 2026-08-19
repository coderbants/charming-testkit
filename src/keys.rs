//! Named key sequences and mouse event encodings for driving terminal
//! programs.

/// Returns the byte sequence for a named key. Literal single characters are
/// passed through; control and navigation keys map to their terminal
/// sequences.
pub fn key_sequence(key: &str) -> Vec<u8> {
    match key {
        "enter" => b"\r".to_vec(),
        "tab" => b"\t".to_vec(),
        "backspace" => b"\x7f".to_vec(),
        "space" => b" ".to_vec(),
        "esc" => b"\x1b".to_vec(),
        "up" => b"\x1b[A".to_vec(),
        "down" => b"\x1b[B".to_vec(),
        "right" => b"\x1b[C".to_vec(),
        "left" => b"\x1b[D".to_vec(),
        "home" => b"\x1b[H".to_vec(),
        "end" => b"\x1b[F".to_vec(),
        "pgup" => b"\x1b[5~".to_vec(),
        "pgdown" => b"\x1b[6~".to_vec(),
        "delete" => b"\x1b[3~".to_vec(),
        "insert" => b"\x1b[2~".to_vec(),
        "ctrl+a" => b"\x01".to_vec(),
        "ctrl+b" => b"\x02".to_vec(),
        "ctrl+c" => b"\x03".to_vec(),
        "ctrl+d" => b"\x04".to_vec(),
        "ctrl+e" => b"\x05".to_vec(),
        "ctrl+f" => b"\x06".to_vec(),
        "ctrl+g" => b"\x07".to_vec(),
        "ctrl+h" => b"\x08".to_vec(),
        "ctrl+i" => b"\x09".to_vec(),
        "ctrl+j" => b"\x0a".to_vec(),
        "ctrl+k" => b"\x0b".to_vec(),
        "ctrl+l" => b"\x0c".to_vec(),
        "ctrl+m" => b"\x0d".to_vec(),
        "ctrl+n" => b"\x0e".to_vec(),
        "ctrl+o" => b"\x0f".to_vec(),
        "ctrl+p" => b"\x10".to_vec(),
        "ctrl+q" => b"\x11".to_vec(),
        "ctrl+r" => b"\x12".to_vec(),
        "ctrl+s" => b"\x13".to_vec(),
        "ctrl+t" => b"\x14".to_vec(),
        "ctrl+u" => b"\x15".to_vec(),
        "ctrl+v" => b"\x16".to_vec(),
        "ctrl+w" => b"\x17".to_vec(),
        "ctrl+x" => b"\x18".to_vec(),
        "ctrl+y" => b"\x19".to_vec(),
        "ctrl+z" => b"\x1a".to_vec(),
        _ => key.as_bytes().to_vec(),
    }
}

/// SGR mouse button codes: 0=left, 1=middle, 2=right, 3=release.
pub const MOUSE_LEFT: u8 = 0;
/// SGR mouse button codes: 0=left, 1=middle, 2=right, 3=release.
pub const MOUSE_MIDDLE: u8 = 1;
/// SGR mouse button codes: 0=left, 1=middle, 2=right, 3=release.
pub const MOUSE_RIGHT: u8 = 2;
/// SGR mouse button codes: 0=left, 1=middle, 2=right, 3=release.
pub const MOUSE_RELEASE: u8 = 3;
/// SGR motion (no button) — used in mode 1003.
pub const MOUSE_MOTION: u8 = 35;
/// SGR wheel up.
pub const MOUSE_WHEEL_UP: u8 = 64;
/// SGR wheel down.
pub const MOUSE_WHEEL_DOWN: u8 = 65;

/// A mouse event (SGR encoding, 1-based cells).
pub struct MouseEvent {
    /// The SGR button/modifier code (see the constants above; add 32 to the
    /// button for presses, 64 for wheels).
    pub button: u8,
    /// 1-based column.
    pub x: u16,
    /// 1-based row.
    pub y: u16,
    /// Whether this is a press (uppercase M) or release (lowercase m).
    pub press: bool,
}

/// Encodes a mouse event in the SGR protocol: `ESC [ < Cb ; Cx ; Cy M/m`.
pub fn mouse_sequence(ev: &MouseEvent) -> Vec<u8> {
    let final_char = if ev.press { b'M' } else { b'm' };
    format!(
        "\x1b[<{};{};{}{}",
        ev.button, ev.x, ev.y, final_char as char
    )
    .into_bytes()
}

/// A left-button press at the given 1-based cell.
pub fn mouse_click(x: u16, y: u16) -> Vec<u8> {
    mouse_sequence(&MouseEvent {
        button: MOUSE_LEFT,
        x,
        y,
        press: true,
    })
}

/// A right-button press at the given 1-based cell.
pub fn mouse_right_click(x: u16, y: u16) -> Vec<u8> {
    mouse_sequence(&MouseEvent {
        button: MOUSE_RIGHT,
        x,
        y,
        press: true,
    })
}

/// A button release at the given 1-based cell.
pub fn mouse_release(x: u16, y: u16) -> Vec<u8> {
    mouse_sequence(&MouseEvent {
        button: MOUSE_RELEASE,
        x,
        y,
        press: false,
    })
}

/// A motion event (no button, mode 1003) at the given 1-based cell.
pub fn mouse_motion(x: u16, y: u16) -> Vec<u8> {
    mouse_sequence(&MouseEvent {
        button: MOUSE_MOTION,
        x,
        y,
        press: true,
    })
}

/// A drag event (motion while a button is held) at the given 1-based cell.
pub fn mouse_drag(button: u8, x: u16, y: u16) -> Vec<u8> {
    mouse_sequence(&MouseEvent {
        button: 32 + button,
        x,
        y,
        press: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_key_sequences() {
        let keys = [
            ("enter", b"\r".to_vec()),
            ("tab", b"\t".to_vec()),
            ("backspace", b"\x7f".to_vec()),
            ("space", b" ".to_vec()),
            ("esc", b"\x1b".to_vec()),
            ("up", b"\x1b[A".to_vec()),
            ("down", b"\x1b[B".to_vec()),
            ("right", b"\x1b[C".to_vec()),
            ("left", b"\x1b[D".to_vec()),
            ("home", b"\x1b[H".to_vec()),
            ("end", b"\x1b[F".to_vec()),
            ("pgup", b"\x1b[5~".to_vec()),
            ("pgdown", b"\x1b[6~".to_vec()),
            ("delete", b"\x1b[3~".to_vec()),
            ("insert", b"\x1b[2~".to_vec()),
            ("ctrl+a", b"\x01".to_vec()),
            ("ctrl+b", b"\x02".to_vec()),
            ("ctrl+c", b"\x03".to_vec()),
            ("ctrl+d", b"\x04".to_vec()),
            ("ctrl+e", b"\x05".to_vec()),
            ("ctrl+f", b"\x06".to_vec()),
            ("ctrl+g", b"\x07".to_vec()),
            ("ctrl+h", b"\x08".to_vec()),
            ("ctrl+i", b"\x09".to_vec()),
            ("ctrl+j", b"\x0a".to_vec()),
            ("ctrl+k", b"\x0b".to_vec()),
            ("ctrl+l", b"\x0c".to_vec()),
            ("ctrl+m", b"\x0d".to_vec()),
            ("ctrl+n", b"\x0e".to_vec()),
            ("ctrl+o", b"\x0f".to_vec()),
            ("ctrl+p", b"\x10".to_vec()),
            ("ctrl+q", b"\x11".to_vec()),
            ("ctrl+r", b"\x12".to_vec()),
            ("ctrl+s", b"\x13".to_vec()),
            ("ctrl+t", b"\x14".to_vec()),
            ("ctrl+u", b"\x15".to_vec()),
            ("ctrl+v", b"\x16".to_vec()),
            ("ctrl+w", b"\x17".to_vec()),
            ("ctrl+x", b"\x18".to_vec()),
            ("ctrl+y", b"\x19".to_vec()),
            ("ctrl+z", b"\x1a".to_vec()),
            ("abc", b"abc".to_vec()),
        ];
        for (name, expected) in keys {
            assert_eq!(key_sequence(name), expected);
        }
    }

    #[test]
    fn test_mouse_sequences() {
        assert_eq!(mouse_click(5, 10), b"\x1b[<0;5;10M");
        assert_eq!(mouse_right_click(5, 10), b"\x1b[<2;5;10M");
        assert_eq!(mouse_release(5, 10), b"\x1b[<3;5;10m");
        assert_eq!(mouse_motion(5, 10), b"\x1b[<35;5;10M");
        assert_eq!(mouse_drag(MOUSE_LEFT, 5, 10), b"\x1b[<32;5;10M");
    }
}
