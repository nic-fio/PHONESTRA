#!/bin/sh
# Prepara una copia appena clonata su una macchina Debian o Ubuntu.
#
# Un clone porta con sé sorgenti, prove, manuali, il componente del telefono
# già compilato (android/phonestra-helper.jar) e tutta la storia, ma non:
# i pacchetti che servono a compilare, Rust (si installa nella cartella
# utente con rustup), l'identità git di questo repository (sta in
# .git/config, che non viene clonato) e strumenti/r8.jar (serve solo per
# ricompilare il componente). Questo script dice cosa manca e, con
# --install, installa i pacchetti con apt.
#
#     tools/setup-dev.sh            # dice cosa manca
#     tools/setup-dev.sh --install  # lo installa (chiede sudo)
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

# pacchetto : un comando che fornisce ("-" = chiedere a dpkg) : a cosa serve
PACKAGES="
build-essential:gcc:compilatore C e linker (libusb, ring)
pkg-config:pkg-config:trovare GTK, libadwaita e GStreamer
libgtk-4-dev:-:GTK4, l'interfaccia
libadwaita-1-dev:-:libadwaita, lo stile GNOME
libgstreamer1.0-dev:-:GStreamer, video e audio
libgstreamer-plugins-base1.0-dev:-:GStreamer app (appsrc)
libudev-dev:-:libusb, il telefono col cavo
gstreamer1.0-plugins-bad:-:decodifica H.264/H.265 (videoparsersbad)
gstreamer1.0-libav:-:decodifica video e audio (avdec)
gstreamer1.0-gtk4:-:gtk4paintablesink, il video nelle finestre
python3:python3:i manuali (docs/sources/build.py)
python3-pygments:-:colori del codice nei manuali
python3-pil:-:il logo nella copertina dei manuali
make:make:i comandi make, make test, make docs
git:git:il repository
curl:curl:installare Rust con rustup
"

# Facoltativi: servono solo per alcune attività.
OPTIONAL="
default-jdk-headless:javac:ricompilare il componente del telefono (make helper)
podman:podman:costruire l'AppImage (make dist)
adb:adb:solo diagnosi che phonestra-prova non sa fare
"

NAME=${PHONESTRA_GIT_NAME:-nic-fio}
EMAIL=${PHONESTRA_GIT_EMAIL:-315794250+nic-fio@users.noreply.github.com}

installed() {
    if [ "$2" = "-" ]; then
        dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "ok installed"
    else
        command -v "$2" >/dev/null 2>&1
    fi
}

missing=
check() {
    while IFS=: read -r pkg cmd why; do
        [ -z "$pkg" ] && continue
        if installed "$pkg" "$cmd"; then ok=yes; else ok=no; fi
        [ "$ok" = no ] && [ "$1" = required ] && missing="$missing $pkg"
        printf '  [%s] %-34s %s\n' "$([ "$ok" = yes ] && echo ' ok ' || echo MANC)" "$pkg" "$why"
    done
}

echo "Pacchetti:"
check required <<LIST
$PACKAGES
LIST

if [ -n "$missing" ]; then
    if [ "${1:-}" = "--install" ]; then
        sudo apt-get update
        # shellcheck disable=SC2086
        sudo apt-get install -y --no-install-recommends $missing
    else
        echo
        echo "  sudo apt-get install -y --no-install-recommends$missing"
    fi
else
    echo "  non manca niente"
fi

echo
echo "Facoltativi (non si installano da soli):"
check optional <<LIST
$OPTIONAL
LIST

echo
echo "Rust:"
if [ -x "$HOME/.cargo/bin/cargo" ] || command -v cargo >/dev/null 2>&1; then
    PATH=$HOME/.cargo/bin:$PATH
    echo "  $(cargo --version)"
    if ! cargo clippy --version >/dev/null 2>&1; then
        echo "  manca clippy: rustup component add clippy"
    fi
else
    echo "  manca: si installa nella cartella utente, senza sudo:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal -c clippy -c rustfmt"
fi

echo
echo "Identità git di questa copia:"
if git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
    if [ -z "$(git -C "$ROOT" config --local --get user.email || true)" ]; then
        git -C "$ROOT" config --local user.name "$NAME"
        git -C "$ROOT" config --local user.email "$EMAIL"
        echo "  impostata a $NAME <$EMAIL>"
        echo "  (tiene l'indirizzo personale fuori dalla storia pubblica;"
        echo "   PHONESTRA_GIT_NAME e PHONESTRA_GIT_EMAIL la cambiano)"
    else
        echo "  già impostata: $(git -C "$ROOT" config --local --get user.name) <$(git -C "$ROOT" config --local --get user.email)>"
    fi
else
    echo "  non è una copia git, saltata"
fi

echo
echo "Fuori dal repository (si ricreano, nessuno contiene dati da salvare):"
if [ -f "$ROOT/strumenti/r8.jar" ]; then
    echo "  [ ok ] strumenti/r8.jar"
else
    echo "  [MANC] strumenti/r8.jar    solo per ricompilare il componente: comando e impronta in android/helper/build.sh"
fi
echo "  ~/.config/Phonestra (chiave ADB e telefoni associati) è personale: su un PC nuovo"
echo "  si riassocia il telefono con «Aggiungi telefono»."

cat <<'END'

Poi verifica che tutto funzioni davvero:

    make            # cargo build: phonestra e phonestra-prova
    make test       # cargo test, compreso il controllo dei manuali
    make clippy     # nessun avviso ammesso
    ./target/debug/phonestra
END
