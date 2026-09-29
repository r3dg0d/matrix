{
  description = "matrix — realtime Matrix-style digital rain for the terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        package = rustPlatform.buildRustPackage {
          pname = "matrix";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          meta = with pkgs.lib; {
            description = "Realtime Matrix-style digital rain for the terminal";
            license = licenses.mit;
            mainProgram = "matrix";
            platforms = platforms.unix;
          };
        };
      in {
        packages.default = package;
        packages.matrix = package;
        apps.default = {
          type = "app";
          program = "${package}/bin/matrix";
        };
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.cargo
            pkgs.rustfmt
            pkgs.clippy
            pkgs.pkg-config
          ];
        };
      });
}
