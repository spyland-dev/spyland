{
  description = "Nix shell for Rust `spyland` development";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = builtins.currentSystem or "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
	  rustc
	  cargo
	  rustfmt
	  rust-analyzer

	  cargo-nextest # optional

	  sqlx-cli
	];

	SQLX_OFFLINE = true;
	DATABASE_URL = "sqlite://$HOME/.local/state/spyland/sessions.sqlite";
	RUST_LOG = "debug";
      };
    };
}
