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
                stable.rustc
                stable.cargo
                crossFenixPkgs.stable.rust-src
                crossFenixPkgs.stable.rust-std
              ];
            libPath =
              with pkgs;
              lib.makeLibraryPath [
                xdotool
                libxkbcommon
                wayland
                xorg.libX11
                xorg.libXrandr
                xorg.libXrender
                xorg.libXcursor
                xorg.libxcb
                xorg.libXi
                libGL
                vulkan-loader
              ];
          in
          [
            {
              system = "x86_64-unknown-linux-gnu";
              arch = "x86_64-linux";
              inherit toolchainPackages;
              depsBuild = with pkgs; [
                patchelf
                pkg-config
                xorg.libX11
                xdotool
                libxkbcommon
              ];
              postInstall = crateName: ''
                find $out -type f -exec sh -c '
                  if file "$1" | grep -q "ELF .* executable"; then
                    patchelf --set-interpreter "/lib64/ld-linux-x86-64.so.2" "$1"
                  fi
                ' sh {} \;
              '';
              postInstallNixStore = crateName: ''
                find $out -type f -exec sh -c '
                  if file "$1" | grep -q "ELF .* executable"; then
                    patchelf --set-rpath "${libPath}" "$1"
                  fi
                ' sh {} \;
              '';
              env = {
                buildInputs = with pkgs; [
                  xdotool
                ];
                LD_LIBRARY_PATH = libPath;
                dontPatchELF = true; # do not compress RPATH since winit uses dlopen
              };
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
