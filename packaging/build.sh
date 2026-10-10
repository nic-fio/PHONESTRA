#!/bin/sh
# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
# compila <nome> <indirizzo del tarball> [opzioni meson…]: scarica, compila e
# installa in $PREFISSO, poi cancella i sorgenti. I file della licenza restano in
# $PREFISSO/share/licenses/<nome>/: collect.sh li mette nell'AppImage.
set -eu
nome=$1; indirizzo=$2; shift 2
mkdir -p "/sorgenti/$nome" && cd "/sorgenti/$nome"
curl -sSfL "$indirizzo" | tar -xJ --strip-components=1
meson setup _build --prefix="$PREFISSO" --buildtype=release "$@"
ninja -C _build
ninja -C _build install
mkdir -p "$PREFISSO/share/licenses/$nome"
for f in COPYING* LICENSE* LICENCE* LICENSES; do
    [ -e "$f" ] && cp -r "$f" "$PREFISSO/share/licenses/$nome/"
done
[ -n "$(ls "$PREFISSO/share/licenses/$nome")" ] || { echo "$nome: nessun file di licenza"; exit 1; }
cd / && rm -rf "/sorgenti/$nome"
ldconfig
