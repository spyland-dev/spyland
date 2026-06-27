<div align="center">

**English** | [Русский](/docs/ru/README.md)

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

Requires Rust (latest stable version). Install via `cargo`:

```bash
git clone https://github.com/NonExistPlayer/spyland
cd spyland
cargo install --path ./daemon
cargo install --path ./cli
```

## Project Structure

```
.
├── crates                            # Crates or source code
│   ├── cli                           # CLI program for interacting with data
│   ├── core                          # Project core: event and session abstraction
│   ├── daemon                        # Unix-daemon, which tracks time
│   ├── lib                           # Public API library: Database API, IPC, utils
│   └── niri                          # Backend for Wayland-compositor 'niri'
│
├── docs                              # Project documentation
│   ├── en                            # English
│   └── ru                            # Russian
│
├── res                               # Additional files (like services)
│   ├── spyland-backend-niri.service  # Systemd service for the niri backend
│   ├── spyland-backends.target       # Systemd target for backends
│   └── spylandd.service              # Systemd service for the daemon
│
├── Cargo.lock
├── Cargo.toml                        # Cargo workspace
├── CONTRIBUTING.md -> docs/en/CONTRIBUTING.md
├── flake.lock
├── flake.nix                         # Nix flake with dev-shell
├── LICENSE                           # Code license
└── README.md -> docs/en/README.md
```

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
- [ ] **Database encryption**: Protect user data
- [ ] **Gtk application**
- [ ] **Database integrity checks**: Validate data on load
- [ ] **Extended OS support**
  - [ ] Windows
  - [ ] Android
    - [ ] Backend
    - [ ] Application

- [x] ~~**Runtime backend loading**: Dynamic backend loading without recompilation.~~ [#1](https://github.com/NonExistPlayer/spyland/pull/1)

## License

This project is licensed under GNU GPL v3.0. See [LICENSE](../../LICENSE) for details.
