# shellcheck shell=bash
if [[ "${__monorepo_env:-not set}" != active ]]
then
    # The __monorepo_env environment variable should be set in shell.nix.
    # Here we check that expectation, belt-n-braces like.
    log_status "$(warning WARNING): $(em "__monorepo_env is '${__monorepo_env}'; expected 'active'.")"
fi
