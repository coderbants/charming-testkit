//! Reconstructs the on-screen state from a terminal's raw output by
//! replaying the cursor-movement and erase escape sequences (a terminal
//! emulator in miniature, covering the sequences the rusty examples
//! emit).

use std::collections::HashMap;

/// The reconstructed screen: a grid of characters plus the cursor position.
#[derive(Debug, Clone)]
pub struct ScreenState {
    rows: usize,
    cols: usize,
    grid: HashMap<(usize, usize), char>,
    pub cursor: (usize, usize),
}

/// The CSI final characters we interpret (plus `d`/`G`/`p` for the moves).
const FINALS: &str = "ABCDGHJKlmnPrshtfudpX";

impl ScreenState {
    /// Replays the raw output bytes into a screen state of the given size.
    pub fn replay(raw: &[u8], rows: usize, cols: usize) -> ScreenState {
        let s = String::from_utf8_lossy(raw);
        let mut st = ScreenState {
            rows,
            cols,
            grid: HashMap::new(),
            cursor: (0, 0),
        };
        let bytes: Vec<char> = s.chars().collect();
        let n = bytes.len();
        let mut i = 0;
        let mut x: usize = 0;
        let mut y: usize = 0;
        while i < n {
            let c = bytes[i];
            if c == '\u{1b}' {
                if i + 1 < n && bytes[i + 1] == '[' {
                    let mut j = i + 2;
                    while j < n && !FINALS.contains(bytes[j]) {
                        j += 1;
                    }
                    if j >= n {
                        break;
                    }
                    let seq: String = bytes[i + 2..j].iter().collect();
                    let final_char = bytes[j];
                    let params: Vec<&str> = seq.split(';').collect();
                    let pv = |k: usize| -> usize {
                        let v = params.get(k).copied().unwrap_or("1");
                        if v.is_empty() {
                            1
                        } else {
                            v.parse().unwrap_or(1)
                        }
                    };
                    let has_private = seq.contains('?') || seq.contains('$');
                    let numeric =
                        seq.is_empty() || seq.replace(';', "").chars().all(|c| c.is_ascii_digit());
                    if has_private || !numeric {
                        // Ignore private/query sequences and unknown params.
                    } else {
                        match final_char {
                            'H' => {
                                y = pv(0).saturating_sub(1).min(rows.saturating_sub(1));
                                x = if params.len() > 1 {
                                    pv(1).saturating_sub(1).min(cols.saturating_sub(1))
                                } else {
                                    0
                                };
                            }
                            'A' => y = y.saturating_sub(pv(0)),
                            'B' => y = (y + pv(0)).min(rows.saturating_sub(1)),
                            'C' => x = (x + pv(0)).min(cols.saturating_sub(1)),
                            'D' => x = x.saturating_sub(pv(0)),
                            'd' => {
                                y = pv(0).saturating_sub(1).min(rows.saturating_sub(1));
                            }
                            'G' => {
                                x = pv(0).saturating_sub(1).min(cols.saturating_sub(1));
                            }
                            'J' => {
                                let p = if params.is_empty() || params[0].is_empty() {
                                    0
                                } else {
                                    pv(0)
                                };
                                if p == 2 {
                                    st.grid.clear();
                                } else if p == 1 {
                                    // Erase above.
                                    for yy in 0..=y {
                                        for xx in 0..cols {
                                            st.grid.remove(&(xx, yy));
                                        }
                                    }
                                } else {
                                    // Erase below.
                                    for yy in y..rows {
                                        for xx in x..cols {
                                            st.grid.remove(&(xx, yy));
                                        }
                                    }
                                }
                            }
                            'K' => {
                                // Erase in line: 0/absent = right, 1 = left,
                                // 2 = entire line. The EL parameter is the
                                // only one whose omitted default is 0.
                                let p = if params.is_empty() || params[0].is_empty() {
                                    0
                                } else {
                                    pv(0)
                                };
                                let start = if p == 1 { 0 } else { x };
                                let end = if p == 1 { x + 1 } else { cols };
                                for xx in start..end.min(cols) {
                                    st.grid.remove(&(xx, y));
                                }
                            }
                            'L' => {
                                // Insert line: shift rows down from y.
                                for yy in (y + 1..rows).rev() {
                                    for xx in 0..cols {
                                        if let Some(v) = st.grid.remove(&(xx, yy - 1)) {
                                            st.grid.insert((xx, yy), v);
                                        }
                                    }
                                }
                                for xx in 0..cols {
                                    st.grid.remove(&(xx, y));
                                }
                            }
                            'M' => {
                                // Delete line: shift rows up.
                                for yy in y..rows.saturating_sub(1) {
                                    for xx in 0..cols {
                                        if let Some(v) = st.grid.remove(&(xx, yy + 1)) {
                                            st.grid.insert((xx, yy), v);
                                        }
                                    }
                                }
                                for xx in 0..cols {
                                    st.grid.remove(&(rows - 1, xx));
                                }
                            }
                            'P' => {
                                // Delete character: shift the rest left.
                                for xx in x..cols.saturating_sub(1) {
                                    if let Some(v) = st.grid.remove(&(xx + 1, y)) {
                                        st.grid.insert((xx, y), v);
                                    }
                                }
                                st.grid.remove(&(cols - 1, y));
                            }
                            'X' => {
                                // Erase characters (ECH).
                                for xx in x..(x + pv(0)).min(cols) {
                                    st.grid.remove(&(xx, y));
                                }
                            }
                            _ => {}
                        }
                    }
                    i = j + 1;
                } else if i + 1 < n && bytes[i + 1] == ']' {
                    // OSC: skip to the ST (BEL or ESC \).
                    let mut j = i + 2;
                    while j < n && bytes[j] != '\u{7}' {
                        if bytes[j] == '\u{1b}' && j + 1 < n && bytes[j + 1] == '\\' {
                            j += 2;
                            break;
                        }
                        j += 1;
                    }
                    if j < n && bytes[j] == '\u{7}' {
                        j += 1;
                    }
                    i = j;
                } else {
                    i += 1;
                }
                continue;
            } else if c == '\n' {
                y = (y + 1).min(rows.saturating_sub(1));
            } else if c == '\r' {
                x = 0;
            } else if c == '\u{8}' {
                // Backspace: cursor left.
                x = x.saturating_sub(1);
            } else if c == '\u{7}' {
                // Bell: ignore.
            } else {
                if x < cols && y < rows {
                    st.grid.insert((x, y), c);
                }
                x = (x + 1).min(cols);
            }
            i += 1;
        }
        st.cursor = (x.min(cols.saturating_sub(1)), y);
        st
    }

    /// The character at the given 0-based cell (space if empty).
    pub fn cell(&self, x: usize, y: usize) -> char {
        if x >= self.cols || y >= self.rows {
            return ' ';
        }
        self.grid.get(&(x, y)).copied().unwrap_or(' ')
    }

    /// The full text of a row (0-based).
    pub fn line(&self, y: usize) -> String {
        (0..self.cols).map(|x| self.cell(x, y)).collect()
    }

    /// Whether the given text appears anywhere on the screen.
    pub fn contains(&self, text: &str) -> bool {
        self.to_string().contains(text)
    }

    /// Whether the given text appears starting at the given 0-based cell.
    pub fn contains_at(&self, x: usize, y: usize, text: &str) -> bool {
        if y >= self.rows || x + text.chars().count() > self.cols {
            return false;
        }
        text.chars()
            .enumerate()
            .all(|(i, c)| self.cell(x + i, y) == c)
    }

    /// The row and column of the first occurrence of `text`, if any.
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        for y in 0..self.rows {
            let line = self.line(y);
            if let Some(x) = line.find(text) {
                return Some((x, y));
            }
        }
        None
    }

    /// The number of rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// The number of columns.
    pub fn cols(&self) -> usize {
        self.cols
    }
}

impl std::fmt::Display for ScreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = (0..self.rows)
            .map(|y| self.line(y).trim_end().to_string())
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{s}")
    }
}
