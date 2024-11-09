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
      meta = {
        description = "Periodic table generator in the SVG format";
        homepage = "https://github.com/michalrus/periodic-table-generator";
        license = lib.licenses.asl20;
      };
    });

  chemfig2svg = pkgs.writeShellApplication {
    name = "chemfig2svg";
    runtimeInputs = [
      (pkgs.texlive.withPackages (ps:
        with ps; [
          scheme-small
          chemfig
          simplekv
          standalone
          dvisvgm
        ]))
      pkgs.xmlstarlet
      pkgs.coreutils
      pkgs.stdenv.shell
      pkgs.gnused
      pkgs.getopt
    ];
    text = builtins.readFile ./chemfig2svg.sh;
    derivationArgs.meta.description = "Converts LaTeX chemfig expressions to SVG";
  };
}
