{ pkgs ? import <nixpkgs> { }, ... }:
with pkgs;
let
  sources = import ./nix/sources.nix;
  gitignore = import sources.gitignore { };
in rustPlatform.buildRustPackage rec {
  pname = "firstaide";
  version = "0.1.5";
  src = gitignore.gitignoreSource ./.;

  # The crypto_hash crate needs the openssl-sys crate (directly or indirectly,
  # I don't know) which ultimately needs openssl proper, and pkg-config.
  buildInputs = [ openssl pkg-config ];

  # Don't run tests when building.
  checkPhase = "";

  # I think this refers to the current state of the crates.io repo. To update,
  # replace the hash with all 0's and Nix will give you the right value to
  # stick in here.
  cargoSha256 = "1766w77b582c6j2qgqbfy652nxr90kgqw72fdwyd8mb3wjxxi35p";

  meta = with stdenv.lib; {
    description = "Bootstrap Nix environments.";
    homepage = "https://github.com/allenap/firstaide";
    license = with licenses; [ asl20 ];
    maintainers = [ ];
    platforms = platforms.all;
  };
}
