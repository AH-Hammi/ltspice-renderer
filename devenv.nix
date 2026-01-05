{
  pkgs,
  lib,
  config,
  ...
}:
{
  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
  };

  packages = [ pkgs.ltspice ];

  git-hooks.hooks = {
    clippy.enable = true;
    unit-tests = {
      enable = true;
      name = "Cargo Unit Tests";
      entry = "cargo test --all";
      files = "**/*.rs";
    };
  }

  # See full reference at https://devenv.sh/reference/options/
}
