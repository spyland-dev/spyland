{ config, lib, defautlCliPkg, defaultDaemonPkg, ... }:
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
      default = defautlCliPkg;
      defaultText = "The flake's CLI package";
      description = "Package providing the spyland (CLI) executable.";
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
  };

  config = mkMerge [
    mkIf ccfg.enable {
      environment.systemPackages = [ ccfg.package ];
    }

    mkIf dcfg.enable {
      systemd.user.units = {
        "spylandd.service".text = builtins.readFile ../services/spylandd.service;
        "spyland-backends.target".text = builtins.readFile ../services/spyland-backends.target;
      };
      systemd.services."spylandd" = {
        enable = true;
        serviceConfig.ExecStart = "${dcfg.package}/bin/spylandd -l ${dcfg.logLevel}";
      };
    }
  ];
}
