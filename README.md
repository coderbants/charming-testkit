<p align="center">
    <a href="https://raw.githubusercontent.com/coderbants/rusty-testkit/dev/rusty-testkit.png"><img src="https://raw.githubusercontent.com/coderbants/rusty-testkit/dev/rusty-testkit.png" width="313" alt="Rusty Testkit"></a><br>
    <a href="https://crates.io/crates/rusty-testkit"><img src="https://img.shields.io/crates/v/rusty-testkit.svg" alt="crates.io"></a>
    <a href="https://github.com/coderbants/rusty-testkit/actions"><img src="https://github.com/coderbants/rusty-testkit/actions/workflows/ci.yml/badge.svg" alt="Build Status"></a>
    <a href="https://raw.githubusercontent.com/coderbants/rusty-testkit/dev/coverage.svg"><img src="https://raw.githubusercontent.com/coderbants/rusty-testkit/dev/coverage.svg" alt="coverage"></a>

</p>

# Rusty Testkit (`rusty-testkit`)

**Rusty Testkit** is a small support crate for the [Rusty](https://github.com/coderbants) port family: helpers for driving and asserting against terminal applications under test — PTY sessions, key/mouse sequence builders, and screen text predicates. It backs the interactive integration tests in the other Rusty crates.

It's part of the Rusty port family of the Bubble Tea ecosystem and is used by [rusty-bubbletea](https://github.com/coderbants/rusty-bubbletea), [rusty-bubbles](https://github.com/coderbants/rusty-bubbles), [rusty-lipgloss](https://github.com/coderbants/rusty-lipgloss) and [rusty-ultraviolet](https://github.com/coderbants/rusty-ultraviolet).

## Installation

Add it as a dev-dependency:

```sh
cargo add --dev rusty-testkit
```

## License

MIT
