# shellcheck shell=sh
###
### Initialises the current shell to use LLVM correctly, best-effort.
###
### This script must be sourced rather than executed directly.
###

_build_dir=./.llvm-build

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

# $0 isn't set consistently, grr.
if ! [ -e ./llvmc/prepare-env.sh ]; then
  _log FAIL "Run this script from the root of the repository."
  return 1
fi

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

  _llvm_envvar_version=$(llvm-config --version | grep -Eo '^[0-9]+\.[0-9]+' | sed 's/\.//g')
  _llvm_prefix_path=$(llvm-config --prefix)

  if [ -z "${_prepare_env_already_sourced:-}" ]; then
    # Change the prompt so we know we are sourced in a modified environment.
    PS1="llvm ($(llvm-config --version)) ${PS1}"; export PS1
    export "LLVM_SYS_${_llvm_envvar_version}_PREFIX=${_llvm_prefix_path}"
    _prepare_env_already_sourced=yes
  fi

else
  _log INFO "Using source-built LLVM."

  _llvm_version=$(cat .llvm-version)
  _build_dir=$(pwd -P)/.llvm-build

  cd llvmc || exit 1
  ./llvmc.bash -C lld -C lldb -d "${_build_dir}" -v "llvmorg-${_llvm_version}"
  _return_code=$?
  cd .. || exit 1

  if [ ${_return_code} -gt 0 ]; then
    _log FAIL "Failed to run llvmc successfully."
    return "${_return_code}"
  fi

  if [ -z "${_prepare_env_already_sourced:-}" ]; then
    # Change the prompt so we know we are sourced in a modified environment.
    PS1="llvm (${_llvm_version}) ${PS1}"; export PS1
    export "LLVM_SYS_${_llvm_envvar_version}_PREFIX=${_build_dir}/llvmorg-${_llvm_version}-build"
    export "PATH=${_build_dir}/llvmorg-${_llvm_version}-build/bin:${PATH}"
    _prepare_env_already_sourced=yes
  fi
fi
