{inputs}: {
  config,
  pkgs,
  ...
}: let
  inherit (pkgs) lib;
  internal = inputs.self.internal.${pkgs.system};
in {
  name = "periodic-table-generator-devshell";

  imports = [
    "${inputs.devshell}/extra/language/c.nix"
    "${inputs.devshell}/extra/language/rust.nix"
  ];

  devshell.packages = [];

  commands = [
    {package = internal.package;}
    {package = internal.chemfig2svg;}
    {package = internal.tikz2svg;}
    {
      package = inputs.self.formatter.${pkgs.system};
      category = "dev";
    }
    {
      package = config.language.rust.packageSet.cargo;
      category = "dev";
    }
    {
      package = pkgs.rust-analyzer;
      category = "dev";
    }
    {
      package = pkgs.shellcheck;
      name = "shellcheck";
      category = "dev";
    }
  ];

  language.c.compiler =
    if pkgs.stdenv.isLinux
    then pkgs.gcc
    else pkgs.clang;
  language.c.includes = internal.commonArgs.buildInputs;

  devshell.motd = ''

    {202}🔨 Welcome to ${config.name}{reset}
    $(menu)

    You can now run ‘{bold}cargo run{reset}’ or ‘{bold}nix run -L{reset}’.
  '';
}
