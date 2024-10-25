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
    {package = inputs.self.formatter.${pkgs.system};}
    {package = config.language.rust.packageSet.cargo;}
    {package = pkgs.rust-analyzer;}
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
