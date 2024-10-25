{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    devshell.url = "github:numtide/devshell";
    devshell.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} ({config, ...}: {
      imports = [
        inputs.devshell.flakeModule
        inputs.treefmt-nix.flakeModule
      ];

      flake.internal =
        inputs.nixpkgs.lib.genAttrs config.systems (
          targetSystem: import ./nix/internal/unix.nix {inherit inputs targetSystem;}
        )
        // inputs.nixpkgs.lib.genAttrs ["x86_64-windows"] (
          targetSystem: import ./nix/internal/windows.nix {inherit inputs targetSystem;}
        );

      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        config,
        system,
        pkgs,
        ...
      }: {
        packages =
          {
            default = inputs.self.internal.${system}.package;
          }
          // (inputs.nixpkgs.lib.optionalAttrs (system == "x86_64-linux") {
            default-x86_64-windows = inputs.self.internal.x86_64-windows.package;
          });

        devshells.default = import ./nix/devshells.nix {inherit inputs;};

        treefmt = {
          projectRootFile = "flake.nix";
          programs.alejandra.enable = true;
          programs.rustfmt.enable = true;
          programs.toml-sort.enable = true;
          programs.yamlfmt.enable = true;
        };
      };
    });
}
