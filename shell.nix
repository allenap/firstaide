{ ... }:
let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
  niv = import sources.niv { };
in pkgs.mkShell {
  buildInputs = with pkgs; [
    git
    niv.niv
    pkgs.cargo
    pkgs.cargo-edit # for `cargo upgrade`
    pkgs.clippy
    pkgs.libiconv
    pkgs.rustc
    pkgs.rustfmt
  ];
}
