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
    channel = "stable";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "llvm-tools"
    ];
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
  };

  packages = [
    pkgs.ltspice
    pkgs.cargo-llvm-cov
    pkgs.cargo-nextest
  ];

  # See full reference at https://devenv.sh/reference/options/
}
