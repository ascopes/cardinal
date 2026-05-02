#!/usr/bin/env bash
# -*- coding: utf-8 -*-
set -o errexit
set -o nounset

# shellcheck source=./llvmc/llvmc-common.bash
source "$(dirname "${BASH_SOURCE[0]}")/llvmc-common.bash"

function container_cli() {
  local command
  for command in docker podman; do
    if command -v "${command}" &>/dev/null; then
      "${command}" "${@}"
      return "$?"
    fi
  done

  error "Cannot find any container development environment. Fix that and try again."
  abort 2
}

function container_name() {
  printf "local/llvm-build-environment:latest"
}

#
# Create the build environment container.
#
function build_container() {
  local optional_args=()
  [[ -v rebuild_container ]] && optional_args+=(--no-cache)

  if container_cli run --rm "$(container_name)" true && [[ ! -v rebuild_container ]]; then
    info "Container already exists, so will not rebuild..."
  else
    run_or_abort container_cli build "${optional_args[@]}" --file "$(dirname "${BASH_SOURCE[0]}")/Containerfile" --progress=plain --tag "$(container_name)" .
  fi
}

function call_llvmc_build() {
  local uid; uid=$(id -u "${USER}")
  local gid; gid=$(id -g "${USER}")

  local container_cli_args=(
    container_cli
    run
    --entrypoint /scripts/llvmc-build.bash
    --rm
    --tmpfs "/tmp:rw,uid=${uid},gid=${gid},nodev,noexec,nosuid,mode=1777,size=128m"
    --user "${uid}:${gid}"
    --volume "${build_directory}:/build"
    --volume "$(pwd):/scripts"
    --workdir "/build"
  )

  if [[ -t 1 || -t 2 ]]; then
    container_cli_args+=(--interactive --tty)
  fi

  # Bypass SELinux if it is enforcing, otherwise exec will fail.
  if [[ $(getenforce 2>/dev/null || echo) = "Enforcing" ]]; then
    container_cli_args+=(--security-opt label:disable)
  fi

  container_cli_args+=("$(container_name)")

  local llvmc_args=(-v "${version}")
  if [[ -v parallelism ]]; then llvmc_args+=(-p "${parallelism}"); fi
  if [[ -v reclone_llvm ]]; then llvmc_args+=(-c); fi
  if [[ -v rebuild_llvm ]]; then llvmc_args+=(-R); fi
  if [[ -v verbose ]]; then llvmc_args+=(-x); fi

  local component
  for component in "${components[@]}"; do
    llvmc_args+=(-C "${component}")
  done

  run_or_abort "${container_cli_args[@]}" "${llvmc_args[@]}"
}

function ensure_build_directory_exists() {
  if [[ ! -d "${build_directory}" ]]; then
    run_or_abort mkdir -pv "${build_directory}"
  fi
  build_directory=$(cd "${build_directory}" && pwd)
}

function usage() {
  echo "USAGE: ${BASH_SOURCE[0]} [-c] { -C <component> } -d <build_directory> [-h] [-p <parallelism>] [-r] [-R] -v <version> [-x]"
  echo "Build LLVM distributions in a container."
  echo ""
  echo "Arguments:"
  echo "    -c                     Always reclone the LLVM project first (implies -R)."
  echo "    -C <component>         Include the given component in the build (e.g. libc, libcxx, clang, llvm, lld, lldb)."
  echo "    -d <build_directory>   Set the directory to build in."
  echo "    -h                     Show this message, then exit."
  echo "    -p                     Control the concurrency."
  echo "    -r                     Always rebuild the development container first."
  echo "    -R                     Always rebuild the LLVM project from scratch."
  echo "    -v <version>           Set the version of LLVM to build from source. This should be a tag or branch."
  echo "    -x                     Enable verbose logging."
  echo ""
}

components=()

while getopts ":cC:d:hp:rRv:x" opt; do
  case "${opt}" in
    c) reclone_llvm= ;;
    C) components+=("${OPTARG}") ;;
    d) build_directory="${OPTARG}" ;;
    h) usage; exit 0 ;;
    p) parallelism=${OPTARG} ;;
    r) rebuild_container= ;;
    R) rebuild_llvm= ;;
    v) version=${OPTARG} ;;
    x) verbose= ; set -o xtrace ;;
    :) error "Argument -${OPTARG} needs a parameter."; usage; abort 1 ;;
    ?) error "Unknown argument -${OPTARG@Q}."; usage; abort 1 ;;
  esac
done

check_required_args build_directory version
check_non_empty components "${components[@]}"
ensure_build_directory_exists

separator
build_container

separator
call_llvmc_build