#!/usr/bin/env bash
set -eu

if [[ $# -lt 1 ]]; then
  echo "USAGE: $0 <target>"
  echo "Target is a target CPU architecture, such as i686-elf."
  exit 1
fi

export TARGET="${1}"
shift 1

if [[ -z ${JOBS+undef} ]]; then
  export JOBS=$(($(nproc) + 1))
fi

readonly this_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

function binutils() {
  export PREFIX="${this_dir}/binutils-${TARGET}"
  if [[ ! -d "${PREFIX}" ]]; then
    mkdir "${PREFIX}"
  fi
  pushd "${PREFIX}"
  time "${this_dir}/binutils/configure" \
      --target="${TARGET}" \
      --prefix="${PREFIX}" \
      --disable-nls \
      --disable-werror \
      --with-sysroot
  time make -j "${JOBS}"
  # Run sequentially to prevent bugs arising.
  time make install -B -j 1
  popd
}

function gcc() {
  export PATH="${this_dir}/binutils-${TARGET}/bin:$PATH"
  export PREFIX="${this_dir}/gcc-${TARGET}"
  if [[ ! -d "${PREFIX}" ]]; then
    mkdir "${PREFIX}"
  fi

  pushd "${this_dir}/gcc"
  time ./contrib/download_prerequisites
  popd

  pushd "${PREFIX}"
  time "${this_dir}/gcc/configure" \
      --target="${TARGET}" \
      --prefix="${PREFIX}" \
      --disable-multilib \
      --disable-nls \
      --enable-languages=c,c++ \
      --without-headers
  time make -j "${JOBS}" all-gcc
  time make -j "${JOBS}" all-target-libgcc
  # Run sequentially to prevent bugs arising.
  time make -B -j 1 install-gcc
  time make -B -j 1 install-target-libgcc
  popd
}

if [[ "$#" -eq 0 ]]; then
  binutils
  gcc
else
  for arg; do
    case "${arg}" in
      binutils)
        binutils
        ;;
      gcc)
        gcc
        ;;
      *)
        echo "Target ${arg} does not exist in this script." >&2
        exit 1
        ;;
    esac
  done
fi
