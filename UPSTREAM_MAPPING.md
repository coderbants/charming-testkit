# Source and platform mapping: `rusty-testkit`

`rusty-testkit` is an original Rust crate. It is not a clean-room port of an
upstream repository, so there are no upstream source files or examples to map.

## Platform boundary

| Rust file | Status | Contract |
| :--- | :--- | :--- |
| `src/pty.rs` | Unix PTY implementation | Uses `forkpty`, Unix file descriptors, signals, and wait APIs only under `cfg(unix)`. |
| `src/pty.rs` | Non-Unix compile-safe implementation | Preserves the public `PtySession` surface and returns a stable `Unsupported` error because native Windows PTY support is deferred. |
| `tests/smoke.rs` | Unix integration coverage | Exercises real PTY spawning, screen capture, input, resize, and exit behavior on Unix. |
| `tests/windows_platform.rs` | Windows regression coverage | Verifies deterministic unsupported-error behavior for the deferred native PTY operations. |

The Windows fallback is intentionally compile-safe for dependent crates such as
`rusty-bubbletea`. It does not claim to provide terminal emulation or process
control; those capabilities remain a future native Windows backend.
