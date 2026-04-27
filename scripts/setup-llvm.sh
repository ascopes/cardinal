# shellcheck shell=sh
###
### Initialises the current shell to use LLVM correctly, best-effort.
###
### This script must be sourced rather than executed directly.
###

_log() {
  if [ -z "${NOCOLOR:-}" ] && [ -t 2 ]; then
    case "${1}" in
      INFO) _color=32 ;;
      WARN) _color=33 ;;
      FAIL) _color=31 ;;
      *) _log FAIL "bad color ${1}"; exit 1 ;;
    esac
    printf "%s \033[1;${_color}m[[%s]] \033[0;${_color}m%s\033[0m\n" "$(date "+%X")" "${1}" "${2}" >&2
  else
    printf "%s [[%s]] %s\n" "$(date "+%X")" "${1}" "${2}" >&2
  fi
}

_command_exists() {
  command -v "${1}" > /dev/null 2>&1
}

# It turns out it is a complete ballache to know if we were sourced or not
# as there is no standard way of determining this. None of this is foolproof
# below and is purely a best-effort guess. I have tested on sh, bash, dash,
# zsh, and the linux port of the OpenBSD ksh. Other shells may not work here.
#
# If changing this, make sure that at a minimum, the following fails consistently:
#   for s in sh bash dash ksh zsh; do echo "$s"; "$s" ./scripts/setup-llvm.sh; done
# ...and the following succeeds consistently:
#   for s in sh bash dash ksh zsh; do echo "$s"; "$s" -c '. ./scripts/setup-llvm.sh'; done
if [ -n "${ZSH_VERSION}" ]; then
  case "${ZSH_EVAL_CONTEXT}" in *:file) : ;; *) not_sourced=0 ;; esac
elif [ -n "${BASH_VERSION:-}" ]; then
  (return 0 2>/dev/null) || not_sourced=0
else
  case "${0##*/}" in sh|dash|ksh) : ;; *) not_sourced=0 ;; esac
fi

if [ "${not_sourced:-1}" = "0" ]; then
  _log FAIL "This script should be sourced into the current shell."
  _log FAIL "  Run '. $0' instead to try again."
  exit 1
fi

# Termux is a nightmare to build massive libraries on, so default to using what
# is in the system repositories. If this doesn't match the version in cargo, that
# is tough luck for now. I probably should use proot-distro instead moving forwards.
if pwd | grep -qE '^/data/data/com.termux(/.*)?$'; then
  _log INFO "Preparing for Termux. Consider using proot distro instead."
  if ! _command_exists llvm-config; then
    _log WARN "Installing LLVM first..."
    if ! time apt install libllvm; then
      _log FAIL "Failed to install llvm."
      exit 1
    fi
  fi

  _log INFO "Using libllvm v$(llvm-config --version) at $(llvm-config --prefix)"
  _llvm_envvar_version=$(llvm-config --version | grep -Eo '^[0-9]+\.[0-9]+' | sed 's/\.//g')
  _llvm_prefix_path=$(llvm-config --prefix)
  export "LLVM_SYS_${_llvm_envvar_version}_PREFIX=${_llvm_prefix_path}"
  return 0
fi

if ! _command_exists llvmenv; then
  _log WARN "Installing llvmenv crate first..."
  if ! cargo install llvmenv; then
    _log FAIL "cargo install failed."
    return 1
  fi
fi

if ! _command_exists llvmenv; then
  _log ERROR "Cargo binary path is probably misconfigured."
  _log ERROR "  Try adding ~/.cargo/bin to your \$PATH."
  return 1
fi

_log INFO "Initializing llvmenv..."
llvmenv init
_log WARN "Nothing else is implemented just yet."
return 1
