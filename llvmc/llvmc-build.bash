#!/usr/bin/env bash
# -*- coding: utf-8 -*-
set -o errexit
set -o nounset

# Core: AArch64;AMDGPU;ARM;AVR;BPF;Hexagon;Lanai;LoongArch;Mips;MSP430;NVPTX;PowerPC;RISCV;Sparc;SPIRV;SystemZ;VE;WebAssembly;X86;XCore
targets=(
  AArch64
  X86
)

# shellcheck source=./llvmc/llvmc-common.bash
source "$(dirname "${BASH_SOURCE[0]}")/llvmc-common.bash"

function maybe_clone_llvm() {
  if [[ -v reclone_llvm ]] && [[ -d ${src_dir} ]]; then
    info "Found existing LLVM sources. Destroying them first..."
    rm -Rf "${src_dir}"
    separator
  fi

  if [[ ! -d ${src_dir} ]]; then
    info "Will clone LLVM ${version} now"
    run_or_abort mkdir -vp "${src_dir}"
    clone_llvm
  else
    info "LLVM ${version} is already cloned."
  fi
}

function clone_llvm() {
  info "Cloning LLVM"

  run_or_abort git -c advice.defaultBranchName=false -C "${src_dir}" init
  run_or_abort git -C "${src_dir}" remote add origin https://github.com/llvm/llvm-project.git
  run_or_abort git -C "${src_dir}" fetch --atomic --depth=1 --force --jobs "${parallelism}" origin "${version}"
  run_or_abort git -C "${src_dir}" reset --hard FETCH_HEAD
  run_or_abort git -C "${src_dir}" submodule update --depth=1 --force --init --jobs "${parallelism}" --recursive
}

function maybe_build_llvm() {
  if [[ -v rebuild_llvm || -v reclone_llvm ]] && [[ -d ${build_dir} ]]; then
    info "Found existing LLVM build outputs. Destroying them first..."
    rm -Rf "${build_dir}"
  fi

  if [[ ! -d ${build_dir} ]]; then
    info "Will build LLVM ${version} now"
    run_or_abort mkdir -vp "${build_dir}"
    build_llvm
  else
    info "LLVM ${version} is already built."
  fi
}

function build_llvm() {
  separator
  info "Building ${components[*]}..."
  local component_build_dir="${build_dir}"
  run_or_abort mkdir -vp "${component_build_dir}"

  cmake_args=(
    -DLLVM_BUILD_TOOLS=OFF
    -DLLVM_ENABLE_BINDINGS=OFF
    -DLLVM_ENABLE_PROJECTS="$(IFS=';'; echo "${components[*]}")"
    -DLLVM_INCLUDE_EXAMPLES=OFF
    -DLLVM_INCLUDE_TESTS=OFF
    -DLLVM_INCLUDE_TOOLS=ON
    -DLLVM_OPTIMIZED_TABLEGEN=ON
    -DLLVM_TARGETS_TO_BUILD="$(IFS=';'; echo "${targets[*]}")"
    -DLLVM_USE_LINKER=mold
    -DLLVM_USE_SPLIT_DWARF=ON
  )

  export CMAKE_BUILD_PARALLEL_LEVEL=${parallelism}
  export CMAKE_BUILD_TYPE=${release_type}

  run_or_abort cmake -B "${component_build_dir}" -G "Unix Makefiles" -S "${src_dir}/llvm" "${cmake_args[@]}"

  separator

  run_or_abort cmake --build "${component_build_dir}"
}

function usage() {
  echo "USAGE: ${BASH_SOURCE[0]} [-c] { -C <component> } [-h] [-p <parallelism>] [-r <type>] [-R] -v <version> [-x]"
  echo "Build LLVM distributions."
  echo ""
  echo "Arguments:"
  echo "    -c              Always re-clone LLVM (implies -R as well)."
  echo "    -C <component>  Include the given component in the build (e.g. libc, libcxx, clang, llvm, lld, lldb)."
  echo "    -h              Show this message, then exit."
  echo "    -p              Control the concurrency."
  echo "    -r <type>       Set the CMAKE release type. Defaults to 'MinSizeRel' if unset. Can be 'Debug' or 'Release'."
  echo "    -R              Always rebuild LLVM."
  echo "    -v <version>    Set the version of LLVM to build from source."
  echo "    -x              Enable verbose logging."
  echo ""
}

components=()
parallelism=$(nproc)
release_type=MinSizeRel

while getopts ":cC:hp:r:Rv:x" opt; do
  case "${opt}" in
    c) reclone_llvm= ;;
    C) components+=("${OPTARG}") ;;
    h) usage; exit 0 ;;
    p) parallelism=${OPTARG} ;;
    r) release_type=${OPTARG} ;;
    R) rebuild_llvm= ;;
    v) version=${OPTARG} ;;
    x) set -o xtrace ;;
    :) error "Argument -${OPTARG} needs a parameter."; usage; abort 1 ;;
    ?) error "Unknown argument -${OPTARG@Q}."; usage; abort 1 ;;
  esac
done

check_required_args version

separator
src_dir=${version}-src
build_dir=${version}-build
maybe_clone_llvm

separator
maybe_build_llvm
