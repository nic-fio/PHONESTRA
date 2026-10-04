#!/usr/bin/env bash
# publish.sh - costruisce il sito di Phonestra in site/public/ e lo pubblica su
# https://phonestra.nicfio.it (la VPS del proprietario, utente «progetti»,
# /srv/www/phonestra.nicfio.it).
#
#   site/publish.sh            costruisce e pubblica
#   site/publish.sh --build    costruisce soltanto (guardare prima site/public/)
#
# Il sito: la pagina iniziale (site/landing/index.html), i due manuali, la
# licenza come pagina e i file della versione corrente (download/: l'AppImage,
# SHA256SUMS, la licenza). L'AppImage viene dalla Release su GitHub (tenuta in
# target/release-v<versione>/), o da target/appimage/ (make dist) con
# PHONESTRA_FROM_BUILD=1.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=$ROOT/site/public
HOST=progetti@57.131.27.241
DEST=/srv/www/phonestra.nicfio.it
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)
URL=https://phonestra.nicfio.it
APPIMAGE=Phonestra-$VERSION-x86_64.AppImage

rm -rf "$OUT"
mkdir -p "$OUT/download"

# la pagina iniziale (dal mockup 12, «Vivid») porta @VERSION@ al posto della versione
sed "s/@VERSION@/$VERSION/g" "$ROOT/site/landing/index.html" > "$OUT/index.html"
cp "$ROOT/docs/User Manual.html" "$ROOT/docs/Technical Manual.html" "$OUT/"
python3 "$ROOT/site/licence-page.py" "$ROOT/LICENSE.md" > "$OUT/licence.html"

# indirizzo canonico, anteprima dei link e un titolo adatto ai motori di ricerca
SEO=$ROOT/site/seo-head.py
python3 "$SEO" "$OUT/index.html" "$URL/" \
    "Phonestra — Android apps on your Linux desktop" \
    "Phonestra opens the apps of your Android phone in Linux desktop windows, over Wi-Fi: mouse and keyboard, sound, clipboard, notifications and files. Nothing to install on the phone."
python3 "$SEO" "$OUT/User Manual.html" "$URL/User%20Manual.html" \
    "Phonestra User Manual — Android phone apps in Linux windows" \
    "How to use Phonestra: connect an Android phone over Wi-Fi and use its apps in Linux desktop windows, with sound, keyboard, clipboard, notifications and files."
python3 "$SEO" "$OUT/Technical Manual.html" "$URL/Technical%20Manual.html" \
    "Phonestra Technical Manual — how Android apps reach the Linux desktop" \
    "How Phonestra is built: its ADB client, the component on the phone, video, audio, input, the drawer and the AppImage."
python3 "$SEO" "$OUT/licence.html" "$URL/licence.html" \
    "Phonestra Freeware Licence" \
    "Phonestra is freeware: free to use, also at work, and to give away unchanged; not to sell, modify or include in commercial products."

# i file della versione corrente, con la licenza della versione stessa
DL=$OUT/download
if [ "${PHONESTRA_FROM_BUILD:-}" = 1 ]; then
    cp "$ROOT/target/appimage/$APPIMAGE" "$DL/"
    sed 's/$/\r/' "$ROOT/LICENSE.md" > "$DL/LICENSE.txt"
    sed 's/$/\r/' "$ROOT/NOTICE.md" > "$DL/NOTICE.txt"
else
    CACHE=$ROOT/target/release-v$VERSION
    if [ ! -f "$CACHE/$APPIMAGE" ]; then
        mkdir -p "$CACHE"
        gh release download "v$VERSION" --repo nic-fio/PHONESTRA --dir "$CACHE" --clobber
        git -C "$ROOT" show "v$VERSION:LICENSE.md" > "$CACHE/LICENSE.md"
        git -C "$ROOT" show "v$VERSION:NOTICE.md" > "$CACHE/NOTICE.md"
    fi
    cp "$CACHE/$APPIMAGE" "$DL/"
    sed 's/$/\r/' "$CACHE/LICENSE.md" > "$DL/LICENSE.txt"
    sed 's/$/\r/' "$CACHE/NOTICE.md" > "$DL/NOTICE.txt"
fi
chmod +x "$DL/$APPIMAGE"
(cd "$DL" && sha256sum "$APPIMAGE" > SHA256SUMS)

# motori di ricerca e anteprime dei link
cp "$ROOT/site/og.png" "$OUT/og.png"
# prova di proprietà per Google Search Console: deve restare sul sito
cp "$ROOT"/site/google*.html "$OUT/" 2>/dev/null || true
cat > "$OUT/robots.txt" <<EOF
User-agent: *
Allow: /
Sitemap: $URL/sitemap.xml
EOF
TODAY=$(date +%F)
{
    echo '<?xml version="1.0" encoding="UTF-8"?>'
    echo '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'
    for p in "" "User%20Manual.html" "Technical%20Manual.html" "licence.html"; do
        echo "  <url><loc>$URL/$p</loc><lastmod>$TODAY</lastmod></url>"
    done
    echo '</urlset>'
} > "$OUT/sitemap.xml"

echo "costruito $OUT (versione $VERSION):"
(cd "$OUT" && find . -type f -printf '  %p  %s byte\n' | sort)

[ "${1:-}" = --build ] && exit 0

# il server diventa identico a site/public (niente rsync da nessuna delle due parti)
tar -C "$OUT" -cf - --mode=u=rwX,go=rX . | ssh "$HOST" "find '$DEST' -mindepth 1 -delete && tar -C '$DEST' -xf - --no-same-owner"
echo "pubblicato su $URL"
