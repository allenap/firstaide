# firstaide

_Reviewed last on 2019-11-15_

This is a bootstrapping tool that'll build, cache, and clean your Nix environment. It supersedes many of loose collection of shell scripts and top-level Makefile that were in use.

## To develop:

```shell
cargo build  # Compiles a debug executable.
cargo test   # Compiles and tests.
cargo run    # Compiles and runs a debug executable.
# ...
```

## To build:

1. [Install the Rust development tools][install-rust].
2. From the top of the tree, `make -C iac/firstaide` will compile a release build of `firstaide` and install it into `iac/bin/$kernel.$machine`, e.g. `iac/bin/Darwin.x86_64`.
3. `git lfs track iac/bin/*/firstaide` to make sure they're put into LFS.
4. `git add .gitattributes iac/bin/*/firstaide` and commit.
5. You'll need to follow these steps on both a macOS and a Linux machine.

[install-rust]: https://www.rust-lang.org/tools/install
