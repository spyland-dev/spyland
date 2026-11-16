<div align="center">

**English** | [Русский](/docs/ru/README.md)

# spyland

**Screen time tracking for Wayland**

</div>

`spyland` is a Rust project for tracking screen time on Wayland compositors.
It includes a Unix daemon, CLI application, and a library with a public API.

## Features

- **Local storage**: All data is stored locally at `$XDG_STATE_HOME/spyland/sessions.sqlite`
- **Minimal resource usage**: < 1% CPU, ~8 MB RAM (spylandd)
- **Easily extensible**: Backend-based architecture allows adding support for new compositors
- **Written in Rust**: Memory safety and high performance
- **Well-tested**: The project includes automated tests

## Installation

### From Source

Requires Rust (latest stable version). Install via `cargo`:

```bash
git clone https://github.com/spyland-dev/spyland
cd spyland
cargo install --path ./crates/daemon
cargo install --path ./crates/cli
```

### Nix / NixOS

The project flake exports `packages`, `apps`, and `nixosModules`. To try the
CLI or daemon without installing it, run:

```bash
nix run github:spyland-dev/spyland#cli
nix run github:spyland-dev/spyland#daemon
```

For an imperative installation (instead of declaratively configuring Spyland
with the NixOS module), use:

```bash
nix profile add github:spyland-dev/spyland#cli
nix profile add github:spyland-dev/spyland#daemon
```

On NixOS, add the flake to your `inputs` and include its module in your
configuration's `modules` list:

```nix
{
  inputs.spyland.url = "github:spyland-dev/spyland";

  outputs = inputs@{ nixpkgs, ... }: {
    nixosConfigurations.default = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        inputs.spyland.nixosModules.default
        ./configuration.nix
      ];
    };
  };
}
```

Enable the CLI and daemon in your NixOS configuration. If you use the niri
compositor, enable its backend as well:

```nix
{
  programs.spyland.enable = true;
  services.spylandd.enable = true;
  services.spylandd.backends.niri.enable = true; # only for the niri compositor
}
```

## Project Structure

```
.
├── crates                                # Crates containing source code
│   ├── cli                               # CLI program for interacting with data
│   ├── core                              # Project core: event and session abstraction
│   ├── daemon                            # Unix-daemon, which tracks time
│   ├── lib                               # Public API library: database API, IPC, utilities
│   └── niri                              # Backend for the niri Wayland compositor
│
├── docs                                  # Project documentation
│   ├── en                                # English
│   └── ru                                # Russian
│
├── res                                   # Additional files
│   ├── nix                               # Nix configuration files
│   │   ├── crane.nix                     # Package builds, checks, and apps
│   │   ├── dev-shell.nix                 # Development shell
│   │   ├── module.nix                    # NixOS module
│   │   └── outputs.nix                   # Flake outputs
│   └── services                          # Systemd services
│       ├── spyland-backend-niri.service  # Systemd service for the niri backend
│       ├── spyland-backends.target       # Systemd target for backends
│       └── spylandd.service              # Systemd service for the daemon
│
├── Cargo.lock
├── Cargo.toml                            # Cargo workspace
├── CONTRIBUTING.md -> docs/en/CONTRIBUTING.md
├── flake.lock
├── flake.nix                             # Nix flake entry point
├── LICENSE                               # Code license
└── README.md -> docs/en/README.md
```

## Roadmap

- [ ] **Installation**
  - [ ] Publish crates on [crates.io](https://crates.io), including binary applications
  - [ ] Package Managers
    - [ ] AUR
      - [ ] `spyland-git`
      - [ ] `spyland-bin`
  - [x] systemd user services
  - [x] ~~Nix~~ [#5](https://github.com/spyland-dev/spyland/pull/5)
- [ ] **New backends**
  - [ ] Hyprland
  - [ ] KDE
  - [ ] Sway
  - [ ] *Mutter?*
- [ ] **Gtk application**
- [ ] **Database encryption**: Protect user data
- [ ] **Database integrity checks**: Validate data on load
- [x] ~~**Activities**: Grouping sessions (work, entertainment, study)~~
- [x] ~~**Runtime backend loading**: Dynamic backend loading without recompilation.~~ [#1](https://github.com/spyland-dev/spyland/pull/1)

## License

This project is licensed under GNU GPL v3.0. See [LICENSE](../../LICENSE) for details.
