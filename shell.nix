{ ... }:
let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
  niv = import sources.niv { };
in pkgs.mkShell {
  buildInputs = with pkgs; [ git niv.niv pkgs.cargo pkgs.libiconv pkgs.rustc ];
}
