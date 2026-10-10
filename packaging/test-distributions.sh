#!/bin/bash
# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
# Prova l'AppImage su altre distribuzioni, in contenitori che usano lo schermo
# (Wayland) e la scheda grafica del PC. In ogni contenitore c'è solo quello che
# ha un desktop normale (driver grafici, caratteri); Phonestra parte con una
# configurazione vuota, quindi apre «Aggiungi un telefono» e non si collega a
# nessun telefono. Esito: l'immagine della finestra e gli errori del registro.
#
#   packaging/test-distributions.sh [cartella dei risultati]
set -uo pipefail
cd "$(dirname "$0")/.."
APPIMAGE=$(ls target/appimage/Phonestra-*-x86_64.AppImage | head -1)
USCITA=${1:-target/appimage/prove}
mkdir -p "$USCITA"

declare -A PACCHETTI=(
    [docker.io/library/ubuntu:22.04]="apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq libegl1 libgl1 libgbm1 libfontconfig1 fonts-dejavu-core libx11-6 libx11-xcb1 libxcb1 libudev1 libasound2 xkb-data libgles2 >/dev/null"
    [docker.io/library/debian:12]="apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq libegl1 libgl1 libgbm1 libfontconfig1 fonts-dejavu-core libx11-6 libx11-xcb1 libxcb1 libudev1 libasound2 xkb-data libgles2 >/dev/null"
    [registry.fedoraproject.org/fedora:43]="dnf install -y -q mesa-libEGL mesa-libGL mesa-dri-drivers fontconfig dejavu-sans-fonts libX11 libX11-xcb alsa-lib xkeyboard-config libglvnd-gles >/dev/null"
    [docker.io/library/archlinux:latest]="pacman -Sy --noconfirm --quiet mesa fontconfig ttf-dejavu libx11 alsa-lib xkeyboard-config >/dev/null"
)

for immagine in "${!PACCHETTI[@]}"; do
    nome=$(echo "$immagine" | sed 's|.*/||; s|:|-|')
    echo "== $nome"
    rm -rf "$USCITA/$nome" && mkdir -p "$USCITA/$nome/foto" "$USCITA/$nome/config"
    timeout 600 podman run --rm \
        --device /dev/dri \
        -e WAYLAND_DISPLAY="$WAYLAND_DISPLAY" -e XDG_RUNTIME_DIR=/tmp/xdg \
        -v "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/xdg/$WAYLAND_DISPLAY:ro" \
        -e APPIMAGE_EXTRACT_AND_RUN=1 -e XDG_CONFIG_HOME=/risultati/config -e XDG_CACHE_HOME=/tmp/cache \
        -e PHONESTRA_FOTO=/risultati/foto \
        -v "$PWD/$APPIMAGE:/Phonestra.AppImage:ro,Z" -v "$PWD/$USCITA/$nome:/risultati:Z" \
        --security-opt label=disable \
        "$immagine" sh -c "${PACCHETTI[$immagine]} && (timeout 15 /Phonestra.AppImage > /risultati/registro.txt 2>&1; true)"
    if [ -e "$USCITA/$nome/foto/prepara.png" ]; then
        echo "   finestra aperta"
    else
        echo "   NESSUNA FINESTRA"
    fi
    grep -iE "error|critical|symbol|not found|failed" "$USCITA/$nome/registro.txt" | grep -v "a11y" | sort -u | head -5
done
