{
  description = "Offline password manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    tomers = {
      url = "github:boralg/tomers";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, tomers, ... }:
    tomers.inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        targetPlatforms =
          let
            toolchainPackages =
              fenixPkgs: crossFenixPkgs: with fenixPkgs; [
                latest.rustfmt
                stable.rust-src
              ];
          in
          [
            {
              system = "x86_64-unknown-linux-gnu";
              arch = "x86_64-linux";
              inherit toolchainPackages;
            }
          ];
        tomersLib = tomers.libFor system targetPlatforms;
      in
      rec {
        packagesForEachPlatform = tomersLib.packagesForEachPlatform;
        devShellsForEachPlatform = tomersLib.devShellsForEachPlatform;

        packages = packagesForEachPlatform ./.;
        devShells = devShellsForEachPlatform ./.;
      }
    );
}
