{
  description = "Nix shell for Rust `spyland` development";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.crane.url = "github:ipetkov/crane";

  outputs = { nixpkgs, crane, ... }:
    let
      system = builtins.currentSystem or "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      varDir = "/var/tmp/spyland-debug/";
      database = "${varDir}/sessions.sqlite";
      socket = "${varDir}/spyland.sock";
      config = "${varDir}/config.toml";

      craneLib = crane.mkLib pkgs;
      src = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          let
            pathString = toString path;
            sqlxDir = "${toString ./.}/crates/lib/.sqlx";
          in
          craneLib.filterCargoSources path type
          || pathString == sqlxDir
          || pkgs.lib.hasPrefix "${sqlxDir}/" pathString;
      };

      commonArgs = {
	inherit src;

	cargoLock = ./Cargo.lock;
	cargoToml = ./Cargo.toml;

	SQLX_OFFLINE = "true";

	strictDeps = true;
      };

      cli = craneLib.buildPackage (
	commonArgs
	// {
	  pname = "spyland";
	  cargoExtraArgs = "-p spyland-cli";
	}
      );
      daemon = craneLib.buildPackage (
	commonArgs
	// {
	  pname = "spylandd";
	  cargoExtraArgs = "-p spylandd";
	}
      );
    in
    {
      packages.${system} = { inherit cli daemon; };
      apps.${system} = {
	cli = {
	  type = "app";
	  program = "${cli}/bin/${cli.pname}";
	};
	daemon = {
	  type = "app";
	  program = "${daemon}/bin/${daemon.pname}";
	};
      };
      nixosModules.default = { config, lib, ... }:
        import ./res/nix/module.nix {
          inherit config lib;
          defaultDaemonPkg = daemon;
          defaultCliPkg = cli;
        };
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
	  rustc
	  cargo
	  rustfmt
	  rust-analyzer
	  clippy

	  just

	  cargo-nextest # optional

	  sqlite
	  sqlx-cli
	];

	shellHook = "mkdir -p ${varDir}"; 

	# Override spyland's directories to `varDir` to avoid conflicts
	# between the release and development versions running at the same time.
	SPYLAND_DATABASE = database;
	SPYLAND_SOCKET = socket;
	SPYLAND_CONFIG = config;

	SQLX_OFFLINE = true;
	DATABASE_URL = "sqlite://${database}";
	RUST_LOG = "debug";
      };
    };
}
