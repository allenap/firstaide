# shellcheck shell=bash
log_status "$(okay OKAY): $(em 'Nix environment is up to date!')" >&2
log_status "This is a $(em minimal environment); subprojects may not be built." >&2
log_status "--> Use $(em __MESSAGE__) to find out what to do next." >&2
log_status "        $(m=__MESSAGE__ && em "${m//?/^}")" >&2
