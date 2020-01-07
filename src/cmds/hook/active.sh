# shellcheck shell=bash
log_status "$(okay OKAY): $(em 'Nix environment is up-to-date!')" >&2
log_status "This is a $(em minimal environment); subprojects may not be built." >&2
log_status "--> Use $(em aide build help) to find out what to do next." >&2
log_status "        $(em ^^^^^^^^^^^^^^^)" >&2
