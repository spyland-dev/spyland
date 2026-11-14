{
  description = "Nix shell for Rust `spyland` development";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.crane.url = "github:ipetkov/crane";

  outputs = { nixpkgs, crane, ... }:
    let
      system = builtins.currentSystem or "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      rust = import ./res/nix/crane.nix { inherit pkgs crane; };
      devShell = import ./res/nix/dev-shell.nix { inherit pkgs; };
    in
    {
      packages.${system} = rust.packages;
      checks.${system} = rust.checks;
      apps.${system} = rust.apps;
      devShells.${system}.default = devShell;

      nixosModules.default = {
	imports = [ ./module.nix ];
	_module.args = {
	  pkg = pkgs;
	  defaultDaemonPkg = rust.packages.daemon;
	  defaultCliPkg = rust.packages.cli;
	  defaultNiriPkg = rust.packages.niri;
	};
      };
  };
}
