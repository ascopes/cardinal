# thirdparty/cross-compiler

Tooling to build a valid cross compiler environment.

This builds binutils and the C compiler.

## Requirements

These packages will be named differently depending on the system you use:

- autoconf
- autogen
- gcc
- g++
- make
- bison
- flex
- gmp-devel (libgmp-dev)
- libmpc-devel (libmpc-dev)
- mpfr-devel (libmpfr-dev)
- texinfo
- isl-devel (libisl-dev)

## Building

Instructions are adapted from https://wiki.osdev.org/Building_GCC#Linux_Users_building_a_System_Compiler

1. Initialize the submodules:

```shell
$ git submodule update --init thirdparty/cross-compiler/binutils thirdparty/cross-compiler/gcc
```

2. Compile the tooling

```shell
$ ./thirdparty/cross-compiler/build.sh i686-elf
```

You can replace `i686-elf` with a different architecture if desired.

If compilation fails due to the wrong autoconf version in GCC, you can edit gcc/config/overrides.m4 to fix this:

```diff
-  [m4_define([_GCC_AUTOCONF_VERSION], [2.69])])
+  [m4_define([_GCC_AUTOCONF_VERSION], [2.71])])
```
