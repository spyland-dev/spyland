<div align="center">

**English** | [Русский](../ru/README.md)

# spyland

**Screen time tracking for Wayland**

</div>

`spyland` is a project for tracking screen time on Wayland compositors, written in Rust.
The project includes a Unix daemon, CLI application, and a public API library.

## Features

- **Local storage**: All data is stored locally at `$XDG_STATE_HOME/spyland/sessions.sqlite`
- **Minimal resource usage**: < 1% CPU, ~8 MB RAM (spylandd)
- **Easily extensible**: Backend-based architecture allows adding support for new compositors
- **Written in Rust**: Memory safety and high performance
- **Well-tested**: All code is covered by automated tests

## Installation

### From Source

Requres Rust (latest stable version). Install via `cargo`:

```bash
git clone https://github.com/NonExistPlayer/spyland
cd spyland
cargo install --path ./daemon
cargo install --path ./cli
```

## Project Structure

| Directory | Crate                  | Description                                        |
| --------- | :--------------------: | -------------------------------------------------- |
| `core`    | `spyland-core`         | Project core: session abstraction, events, backend |
| `lib`     | `spyland-lib`          | Public library: database API, IPC, utilities       |
| `daemon`  | `spylandd`             | Unix daemon that tracks screen time in background  |
| `cli`     | `spyland`              | CLI application for interacting with daemon        |
| `niri`    | `spyland-backend-niri` | Backend for niri compositor                        |
| `docs`    | -                      | Project documentation                              |

## Roadmap

- [ ] **Activities**: Grouping sessions (work, entertainment, study)
- [ ] **Installation**
  - [ ] Publish crates on [crates.io](https://crates.io). Including binaries.
  - [ ] Package Managers
    - [ ] AUR
      - [ ] `spyland-git`
      - [ ] `spyland-bin`
  - [ ] System services for the daemon
    - [ ] systemd
- [ ] **New backends**
  - [ ] Hyprland
  - [ ] KDE
  - [ ] Sway
  - [ ] *Mutter?*
- [ ] **Runtime backend loading**: Dynamic backend loading without recompilation
- [ ] **Database encryption**: Protect user data
- [ ] **Gtk application**
- [ ] **Database integrity checks**: Validate data on load
- [ ] **Extended OS support**
- [ ] **Activities**: Grouping sessions (work, entertainment, study)
  - [ ] Windows
  - [ ] Android
    - [ ] Backend
    - [ ] Application

## License

This project is licensed under GNU GPL v3.0. See [LICENSE](../../LICENSE) for details.
