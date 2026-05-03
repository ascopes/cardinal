# shellcheck shell=sh
###
### Initialises the current shell to use LLVM correctly, best-effort.
###
### This script must be sourced rather than executed directly, and must be done
### from the root of this repository.
###

_build_dir=$(pwd -P)/.llvm
_llvm_version=$(cat .llvm-version) || return 1
_llvm_releases_url=https://github.com/llvm/llvm-project/releases/download/llvmorg-${_llvm_version}
_llvm_gpg_keys_url=https://releases.llvm.org/release-keys.asc

_log() {
  if [ ! -t 2 ] && [ ! -n "${CI:-}" ] || [ -n "${NOCOLOR:-}" ]; then
    printf "%s [[%s]] %s\n" "$(date "+%X")" "${1}" "${2}" >&2
  else
    case "${1}" in
      INFO) _color=32 ;;
      WARN) _color=33 ;;
      FAIL) _color=31 ;;
      *) _log FAIL "bad color ${1}"; exit 1 ;;
    esac
    printf "%s \033[1;${_color}m[[%s]] \033[0;${_color}m%s\033[0m\n" "$(date "+%X")" "${1}" "${2}" >&2
  fi
}

_command_exists() {
  command -v "${1}" > /dev/null 2>&1
}

_downcase() {
  tr '[:upper:]' '[:lower:]'
}

_curl() {
  curl --fail-with-body -L \
      -w '%{method} %{url} -> HTTP/%{http_version} %{http_code} %{content_type} %{size_download} %{errormsg} %{onerror}\n' \
      "${@}" \
      1>&2
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
  USE_SYSTEM_LLVM=true
fi

if [ -n "${USE_SYSTEM_LLVM:-}" ]; then
  _log INFO "Using system LLVM."
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

for command in curl gpg uname tar xz; do
  if ! _command_exists "${command}"; then
    _log FAIL "No executable ${command} found. Install it and try again."
    return 1
  fi
done

# Determine which LLVM release to download

_log INFO "Detecting OS and CPU architecture..."

case "$(uname -s | _downcase)" in
  linux|gnu/linux|cygwin)
    case "$(uname -m | _downcase)" in
      x86_64|amd64)
        _arch=X64
        ;;
      aarch64|arm64)
        _arch=ARM64
        ;;
      *)
        _log FAIL "Unsupported arch for Linux $(uname -m)"
        return 1
        ;;
    esac

    _release=LLVM-${_llvm_version}-Linux-${_arch}
    ;;

  darwin)
    case "$(uname -m | _downcase)" in
      aarch64|arm64)
        _arch=ARM64
        ;;
      *)
        _log FAIL "Unsupported arch for macOS $(uname -m)"
        return 1
        ;;
    esac

    _release=LLVM-${_llvm_version}-macOS-${_arch}
    ;;

  win*|mingw*|msys*)
    case "$(uname -m | _downcase)" in
      aarch64|arm64)
        _arch=aarch64
        ;;
      x86_64|amd64)
        _arch=x86_64
        ;;
      *)
        _log FAIL "Unsupported arch for Windows $(uname -m)"
        return 1
        ;;
    esac

    _release=clang+llvm-${_llvm_version}-${_arch}-pc-windows-msvc
    ;;

  *)
    _log FAIL "Unsupported OS $(uname -s)"
    return 1
    ;;
esac

# Fetch and extract the LLVM release if it does not already exist locally.
_log INFO "Will use LLVM ${_release} for this environment."

if ! [ -d "${_build_dir}" ]; then
  mkdir -p "${_build_dir}"
fi

_tarball=${_release}.tar.xz
_gpg_sig_file=${_tarball}.sig

# Fetch the release, verify, and extract if the extracted content does not yet exist...

if [ -d "${_build_dir}/${_release}" ]; then
  _log INFO "Found existing LLVM release in the local workspace"
else

  _cleanup() {
    cd "${_old_cwd}" || :
    trap - INT
  }

  _old_cwd=$(pwd -P)
  cd "${_build_dir}" || :
  trap _cleanup INT

  _log INFO "Importing GPG keys..."
  if ! _curl -o release.asc "${_llvm_gpg_keys_url}" || ! gpg --import release.asc; then
    _log FAIL "Failed to import gpg keys for LLVM"
    _cleanup
    return 1
  fi

  if ! [ -f "${_gpg_sig_file}" ]; then
    _log INFO "No local signatures for this LLVM version exists. Fetching from GitHub..."
    if ! _curl -O "${_llvm_releases_url}/${_gpg_sig_file}"; then
      _log WARN "Failed to download GPG signatures. Skipping verification..."
      rm "${_gpg_sig_file}"
      _no_gpg=true
    fi
  fi

  if ! [ -f "${_tarball}" ]; then
    _log INFO "No archive for this LLVM version exists. Fetching from GitHub..."
    if ! _curl -O "${_llvm_releases_url}/${_tarball}"; then
      rm -f "${_gpg_sig_file}" "${_tarball}"
      _log FAIL "Failed to download LLVM"
      _cleanup
      return 1
    fi
  fi

  if ! [ -n "${_no_gpg}" ]; then
    _log INFO "Verifying signature of downloaded LLVM release..."
    if ! gpg --verify "${_gpg_sig_file}" "${_tarball}"; then
      _log FAIL 'Signatures do not match!'
      _cleanup
      return 1
    fi
  fi

  _log INFO "Extracting LLVM archive..."
  if ! tar xf "${_tarball}"; then
    _log FAIL "Failed to extract tarball"
    _cleanup
    return 1
  fi

  _cleanup
fi

if [ "${LLVM_SOURCED:-}" != "${_release}" ]; then
  _log INFO "Configuring environment variables for this shell..."

  _llvm_envvar_version=$(echo "${_llvm_version}" | grep -Eo '^[0-9]+\.[0-9]+' | sed 's/\.//g')
  _llvm_prefix_path=${_build_dir}/${_release}

  export "PATH=${_build_dir}/${_release}/bin:${PATH}"
  export "LLVM_SYS_${_llvm_envvar_version}_PREFIX=${_llvm_prefix_path}"
  export "LLVM_SOURCED=${_release}"

  if [ -n "${GITHUB_ENV}" ]; then
    _log INFO "Exporting enviromment variables to subsequent GitHub Actions steps in this job."
    {
      echo "PATH=${_build_dir}/${_release}/bin:${PATH}"
      echo "LLVM_SYS_${_llvm_envvar_version}_PREFIX=${_llvm_prefix_path}"
      echo "LLVM_SOURCED=${_release}"
    } > "${GITHUB_ENV}"
  fi

  deactivate() {
    unset LLVM_SOURCED
    unset deactivate
  }

  _log INFO "Successfully initialised environment."
  return 0

else
  _log WARN "Not doing anything as this shell already has a sourced LLVM build. Start a fresh"
  _log WARN "  shell to force the setup again, or run 'deactivate'."
  return 1
fi
