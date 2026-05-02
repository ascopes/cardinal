# shellcheck shell=bash
# -*- coding: utf-8 -*-

export PS4='+ ($0:$LINENO) '

function log() {
  local date; date=$(date +%H:%M:%S)
  local line function file
  read -r line function file < <(caller 1)
  local location; location="($(basename "${file}"):${line}) (${function})"

  if [[ (-t 2 || -v CI ) && ! -v NOCOLOR ]]; then
    printf "\e[2;37m[%s] %s\e[0m \e[1;%sm%s:\e[0;%sm %s\e[0m\n" "${date}" "${location}" "${1}" "${2}" "${1}" "${3}" >&2
  else
    printf "[%s] %s %s: %s\n" "${date}" "${location}" "${2}" "${3}" >&2
  fi
}

function info() {
  log 32 INFO "${*}"
}

function note() {
  log '2;37' NOTE "${*}"
}

function error() {
  log 31 ERROR "${*}"
}

function separator() {
  printf "\n" >&2
  local i
  shopt -s checkwinsize

  # needed to refresh the window size details. Cannot be a bash builtin.
  /bin/true

  # columns may remain unset if we are not in a tty. Just fudge it to 60 chars if so.
  for ((i = 0; i < ${COLUMNS:-60}; ++i)); do
    printf "=" >&2
  done
  printf "\n\n" >&2
}

function run_or_abort() {
  log 33 "EXEC" "${*}"

  if "$@"; then
    local status=0
  else
    local status=${?}
    note "Command failed miserably with code ${status}..."
    abort "${status}"
  fi
}

function check_non_empty() {
  local name=$1
  shift 1
  if (( $# == 0 )); then
    error "At least one value for ${name@Q} must be provided."
    usage
    exit 1
  fi
}

function check_required_args() {
  local missing_args=()
  for required_arg in "${@}"; do
    if [[ ! -v ${required_arg} ]]; then
      missing_args+=("${required_arg}")
    fi
  done

  if ((${#missing_args[@]} > 0)); then
    error "Missing required arguments: ${missing_args[*]}."
    usage
    abort 1
  fi
}

function abort() {
  local exit_code=$1
  error "Aborting script with code ${exit_code}..."
  local line function file
  local caller_frame=0
  while read -r line function file < <(caller "${caller_frame}"); do
    error "      in ${function} at ${file}:${line}"
    ((caller_frame+=1))
  done
  exit "${exit_code}"
}

# Always abort this shell script if we interrupt, terminate, or hang up.
trap 'abort "$?"' INT TERM HUP