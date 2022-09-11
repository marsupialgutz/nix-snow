{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    fenix,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      rust = with fenix.packages.${system}; rec {
        native = latest;
        dev.toolchain = combine [native.toolchain rust-analyzer];
      };
    in {
      defaultPackage = naersk-lib.buildPackage ./.;

      defaultApp = utils.lib.mkApp {
        drv = self.defaultPackage."${system}";
      };

      devShell = with pkgs;
        mkShell {
          nativeBuildInputs = [
            rust.dev.toolchain
            mold
            cmake
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;

          LD_LIBRARY_PATH =
            nixpkgs.lib.strings.makeLibraryPath
            (with pkgs; [
              xorg.libX11
              xorg.libXcursor
              # xorg.libXrandr
              libxkbcommon
            ]);

          MOLD_PATH = "${pkgs.mold}/bin/mold";
          LD_PRELOAD = "${pkgs.mold}/lib/mold/mold-wrapper.so";
        };
    });
}
