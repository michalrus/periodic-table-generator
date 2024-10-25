{
  inputs,
  targetSystem,
}:
assert __elem targetSystem ["x86_64-windows"]; let
  buildSystem = "x86_64-linux";
  pkgs = inputs.nixpkgs.legacyPackages.${buildSystem};
  inherit (pkgs) lib;
in rec {
  package = pkgs.runCommandNoCC "periodic-table-generator" {} ''
    echo >&2 'fatal: unimplemented'
    exit 1
  '';
}
