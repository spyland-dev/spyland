<div align="center">

**English** | [Русский](/docs/ru/CONTRIBUTING.md)

</div>

Before creating a Pull Request to this project, please read this guide.

Thank you for your interest in `spyland`!

# Development

## Environment

Before you begin, you need to prepare the development environment.

The easiest way is to use the provided Nix flake:
```
nix develop .
```
With this, you will get all the necessary packages and environment variables. No additional
configuration is required.

Alternatively, you can set up everything manually:

1. **Dependencies**: In addition to Rust and Cargo, for comfortable development you will need:
   - `clippy`: The linter integrated with CI and required for tests.
   - `cargo-nextest`: A more convenient and less verbose test runner (optional).
   - `just`: To run the necessary checks in a single command.
   - `sqlite3` and `sqlx-cli`: For working with the database in `spyland-lib`.
2. **Environment Variables**: For manual testing of the binary files, you will need:
   - `SPYLAND_DATABASE`: Overrides the spyland database file path; use it to avoid conflicts.
   - `SPYLAND_SOCKET`: Overrides the socket path for `spylandd`; use it to avoid conflicts.
   - `SPYLAND_CONFIG`: Overrides the spyland config path; use it to avoid conflicts.
   - `SQLX_OFFLINE`: Set to `true` to use cached database query results.
   - `DATABASE_URL`: Specify the database path in the format: `sqlite:///path/to/it.sqlite`.
   - `RUST_LOG`: Set to `debug`/`trace` for more detailed logging from the daemon and backends.

You do not need to set the `SPYLAND_` variables unless you are using or plan to use spyland on a
regular basis. These variables are needed to avoid conflicts with a potentially running spyland
instance on your computer. Choose any path that is convenient for you.

## Checks

Our CI has several checks:
- Formatting
- Build
- Tests + Doc-tests
- Clippy

You can run these checks in any way you want, as long as each of them matches the CI results.
Note that CI rejects all Clippy warnings.

> [!IMPORTANT]
> The `sqlx` library used in `spyland-lib` (with `db` feature enabled) adds compile-time
> verification for SQL queries. If you modify or add queries, you must update the `sqlx` offline
> cache for CI verification. To update the cache, go to `crates/lib` and run: `cargo sqlx prepare`
> (you must have `sqlx-cli` installed). Do not forget to commit the `.sqlx` directory containing
> the cache!

To run checks, go to the directory of a specific crate (to check only that crate) or to the
repository root (to check all crates) and run the required commands:

### Manually

"Manually" means running each check command by command:

1. Formatting: `cargo fmt --check`
2. Build: `cargo build`
3. Tests: `cargo test` or `cargo nextest run` + `cargo test --doc`
4. Clippy: `cargo clippy --all-targets --all-features`

> [!WARNING]
> The commands may differ from CI and `justfile` for brevity and ease of use.

### Using Just

There is a `justfile` in the repository root. You can run all the necessary checks with a single command:
```
just check
```
Each check is split into its own recipe, which you can run using `just <RECIPE>`
for example:`just fmt`, `just test`, `just doc`.

> [!TIP]
> For quick checks (formatting, build, and Clippy) and testing binaries, just use `just`. For a
> complete verification (mandatory before commit/push), use `just check`.

## Running and Installing

To run a binary crate, use `cargo run -p spyland` (for the CLI) and
`cargo run -p spylandd` (for the Unix daemon). For local installation, use
`cargo install --path crates/cli` (for the CLI) and
`cargo install --path crates/daemon` (for the Unix daemon).

# Code

## Code Style

Use [`rustfmt`](https://github.com/rust-lang/rustfmt) for code formatting.
You can run it directly via `cargo fmt` or through your LSP.

### Imports (`use`)

This project has no strict rules for import formatting.
You will encounter different styles:

**Verbose style:**
```rust
use spyland_core::Clock;
use spyland_core::Configuration;
use spyland_core::Event;
use spyland_core::SessionManager;
use std::cell::RefCell;
use std::rc::Rc;
```

**Structured style:**
```rust
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, Mutex, mpsc::{self, Receiver, Sender}},
    thread,
    time::Duration,
};
```

**Mixed style:**
```rust
use log::{error, warn};
use niri_ipc::socket::Socket;
use niri_ipc::{Event as NiriEvent, Request, Response};
use spyland_core::{Backend, Event};
use std::path::PathBuf;
use std::sync::mpsc;
```

Format `use` imports however you find convenient — as long as `rustfmt` is satisfied.

## Tests

**This is critical!** `spyland` is developed with the requirement
that all new code must be tested.

Every new feature requires tests.
Just like bug fixes: if you fix a bug, please write a test for it.

If you think they are not needed in your case, open an issue or include a justification in your PR.

### Constants

By looking at the test code, you may find that constants are used frequently.
For test code, we have a rule: if you use the same value compile time multiple times
(e.g. for data validation), then declare constant inside the function and use it
instead of the same values. This is done for the correctness of the data and to
indicate the relationship.

**Incorrect**, not using constants — "magic" values:
```rust
d.event(Event::WorkspaceChanged(2));
d.event(Event::ActiveWindowChanged(Some("discord".into())));
d.flush();

match &d.mgr.sessions()[0].state {
    State::Active { app_id, workspace } => {
        assert_eq!("discord", app_id, "app_id not matching");
        assert_eq!(
            2,
            workspace.expect("workspace is none"),
            "workspace not matching"
        );
    }
    _ => panic!("Incorrect state"),
}
```
**Incorrect**, only used once — no relationship:
```rust
const APP_ID: &str = "firefox";

d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
d.flush();

assert_eq!(d.mgr.sessions().len(), 1, "Less than one session");
```
**Correct**, constants are used, showing relationship:
```rust
const WORKSPACE: i32 = 1;
const APP_ID: &str = "firefox";

d.event(Event::WorkspaceChanged(WORKSPACE));
d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
d.flush();

match &d.mgr.sessions()[0].state {
    State::Active { app_id, workspace } => {
        assert_eq!(APP_ID, app_id, "app_id not matching");
        assert_eq!(
            WORKSPACE,
            workspace.expect("workspace is none"),
            "workspace not matching"
        );
    }
    _ => panic!("Incorrect state"),
}
```

## Artificial Intelligence

Using AI is allowed, but under your strict control. You must:
- Fully understand the generated code
- Verify that the code works correctly
- Take responsibility for code quality

If we find that code was clearly generated by AI
without understanding — your PR may be rejected.

# Commits

Commit names must follow
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
type(scope): description
```

# Documentation

## Comments in code (`//`)

Comments should be meaningful, not obvious.

Correct example:
```rust
let mut d = TestDriver::new();

d.event(Event::ActiveWindowChanged(Some("firefox".into())));
d.tick(30);
// d.update_and_flush();
// not needed because of automatic update()
// and update() in SessionManager
```

Incorrect example:
```rust
let mut d = TestDriver::new();

d.event(Event::ActiveWindowChanged(Some("firefox".into())));
d.update_and_flush(); // explicit flushes
```

For important comments, use prefixes:
```rust
// TODO: optimize this loop
// FIXME: handle edge case when buffer is empty
// NOTE: this must run before db initialization
// WARN: never call this from async context
```

## API Documentation (`///`, `//!`)

Document the public API of `spyland-lib` and `spyland-core` crates.

Requirements:
1. Brief description
2. More detailed description (if necessary)
3. Function parameters in section `# Arguments` (if it's a function)
4. Usage examples in section `# Example`
5. Warnings in sections `# Panics` or `# Safety` (if applicable)
6. Doctests to verify examples

## Markdown Documentation (`docs/`)

Files in `docs/` must exist in two languages:
- `docs/en/` — English version
- `docs/ru/` — Russian version

Symbolic links to the English version are in the repository root.

When modifying documentation:
1. Update the main English version
2. Verify all links are correct (internal, external, images)
3. Ensure translation is accurate
4. Check formatting (headings, code, lists)

---

Questions?

1. Check existing code in the repository — it serves as the best example
2. Open an issue if something is unclear
3. Discuss in your PR before starting work

Thank you for helping develop `spyland`!
