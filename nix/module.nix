inputs: {
  config,
  lib,
  pkgs,
  ...
}: let
  inherit
    (lib)
    mkOption
    mkEnableOption
    mkPackageOption
    types
    mkIf
    ;
  cfg = config.services.bql-print;
in {
  options.services = {
    bql-print = {
      enable = mkEnableOption "bql-print";
      package = mkPackageOption pkgs "bql-print" {};
      address = mkOption {
        description = ''
          Address to listen on
        '';
        type = types.str;
        default = "127.0.0.1";
        example = "[::1]";
      };
      port = mkOption {
        description = ''
          Port to listen on
        '';
        type = types.port;
        default = 3332;
      };
      printer-address = mkOption {
        description = ''
          Address of the label printer
        '';
        type = types.str;
        default = "192.168.178.39";
        example = "[::1]";
      };
      printer-port = mkOption {
        description = ''
          Port of the label printer
        '';
        type = types.port;
        default = 9100;
      };
    };
  };
  config = {
    nixpkgs.overlays = [
      inputs.self.overlays.default
    ];
    systemd.services = {
      bql-print = mkIf cfg.enable {
        description = "bql-print";
        after = ["network.target"];
        wantedBy = ["multi-user.target"];
        serviceConfig = {
          ExecStart = "${cfg.package}/bin/bql-print";
          DynamicUser = true;
          Restart = "on-failure";
        };
        environment = {
          BQL_PRINT_ADDRESS = cfg.address;
          BQL_PRINT_PORT = toString cfg.port;
          BQL_PRINT_PRINTER_ADDRESS = cfg.printer-address;
          BQL_PRINT_PRINTER_PORT = toString cfg.printer-port;
        };
      };
    };
  };
}
