{
  description = "A simple axum server wrapping brother_ql";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nci.url = "github:yusdacra/nix-cargo-integration";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];

      imports = [
        inputs.nci.flakeModule
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      flake = {config, ...}: {
        nixosModules.bql-print = import ./nix/module.nix inputs;
        nixosModules.default = config.nixosModules.bql-print;
      };

      perSystem = {
        pkgs,
        config,
        ...
      }: let
        crateName = "bql-print";
        projectName = crateName;
        crateOutput = config.nci.outputs.${crateName};
      in {
        formatter = pkgs.alejandra;
        nci = {
          projects.${projectName}.path = ./.;
          crates.${crateName} = {};
        };
        devShells.default = crateOutput.devShell.overrideAttrs (old: {
          nativeBuildInputs =
            (with pkgs; [
              nil
              rust-analyzer
              cargo-watch
            ])
            ++ old.nativeBuildInputs;

          BQL_PRINT_ADDRESS = "localhost";
          BQL_PRINT_PORT = 3000;
          BQL_PRINT_PRINTER_ADDRESS = "192.168.178.39";
          BQL_PRINT_PRINTER_PORT = 9100;
        });
        packages.default = crateOutput.packages.release;
        overlayAttrs.bql-print = config.packages.default;
      };
    };
}
