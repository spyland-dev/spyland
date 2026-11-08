{ config, lib, defaultPackage, ... }:
let
  inherit (lib) mkEnableOption mkIf mkOption types;
  cfg = config.services.spylandd;
in
{
  options.services.spylandd = {
    enable = mkEnableOption "spyland screen-time tracking daemon";

    package = mkOption {
      type = types.package;
      default = defaultPackage;
      defaultText = "The flake's daemon package";
      description = "Package providing the spylandd executable.";
    };

    logLevel = mkOption {
      type = types.enum [ "trace" "debug" "info" "warn" "error" "off" ];
      default = "info";
      description = "Log level passed to spylandd.";
    };
  };

  config = mkIf cfg.enable {
    systemd.user.units = {
      "spylandd.service".text = builtins.readFile ../services/spylandd.service;
      "spyland-backends.target".text = builtins.readFile ../services/spyland-backends.target;
    };
    systemd.services."spylandd" = {
      enable = true;
      serviceConfig.ExecStart = "${cfg.package}/bin/spylandd -l ${cfg.logLevel}";
    };
  };
}
