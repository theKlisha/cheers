{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/25.11";
    };
  };

  outputs =
    { self, nixpkgs }:
    let
      forEachSystem =
        f:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
          system:
          f rec {
            inherit system;
            pkgs = import nixpkgs { inherit system; };
          }
        );
    in
    {
      devShells = forEachSystem (
        { pkgs, ... }:
        {
          default = pkgs.mkShellNoCC {
            packages = [
              pkgs.cargo
              pkgs.rust-analyzer
              pkgs.fastchess
            ];
          };
        }
      );
    };
}
