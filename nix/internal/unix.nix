{
  inputs,
  targetSystem,
}:
assert __elem targetSystem ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"]; let
  buildSystem = targetSystem;
  pkgs = inputs.nixpkgs.legacyPackages.${buildSystem};
  inherit (pkgs) lib;
in rec {
  craneLib = inputs.crane.mkLib pkgs;

  src = craneLib.cleanCargoSource ../../.;

  commonArgs = {
    inherit src;
    strictDeps = true;
    buildInputs = lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  package = craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
    });
}
