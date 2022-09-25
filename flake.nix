{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      naersk-lib = pkgs.callPackage naersk {};
    in {
      defaultPackage = naersk-lib.buildPackage {
        root = ./.;
        buildPhase = ''
          makeWrapper $out/bin/foo $wrapperfile \
            --prefix PATH : ${lib.makeBinPath [hello git]} \
            --suffix PATH : ${lib.makeBinPath [xdg-utils]}
        '';
      };

      defaultApp = utils.lib.mkApp {
        drv = self.defaultPackage."${system}";
      };

      devShell = with pkgs;
        mkShell {
          buildInputs = [cargo rustc rustfmt pre-commit rustPackages.clippy];
          nativeBuildInputs = [makeWrapper];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
}
