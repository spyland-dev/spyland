{
  description = "Nix shell for Rust `spyland` development";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = builtins.currentSystem or "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      varDir = "/var/tmp/spyland-debug/";
      database = "${varDir}/sessions.sqlite";
      socket = "${varDir}/spyland.sock";
      config = "${varDir}/config.toml";
    in
    {
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
