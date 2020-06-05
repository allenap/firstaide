use std::env::var_os;
use std::path::PathBuf;

// Find the system-wide nix.conf according to the same rules that Nix uses.
//
// nix.conf(5) has this to say:
//
//   Nix reads settings from two configuration files:
//
//   - The system-wide configuration file sysconfdir/nix/nix.conf (i.e.
//     /etc/nix/nix.conf on most systems), or $NIX_CONF_DIR/nix.conf if
//     NIX_CONF_DIR is set.
//
//   - The user configuration file $XDG_CONFIG_HOME/nix/nix.conf, or
//     ~/.config/nix/nix.conf if XDG_CONFIG_HOME is not set.
//
fn find_system_nix_conf() -> PathBuf {
    match var_os("NIX_CONF_DIF") {
        Some(nix_conf_dir) => {
            // Nix doesn't care if this file exists or not, so we don't either.
            PathBuf::from(nix_conf_dir).join("nix.conf")
        }
        None => {
            // It doesn't seem possible or at least obvious how to find out what
            // Nix's notion of `sysconfdir` is, so we assume it's /etc for now.
            PathBuf::from("/etc/nix/nix.conf")
        }
    }
}

// nix.conf(5) explains its format:
//
//   The configuration files consist of `name = value` pairs, one per line.
//   Other files can be included with a line like `include` path, where `path`
//   is interpreted relative to the current conf file and a missing file is an
//   error unless `!include` is used instead. Comments start with a # character.
//   Here is an example configuration file:
//
//     keep-outputs = true       # Nice for developers
//     keep-derivations = true   # Idem
//
// Reading the code for Nix's applyConfigFile clarifies some things:
//
// - When it says "one per line" it means it: there is no way to break
//   a setting over multiple lines.
//
// - Characters cannot be escaped, strings cannot be quoted.
//
// - When using include or !include, the filename is used as-is. It cannot
//   contain whitespace; that would result in an error.
//
// - Options can contain whitespace (" \r\t") but each run of whitespace is
//   normalized into a single space.
//
// - Everything is bytes; there's no encoding or decoding using character sets.
//
// - Included files can be relative paths. These are resolved relative to the
//   directory of the file being read.
//
