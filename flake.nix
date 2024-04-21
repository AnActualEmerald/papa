{
  description = "A command line mod manager for Northstar";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      buildDeps = with pkgs; [pkg-config openssl rustc];
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      rustPackage = features:
        pkgs.rustPlatform.buildRustPackage {
          inherit (cargoToml.package) name version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "libthermite-0.8.1" = "sha256-DPc6Nt8BN0Q+t+bf3p171BiIXuAAcVBbve2rR1l9QTg=";
            };
          };
          buildFeatures = features;
          nativeBuildInputs = buildDeps;
        };
    in rec {
      packages.papa = rustPackage "";

      packages.default = packages.papa;

      formatter = pkgs.alejandra;

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = buildDeps;
        packages = [pkgs.just pkgs.cargo pkgs.cargo-watch pkgs.rust-analyzer];
      };
    });
}
