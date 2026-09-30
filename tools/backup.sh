#!/bin/sh
# Impacchetta tutto il progetto in un solo file: un git bundle con ogni ramo,
# ogni tag e la storia completa. Si ripristina senza rete e senza GitHub:
#
#     tools/backup.sh ~/backup                    # scrive phonestra-AAAA-MM-GG.bundle
#     git clone phonestra-2026-09-30.bundle PHONESTRA
#     cd PHONESTRA && tools/setup-dev.sh --install && make test
#
# Non c'è niente di personale da aggiungere: ~/.config/Phonestra (chiave ADB e
# telefoni associati) di proposito non si salva, su un PC nuovo si riassocia
# il telefono. Restano fuori anche target/ e strumenti/r8.jar, che si
# ricreano. Le AppImage delle Release stanno su GitHub, non nel bundle.
set -eu

DEST=${1:-.}
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DAY=$(date +%F)
OUT="$DEST/phonestra-$DAY.bundle"

[ -d "$DEST" ] || mkdir -p "$DEST"
git -C "$ROOT" bundle create "$OUT" --all
git -C "$ROOT" bundle verify "$OUT" >/dev/null

if [ -n "$(git -C "$ROOT" status --porcelain)" ]; then
    echo "attenzione: le modifiche non ancora registrate NON sono nel bundle:"
    git -C "$ROOT" status --short
fi
if [ -n "$(git -C "$ROOT" log --oneline '@{upstream}..HEAD' 2>/dev/null)" ]; then
    echo "nota: il bundle ha commit che non sono ancora su GitHub."
fi

printf '%s: %s, %s commit, tag: %s\n' "$OUT" "$(du -h "$OUT" | cut -f1)" \
    "$(git -C "$ROOT" rev-list --count HEAD)" \
    "$(git -C "$ROOT" tag | tr '\n' ' ')"
