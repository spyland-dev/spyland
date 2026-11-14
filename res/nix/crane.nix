{ pkgs, crane }:

let
  craneLib = crane.mkLib pkgs;
  src = pkgs.lib.cleanSourceWith {
    src = ../..;
    filter = path: type:
      let
        pathString = toString path;
        sqlxDir = "${toString ../..}/crates/lib/.sqlx";
      in
      craneLib.filterCargoSources path type
      || pathString == sqlxDir
      || pkgs.lib.hasPrefix "${sqlxDir}/" pathString;
  };

  baseArgs = {
    inherit src;
    strictDeps = true;

    cargoLock = ../../Cargo.lock;
    cargoToml = ../../Cargo.toml;

    SQLX_OFFLINE = "true";
  };

  cargoArtifacts = craneLib.buildDepsOnly baseArgs;
  commonArgs = baseArgs // { inherit cargoArtifacts; };

  cli = craneLib.buildPackage (commonArgs // {
    pname = "spyland";
    cargoExtraArgs = "-p spyland-cli";
  });
  daemon = craneLib.buildPackage (commonArgs // {
    pname = "spylandd";
    cargoExtraArgs = "-p spylandd";
  });
  core = craneLib.buildPackage (commonArgs // {
    pname = "spyland-core";
    cargoExtraArgs = "-p spyland-core";
  });
  lib = craneLib.buildPackage (commonArgs // {
    pname = "spyland-lib";
    cargoExtraArgs = "-p spyland-lib";
  });
  niri = craneLib.buildPackage (commonArgs // {
    pname = "spyland-backend-niri";
    cargoExtraArgs = "-p spyland-backend-niri";
  });

  commonMeta = {
    meta = {
      homepage = "https://github.com/spyland-dev/spyland";
      platforms = [ "x86_64-linux" ];
      license = pkgs.lib.licenses.gpl3Plus;
    };
  };
in
{
  packages = { inherit cli daemon core lib niri; };

  checks = {
    fmt = craneLib.cargoFmt { inherit src; };

    inherit cli daemon core lib niri;

    test = craneLib.cargoNextest commonArgs;

    doc = craneLib.cargoDoc commonArgs;

    clippy = craneLib.cargoClippy (commonArgs // {
      cargoClippyExtraArgs = "--all-targets --all-features -- -D warnings";
    });
  };

  apps = {
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
}
