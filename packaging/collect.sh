#!/bin/bash
# Raccoglie Phonestra e le sue librerie in un AppDir e ne fa un'AppImage
# (SPECIFICATION §2). Gira dentro il contenitore `phonestra-appimage`:
#
#   podman run --rm -v .:/phonestra:Z phonestra-appimage packaging/collect.sh
#
# Restano del sistema (non si includono): glibc, libstdc++ e libgcc_s (i driver
# del sistema vogliono le loro, più recenti), driver grafici e le librerie
# che li caricano (GL, EGL, DRM, gbm, Vulkan), X11/xcb, fontconfig e freetype
# (i caratteri seguono le impostazioni del PC), PipeWire (l'audio passa da
# PulseAudio). libva, libxcb-* e poche altre sono «di riserva» (vedi sotto). Nessun
# modulo GIO del sistema: compilati per un'altra glib, rompono i programmi
# (la lezione di Android-Dex, notes/user-decisions.md); dconf è incluso.
set -euo pipefail

RADICE=/phonestra
USCITA=$RADICE/target/appimage
APPDIR=$USCITA/Phonestra.AppDir
BINARIO=$USCITA/release/phonestra
[ -x "$BINARIO" ] || { echo "manca $BINARIO: prima cargo build --release nel contenitore"; exit 1; }

rm -rf "$APPDIR"
mkdir -p "$APPDIR"/usr/{bin,lib,libexec,share}

# Librerie da non includere (nomi, senza versione).
ESCLUSE='^(ld-linux.*|libc|libm|libstdc\+\+|libgcc_s|libdl|libpthread|librt|libresolv|libutil|libnsl|libanl|libBrokenLocale|libmvec|libGL|libGLX|libGLdispatch|libGLESv2|libEGL|libOpenGL|libgbm|libdrm|libdrm_.*|libvulkan|libglapi|libX11|libX11-xcb|libxcb|libXau|libXdmcp|libfontconfig|libfreetype|libexpat|libpipewire-0.3|libasound|libudev)\.so'

# Copia le librerie di cui ha bisogno un file (ricorsivamente, tramite ldd).
copia_dipendenze() {
    ldd "$1" 2>/dev/null | awk '/=> \// {print $3}' | while read -r lib; do
        nome=$(basename "$lib")
        if [[ "$nome" =~ $ESCLUSE ]] || [ -e "$APPDIR/usr/lib/$nome" ]; then
            continue
        fi
        cp -L "$lib" "$APPDIR/usr/lib/$nome"
        copia_dipendenze "$lib"
    done
}

echo "== eseguibile"
cp "$BINARIO" "$APPDIR/usr/bin/phonestra"
copia_dipendenze "$APPDIR/usr/bin/phonestra"

echo "== GStreamer"
GST=/usr/lib/x86_64-linux-gnu/gstreamer-1.0
mkdir -p "$APPDIR/usr/lib/gstreamer-1.0"
for p in coreelements app typefindfunctions playback videoparsersbad videoconvert videoscale \
         libav opus audioconvert audioresample autodetect pulseaudio isomp4 vaapi va; do
    if [ -e "$GST/libgst$p.so" ]; then
        cp "$GST/libgst$p.so" "$APPDIR/usr/lib/gstreamer-1.0/"
    else
        echo "   (plugin $p assente)"
    fi
done
cp /opt/phonestra/lib/gstreamer-1.0/libgstgtk4.so "$APPDIR/usr/lib/gstreamer-1.0/"
cp /usr/lib/x86_64-linux-gnu/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner "$APPDIR/usr/libexec/"
for f in "$APPDIR"/usr/lib/gstreamer-1.0/*.so "$APPDIR/usr/libexec/gst-plugin-scanner"; do
    copia_dipendenze "$f"
done

echo "== caricatori delle immagini (PNG, JPEG, SVG per le icone)"
PIXBUF=$(pkg-config --variable=gdk_pixbuf_moduledir gdk-pixbuf-2.0)
mkdir -p "$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders"
for l in png jpeg svg; do
    f=$(ls "$PIXBUF"/libpixbufloader-$l.so 2>/dev/null || true)
    [ -n "$f" ] && cp "$f" "$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders/" && copia_dipendenze "$f"
done
# Il file dei caricatori ha percorsi assoluti: AppRun li sistema all'avvio.
GDK_PIXBUF_MODULEDIR="$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders" \
    "$(pkg-config --variable=gdk_pixbuf_query_loaders gdk-pixbuf-2.0 2>/dev/null || echo /usr/lib/x86_64-linux-gnu/gdk-pixbuf-2.0/gdk-pixbuf-query-loaders)" \
    | sed "s|$APPDIR|@APPDIR@|g" > "$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders.cache.in"

echo "== schemi, icone di riserva, licenze"
mkdir -p "$APPDIR/usr/share/glib-2.0/schemas"
cp /opt/phonestra/share/glib-2.0/schemas/*.xml "$APPDIR/usr/share/glib-2.0/schemas/"
glib-compile-schemas "$APPDIR/usr/share/glib-2.0/schemas"
mkdir -p "$APPDIR/usr/share/icons"
cp -r /usr/share/icons/Adwaita "$APPDIR/usr/share/icons/"
cp /usr/share/icons/hicolor/index.theme "$APPDIR/usr/share/icons/" 2>/dev/null || true
mkdir -p "$APPDIR/usr/share/doc/phonestra"
cp "$RADICE/LICENSE.md" "$APPDIR/usr/share/doc/phonestra/"
# Moduli GIO: niente quelli del sistema (vedi sopra), solo la nostra copia di
# dconf. Senza, GSettings non legge le impostazioni del desktop dell'utente e
# GTK usa quelle predefinite: per esempio nella barra delle finestre restava
# solo la X, senza Riduci a icona e Ingrandisci (provato il 27 set 2026).
mkdir -p "$APPDIR/usr/lib/gio/modules"
cp /usr/lib/x86_64-linux-gnu/gio/modules/libdconfsettings.so "$APPDIR/usr/lib/gio/modules/"
copia_dipendenze "$APPDIR/usr/lib/gio/modules/libdconfsettings.so"

echo "== librerie di riserva (si usano solo se il sistema non le ha)"
# Queste le usano anche i driver del sistema (Mesa, VA-API): se ne caricassimo
# una copia più vecchia, un driver più nuovo non partirebbe (successo con
# libwayland-client 1.22 e Mesa di Debian 13). AppRun le usa solo se mancano
# nel sistema o sono troppo vecchie.
mkdir -p "$APPDIR/usr/lib/riserva"
for f in "$APPDIR"/usr/lib/lib{wayland-client,wayland-cursor,wayland-egl,wayland-server,z,zstd,xml2,ffi,elf,tinfo,edit,bz2,lzma,va,va-drm,va-wayland,va-x11}.so* "$APPDIR"/usr/lib/libxcb-*.so*; do
    [ -e "$f" ] && mv "$f" "$APPDIR/usr/lib/riserva/"
done
ls "$APPDIR/usr/lib/riserva"

echo "== percorsi relativi (rpath)"
patchelf --set-rpath '$ORIGIN/../lib' "$APPDIR/usr/bin/phonestra"
patchelf --set-rpath '$ORIGIN/../lib' "$APPDIR/usr/libexec/gst-plugin-scanner"
for f in "$APPDIR"/usr/lib/*.so* "$APPDIR"/usr/lib/riserva/*.so*; do patchelf --set-rpath '$ORIGIN' "$f" 2>/dev/null || true; done
for f in "$APPDIR"/usr/lib/gstreamer-1.0/*.so "$APPDIR"/usr/lib/gdk-pixbuf-2.0/loaders/*.so "$APPDIR"/usr/lib/gio/modules/*.so; do
    patchelf --set-rpath '$ORIGIN/..:$ORIGIN/../..' "$f" 2>/dev/null || true
done

echo "== AppRun, desktop, icona"
cp "$RADICE/packaging/AppRun" "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"
# Icona del file AppImage: il logo ufficiale senza scritta (logos/icons).
cp "$RADICE/logos/icons/phonestra-256.png" "$APPDIR/phonestra.png"
# Serve ad appimagetool; resta dentro l'AppImage e non si installa nel menu.
cat > "$APPDIR/phonestra.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Phonestra
Comment=Le app del telefono Android in finestre sul PC
Exec=phonestra
Icon=phonestra
Categories=Utility;
Terminal=false
NoDisplay=true
EOF

echo "== AppImage"
STRUMENTO=$USCITA/appimagetool
if [ ! -x "$STRUMENTO" ]; then
    curl -sSfL -o "$STRUMENTO" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x "$STRUMENTO"
fi
VERSIONE=$(grep -m1 '^version' "$RADICE/Cargo.toml" | cut -d'"' -f2)
ARCH=x86_64 "$STRUMENTO" --appimage-extract-and-run -n "$APPDIR" "$USCITA/Phonestra-$VERSIONE-x86_64.AppImage"
ls -la "$USCITA"/*.AppImage
du -sh "$APPDIR"
