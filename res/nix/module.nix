{ config, lib, pkg, defaultCliPkg, defaultDaemonPkg, ... }:
let
  inherit (lib) mkEnableOption mkIf mkOption mkMerge types;

  ccfg = config.programs.spyland;
  dcfg = config.services.spylandd;
in
{
  options.programs.spyland = {
    enable = mkEnableOption "The spyland command-line interface";

    package = mkOption {
      type = types.package;
      default = defaultCliPkg;
      defaultText = "The flake's CLI package";
      description = "Package providing the spyland (CLI) executable.";
    };

    settings = mkOption {
      type = types.attrsOf types.anything;
      default = { };
      description = "Configuration for spyland CLI; sets `frontend.cli` section in TOML configuration.";
    };
  };

  options.services.spylandd = {
    enable = mkEnableOption "The spyland screen-time tracking daemon";

    package = mkOption {
      type = types.package;
      default = defaultDaemonPkg;
      defaultText = "The flake's daemon package";
      description = "Package providing the spylandd (daemon) executable.";
    };

    logLevel = mkOption {
      type = types.enum [ "trace" "debug" "info" "warn" "error" "off" ];
      default = "info";
      description = "Log level passed to spylandd.";
    };

    coreOptions = mkOption {
      type = types.attrsOf types.anything;
      default = { };
      description = "Configuration for spylandd; provides `core` section in TOML configuration.";
    };
  };

  config =
    let
      toml = pkg.formats.toml { };
      configToml = toml.generate "config.toml" (
        (lib.optionalAttrs ccfg.enable {
          frontend.cli = ccfg.settings;
        }) // (lib.optionalAttrs dcfg.enable {
          core = dcfg.coreOptions;
        })
      );
    in
    mkMerge [
      (mkIf (ccfg.enable || dcfg.enable) {
        environment.variables.SPYLAND_CONFIG = configToml;
      })

      (mkIf ccfg.enable {
        environment.systemPackages = [ ccfg.package ];
      })

      (mkIf dcfg.enable {
        systemd.user.units = {
          "spylandd.service" = {
            text = builtins.readFile ../services/spylandd.service;
            wantedBy = [ "graphical-session.target" ];
          };
          "spylandd.service.d/override.conf".text = ''
            [Service]
            ExecStart=
            ExecStart=${dcfg.package}/bin/spylandd -l ${dcfg.logLevel}
          '';
          "spyland-backends.target" = {
            text = builtins.readFile ../services/spyland-backends.target;
            wantedBy = [ "graphical-session.target" ];
          };
        };
      })
    ];
}
