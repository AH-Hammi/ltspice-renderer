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

  # https://devenv.sh/guides/rust-in-depth/#rust-analyzer
  # rust-analyzer is enabled by default

  # See full reference at https://devenv.sh/reference/options/
}
