# shellcheck shell=bash
log_status "$(error ERROR): $(em 'Nix environment is not yet built!')" >&2
log_status "--> Use $(em firstaide build) to build it." >&2
