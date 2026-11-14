{ config, lib, pkg, defaultCliPkg, defaultNiriPkg, defaultDaemonPkg, ... }:

let
  inherit (lib) mkEnableOption mkIf mkOption mkMerge types;

  ccfg = config.programs.spyland;
  dcfg = config.services.spylandd;
  ncfg = config.services.spylandd.backends.niri;
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
      description = "Log level passed to spylandd and backends.";
    };

    coreOptions = mkOption {
      type = types.attrsOf types.anything;
      default = { };
      description = "Configuration for spylandd; provides `core` section in TOML configuration.";
    };
  };

  options.services.spylandd.backends.niri = {
    enable = mkEnableOption "The spyland niri compositor backend";

    package = mkOption {
      type = types.package;
      default = defaultNiriPkg;
      defaultText = "The flake's niri backend package";
      description = "Package providing the spyland-backend-niri executable.";
    };
  };

  config =
    let
      toml = pkg.formats.toml { };
      configToml = toml.generate "spyland.toml" (
        (lib.optionalAttrs ccfg.enable {
          frontend.cli = ccfg.settings;
        }) // (lib.optionalAttrs dcfg.enable {
          core = dcfg.coreOptions;
        })
      );
      coreOptionsHash = builtins.hashString "sha256" (builtins.toJSON dcfg.coreOptions);
    in
    mkMerge [
      (mkIf (ccfg.enable || dcfg.enable) {
        environment.etc."spyland.toml".source = configToml;
      })

      (mkIf ccfg.enable {
        environment.systemPackages = [ ccfg.package ];
      })

      (mkIf ncfg.enable {
	systemd.user.units = {
	  "spyland-backend-niri.service" = {
	    text = builtins.readFile ../services/spyland-backend-niri.service;
	    wantedBy = [ "spyland-backends.target" ];
	  };

          "spyland-backend-niri.service.d/override.conf".text = ''
            [Service]
            Environment=RUST_LOG=${dcfg.logLevel}
            ExecStart=
            ExecStart=${ncfg.package}/bin/spyland-backend-niri
          '';
	};
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

            [Unit]
            X-Restart-Triggers=${coreOptionsHash}
          '';

          "spyland-backends.target" = {
            text = builtins.readFile ../services/spyland-backends.target;
            wantedBy = [ "graphical-session.target" ];
          };
          "spyland-backends.target.d/core-options-restart.conf".text = ''
            [Unit]
            X-Restart-Triggers=${coreOptionsHash}
          '';
        };
      })

      {
        assertions = [
          {
            assertion = !ncfg.enable || dcfg.enable;
            message = "services.spylandd.backends.niri.enable requires services.spylandd.enable.";
          }
        ];
      }
    ];
}
