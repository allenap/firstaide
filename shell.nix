{ ... }:
let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
  niv = import sources.niv { };
in pkgs.mkShell {
  buildInputs = with pkgs; [ niv.niv git pkgs.rustc pkgs.cargo pkgs.libiconv ];
}
