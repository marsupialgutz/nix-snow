{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    fenix,
    flake-utils,
    naersk,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (system: {
      packages.default = let
        toolchain = fenix.packages.${system}.minimal.toolchain;
      in
        (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        })
        .buildPackage {
          src = ./.;
        };
      devShell = let
        pkgs = nixpkgs.legacyPackages.${system};
      in
        with pkgs;
          mkShell {
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
    });
}
