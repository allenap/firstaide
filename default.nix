{ pkgs ? import <nixpkgs> { }, ... }:
with pkgs;
let
  sources = import ./nix/sources.nix;
  gitignore = import sources.gitignore { };
  naersk = pkgs.callPackage sources.naersk { };
in naersk.buildPackage (gitignore.gitignoreSource ./.)
