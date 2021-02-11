# shellcheck shell=bash
log_status "$(warning WARNING): $(em 'Nix environment is out of date!') " >&2
log_status "--> Use $(em firstaide-update) to rebuild it." >&2
log_status "$(warning WARNING): Loading $(em STALE) environment ;-(" >&2
