#!/bin/sh
# compila <nome> <indirizzo del tarball> [opzioni meson…]: scarica, compila e
# installa in $PREFISSO, poi cancella i sorgenti.
set -eu
nome=$1; indirizzo=$2; shift 2
mkdir -p "/sorgenti/$nome" && cd "/sorgenti/$nome"
curl -sSfL "$indirizzo" | tar -xJ --strip-components=1
meson setup _build --prefix="$PREFISSO" --buildtype=release "$@"
ninja -C _build
ninja -C _build install
cd / && rm -rf "/sorgenti/$nome"
ldconfig
