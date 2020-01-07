# shellcheck shell=bash
# Load a parent directory's `.envrc` since `direnv` doesn't do this by default,
# and its solution – the `source_up` function – is kind of insecure.
direnv_load "${direnv:-direnv}" exec .. "${direnv:-direnv}" dump
