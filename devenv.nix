{
  pkgs,
  lib,
  config,
  ...
}:
{
  # overlays = [
  #   (final: prev: {
  #     ltspice = prev.ltspice.overrideAttrs (oldAttrs: {
  #       src = pkgs.fetchurl {
  #         url = "https://www.analog.com/media/en/simulation-models/spice-models/ltspiceXVIIx64Setup.exe";
  #         sha256 = "LSK84ogbBk9kP7LKg8rzCGDqq36XfsK4Kzn2Zwea8C4=";
  #       };
  #     });
  #   })
  # ];

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
  };

  packages = [ pkgs.ltspice ];

  # See full reference at https://devenv.sh/reference/options/
}
