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

      commonMeta = {
	meta = {
	  homepage = "https://github.com/spyland-dev/spyland";
	  platforms = [ "x86_64-linux" ];
	  license = pkgs.lib.licenses.gpl3Plus;
	};
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
      niri = craneLib.buildPackage (
	commonArgs
	// {
	  pname = "spyland-backend-niri";
	  cargoExtraArgs = "-p spyland-backend-niri";
	}
      );
    in
    {
      packages.${system} = { inherit cli daemon niri; };
      apps.${system} = {
	cli = commonMeta // {
	  type = "app";
	  program = "${cli}/bin/${cli.pname}";

	  meta.description = "The spyland command-line interface";
	  meta.mainProgram = "spyland";
	};
	daemon = commonMeta // {
	  type = "app";
	  program = "${daemon}/bin/${daemon.pname}";

	  meta.description = "The spyland screen-time tracking daemon";
	  meta.mainProgram = "spylandd";
	};
	niri = commonMeta // {
	  type = "app";
	  program = "${niri}/bin/${niri.pname}";

	  meta.description = "The spyland niri compositor backend";
	  meta.mainProgram = "spyland-backend-niri";
	};
      };
      nixosModules.default = {
        imports = [ ./res/nix/module.nix ];
        _module.args = {
          pkg = pkgs;
          defaultDaemonPkg = daemon;
          defaultCliPkg = cli;
	  defaultNiriPkg = niri;
        };
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
