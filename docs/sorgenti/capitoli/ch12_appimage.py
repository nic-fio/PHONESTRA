from build import c, flow, p, steps, table, term, warn

S1 = p("Un solo file che parte su tutte le distribuzioni diffuse: si compila su una glibc vecchia e porta con sé GTK, "
       "libadwaita e GStreamer, ma usa i driver grafici del sistema.", lead=True) + \
    p(c("costruzione/Containerfile") + " parte da Ubuntu 22.04 (glibc 2.35), così l'eseguibile chiede al massimo "
      "glibc 2.34 e parte anche sui sistemi del 2022. GTK e libadwaita di Ubuntu 22.04 sono troppo vecchie: il "
      "contenitore compila in " + c("/opt/phonestra") + " wayland 1.22, wayland-protocols 1.36, glib 2.80, graphene "
      "1.10, GTK 4.14 e libadwaita 1.5 (" + c("compila.sh") + ", meson), più " + c("gst-plugin-gtk4") + " 0.13.5 da "
      "gst-plugins-rs.") + \
    term("""
$ podman build -t phonestra-appimage costruzione
$ podman run --rm -v .:/phonestra:Z phonestra-appimage sh -c 'CARGO_TARGET_DIR=target/appimage cargo build --release && costruzione/raccogli.sh'
""", "Costruire l'AppImage") + \
    warn(c("raccogli.sh") + " da solo impacchetta l'eseguibile che trova in " + c("target/appimage/release")
         + ", anche se è vecchio: dopo ogni cambiamento del codice va rifatto prima " + c("cargo build --release")
         + " nel contenitore (il 28 set 2026 è stata impacchettata una versione vecchia).", "Sempre tutti e due i passi.") + \
    p("Il risultato è " + c("target/appimage/Phonestra-<versione>-x86_64.AppImage") + "; la versione viene da "
      + c("Cargo.toml") + ".")

S2 = flow([("Eseguibile", "e librerie con ldd", "navy"), ("Plugin", "GStreamer, immagini, GIO", "blue"),
           ("Riserva", "usr/lib/riserva", "blue"), ("patchelf", "$ORIGIN", "blue"),
           ("appimagetool", "runtime statico", "light")],
          "«FIG» — I passi di " + c("raccogli.sh")) + steps([
    "Copia l'eseguibile e, ricorsivamente con " + c("ldd") + ", le librerie che usa, tranne quelle che devono venire "
    "dal sistema: glibc, " + c("libstdc++") + ", " + c("libgcc_s") + ", driver grafici e librerie che li caricano (GL, "
    "EGL, DRM, gbm, Vulkan), " + c("libX11") + " e " + c("libxcb") + ", fontconfig, freetype, " + c("libexpat")
    + ", PipeWire, ALSA, udev.",
    "Copia i plugin di GStreamer che servono (" + c("coreelements") + ", " + c("app") + ", "
    + c("typefindfunctions") + ", " + c("playback") + ", " + c("videoparsersbad") + ", " + c("videoconvert") + ", "
    + c("videoscale") + ", " + c("libav") + ", " + c("opus") + ", " + c("audioconvert") + ", " + c("audioresample")
    + ", " + c("autodetect") + ", " + c("pulseaudio") + ", " + c("isomp4") + ", " + c("vaapi") + ", " + c("va")
    + ") e " + c("gtk4") + ", con " + c("gst-plugin-scanner") + ".",
    "Copia i caricatori di immagini (PNG, JPEG, SVG), gli schemi GSettings, le icone Adwaita di riserva, la licenza "
    "e il modulo GIO di dconf: senza, GTK non legge le impostazioni del desktop (nella barra delle finestre restava "
    "solo la X).",
    "Sposta in " + c("usr/lib/riserva") + " le librerie che anche i driver del sistema usano (wayland, zlib, zstd, "
    "libxml2, libffi, libelf, libva, le " + c("libxcb-*") + "…): si usano solo se il sistema non le ha o le ha troppo "
    "vecchie.",
    "Sistema i percorsi con " + c("patchelf") + " (" + c("$ORIGIN") + "), aggiunge " + c("AppRun") + ", l'icona "
    "(" + c("grafica/icone/phonestra-256.png") + ") e un " + c(".desktop") + " con " + c("NoDisplay=true")
    + " (serve ad " + c("appimagetool") + ", non si installa), e crea l'AppImage col runtime statico (niente "
    + c("libfuse2") + ").",
])

S3 = p(c("costruzione/AppRun") + " prepara l'ambiente e lancia " + c("usr/bin/phonestra") + ".") + \
    table(["Impostazione", "Perché"], [
        ["Solo i plugin di GStreamer inclusi, registro in " + c("~/.cache/Phonestra"), "I plugin del sistema sono "
         "compilati per un'altra GStreamer."],
        ["Caricatori delle immagini coi percorsi di questo avvio", "Il file dei caricatori ha percorsi assoluti "
         "(" + c("@APPDIR@") + " sostituito all'avvio)."],
        ["Nessun modulo GIO del sistema", "Compilati per un'altra glib, rompono i programmi."],
        [c("GSK_RENDERER=gl"), "Il predefinito di GTK 4.14 sbaglia le sfumature coi Mesa meno recenti; chi lo imposta "
         "da sé vince."],
        ["Icone del sistema prima delle nostre", "Le nostre sono solo una riserva."],
        ["Librerie di riserva collegate in " + c("~/.cache/Phonestra/riserva") + " solo quando servono",
         "Per esempio " + c("libwayland-client") + " più vecchia della 1.21."],
    ], "«TAB» — Che cosa prepara AppRun")

S4 = p(c("costruzione/prova-distribuzioni.sh") + " avvia l'AppImage in contenitori Ubuntu 22.04, Debian 12, Fedora 43 "
       "e Arch, collegati allo schermo Wayland e alla scheda grafica del PC, con una configurazione vuota: Phonestra "
       "deve aprire «Aggiungi un telefono». Con " + c("PHONESTRA_FOTO") + " la finestra si salva in PNG e lo script "
       "riporta gli errori del registro.")

S5 = steps([
    "Versione in " + c("Cargo.toml") + " (per le candidate: " + c("1.0.0-rc.N") + "), " + c("python3 docs/sorgenti/build.py")
    + " (la versione compare nel manuale), commit e push.",
    "AppImage col contenitore, prova di avvio e di collegamento, impronta SHA-256.",
    "Tag annotato sul commit e " + c("gh release create … --prerelease") + " con l'AppImage e le note (novità, "
    "requisiti, impronta).",
    "Copia dell'AppImage nella home dell'utente (" + c("~/Phonestra-<versione>-x86_64.AppImage") + "): è quella che "
    "l'utente avvia. Su GitHub resta solo l'ultima candidata; le etichette git delle precedenti si tengono.",
])

CHAPTER = ("Costruire l'AppImage", [
    ("Il contenitore", S1),
    ("Cosa fa raccogli.sh", S2),
    ("AppRun", S3),
    ("Prova sulle distribuzioni", S4),
    ("Pubblicare una versione", S5),
])
