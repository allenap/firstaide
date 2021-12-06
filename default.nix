{ sources ? import ./nix/sources.nix { }, pkgs ? import sources.nixpkgs { }, ...
}:
with pkgs;
let
  sources = import ./nix/sources.nix;
  gitignore = import sources.gitignore { };
in rustPlatform.buildRustPackage rec {
  pname = "firstaide";
  version = "0.1.6";
  src = gitignore.gitignoreSource ./.;

  # The crypto_hash crate needs the openssl-sys crate (directly or indirectly,
  # I don't know) which ultimately needs openssl proper, and pkg-config.
  buildInputs = [ openssl pkg-config ];

  # Don't run tests when building.
  doCheck = false;

  # I think this refers to the current state of the crates.io repo. To update,
  # replace the hash with all 0's and Nix will give you the right value to
  # stick in here.
  cargoSha256 = "0si3fwfyxinlf3ccc9l4jv0k0f9bn2200b0f9hhvz47w9zln2hlp";

  meta = with pkgs.lib; {
    description = "Bootstrap Nix environments.";
    homepage = "https://github.com/allenap/firstaide";
    license = with licenses; [ asl20 ];
    maintainers = [ ];
    platforms = platforms.all;
  };
}
