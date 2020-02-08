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
  version = "0.1.1";
  src = gitignoreSource ./.;

  # The crypto_hash crate needs the openssl-sys crate (directly or indirectly,
  # I don't know) which ultimately needs openssl proper, and pkg-config.
  buildInputs = [ openssl pkg-config ];

  # Don't run tests when building.
  checkPhase = "";

  # I think this refers to the current state of the crates.io repo.
  cargoSha256 = "0rfkp3ka4apyd0smvalpqg90x02hdpih90qb6l0n9x5pdzpkjb7a";

  meta = with stdenv.lib; {
    description = "Bootstrap Nix environments.";
    homepage = "https://github.com/allenap/firstaide";
    license = with licenses; [ asl20 ];
    maintainers = [ ];
    platforms = platforms.all;
  };
}
