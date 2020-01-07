# shellcheck shell=bash
if tty -s
then
    okay() { printf '\033[1;32m%s\033[0m' "$*"; }
    warning() { printf '\033[1;33m%s\033[0m' "$*"; }
    error() { printf '\033[1;31m%s\033[0m' "$*"; }
    em() { printf '\033[1m%s\033[0m' "$*"; }
else
    okay() { printf '%s' "$*"; }
    warning() { printf '*%s*' "$*"; }
    error() { printf '*** %s ***' "$*"; }
    em() { printf '%s' "$*"; }
fi
