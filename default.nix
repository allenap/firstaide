with import <nixpkgs> { };

let
  # A handy tool to correctly process .gitignore files.
  gitignoreSrc = pkgs.fetchFromGitHub {
    owner = "hercules-ci";
    repo = "gitignore";
    rev = "7415c4feb127845553943a3856cbc5cb967ee5e0";
    sha256 = "sha256:1zd1ylgkndbb5szji32ivfhwh04mr1sbgrnvbrqpmfb67g2g3r9i";
  };
  inherit (import gitignoreSrc { inherit (pkgs) lib; }) gitignoreSource;

in rustPlatform.buildRustPackage rec {
  pname = "firstaide";
  version = "0.1.5";
  src = gitignoreSource ./.;

  # The crypto_hash crate needs the openssl-sys crate (directly or indirectly,
  # I don't know) which ultimately needs openssl proper, and pkg-config.
  buildInputs = [ openssl pkg-config ];

  # Don't run tests when building.
  checkPhase = "";

  # I think this refers to the current state of the crates.io repo.
  cargoSha256 = "1766w77b582c6j2qgqbfy652nxr90kgqw72fdwyd8mb3wjxxi35p";

  meta = with stdenv.lib; {
    description = "Bootstrap Nix environments.";
    homepage = "https://github.com/allenap/firstaide";
    license = with licenses; [ asl20 ];
    maintainers = [ ];
    platforms = platforms.all;
  };
}
