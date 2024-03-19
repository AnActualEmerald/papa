{
  description = "A command line mod manager for Northstar";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = import nixpkgs {system = "x86_64-linux";};
  in {
    formatter.x86_64-linux = pkgs.alejandra;

    devShells.x86_64-linux.default = pkgs.mkShell {
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = [pkgs.rustc pkgs.openssl];
      packages = [pkgs.just pkgs.cargo pkgs.cargo-watch];
    };
  };
}
