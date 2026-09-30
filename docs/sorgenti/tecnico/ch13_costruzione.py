from build import c, flow, note, p, rif, steps, table, term, tip, warn

S1 = p("Per compilare Phonestra, il suo componente e i manuali servono questi strumenti. Il sistema di riferimento è "
       "Debian 13 «trixie».", lead=True) + \
    table(["Strumento", "Per cosa", "Note"], [
        ["Rust stabile (edizione 2024)", "Tutto il PC", c("cargo") + " sta in " + c("~/.cargo/bin") + ": aggiungilo "
         "al " + c("PATH") + " nei comandi."],
        ["GTK 4.12+, libadwaita 1.5+, GStreamer 1.x (con " + c("-dev") + ")", "Compilare e avviare sul PC di sviluppo",
         "Plugin necessari: base, good, bad, " + c("gstreamer1.0-libav") + " (per " + c("avdec_aac") + ") e "
         + c("gst-plugin-gtk4") + " (" + c("gtk4paintablesink") + ")."],
        ["JDK (" + c("javac") + ")", "Compilare il componente", "Compilato con " + c("--release 11") + "; va bene un "
         "JDK recente."],
        ["D8 di R8 9.4.26", "Da classi Java a dex", "In " + c("strumenti/r8.jar") + " (non nel repository: indirizzo "
         "e impronta SHA-256 in " + c("telefono/aiuto/costruisci.sh") + ")."],
        [c("podman"), "Costruire l'AppImage", "Contenitore " + c("phonestra-appimage") + " ("
         + rif("Il contenitore dell'AppImage") + ")."],
        ["Python 3 con " + c("pygments") + " e Pillow", "Generare i due manuali", c("python3 docs/sorgenti/build.py")
         + " scrive " + c("docs/Phonestra_Manuale_Tecnico.html") + " e " + c("docs/Phonestra_Manuale_Utente.html")
         + "; pacchetti " + c("python3-pygments") + " e " + c("python3-pil") + " (Pillow serve al logo ufficiale "
         "della copertina). Lo usa anche " + c("cargo test") + "."],
    ], "«TAB» — Gli strumenti che servono sul PC") + \
    p("Per le prove serve un telefono con Android 14 o successivo, il Debug wireless acceso e il PC sulla stessa rete "
      "Wi-Fi.") + \
    note("tre cose stanno fuori dal repository. " + c("strumenti/r8.jar") + " si scarica con il comando scritto in "
         + c("costruisci.sh") + ". L'immagine del contenitore " + c("phonestra-appimage") + " si ricostruisce con "
         + c("podman build") + ". La configurazione di chi sviluppa (" + c("~/.config/Phonestra") + ": chiave ADB e "
         "telefoni associati) resta sul PC: su un PC nuovo il telefono va associato di nuovo. Le Release di GitHub "
         "con le AppImage si rifanno dal codice.", "Cosa non porta un clone.")

S2 = p("Pochi comandi coprono il lavoro di tutti i giorni; " + c("cargo test") + " controlla anche che i manuali siano "
       "allineati al codice.", lead=True) + term("""
$ export PATH=$HOME/.cargo/bin:$PATH
$ cargo build   # programma e strumento delle prove
$ cargo test   # prove sul PC, senza telefono; controlla anche i manuali
$ cargo clippy --all-targets   # nessun avviso ammesso
$ cargo run --bin phonestra   # avvia Phonestra dai sorgenti
$ telefono/aiuto/costruisci.sh   # ricompila il componente del telefono
$ python3 docs/sorgenti/build.py   # rigenera i due manuali
""", "Comandi di tutti i giorni") + \
    tip("cambi il codice → " + c("cargo test") + " (secondi) → se hai toccato il componente, " + c("costruisci.sh")
        + " e la prova " + c("phonestra-prova") + " del pezzo → Phonestra vero sul telefono. Prima di ogni commit: "
        + c("cargo build") + ", " + c("cargo test") + " e " + c("cargo clippy") + " senza avvisi.",
        "Il giro di tutti i giorni.") + \
    warn("i manuali si cambiano nei sorgenti di " + c("docs/sorgenti/") + ", mai nei file HTML di " + c("docs/")
         + ": la generazione successiva cancellerebbe la modifica, e " + c("cargo test") + " fallisce finché i file "
         "pubblicati non corrispondono ai sorgenti (" + rif("Il manuale e i suoi controlli") + ").",
         "I manuali sono generati.")

S3 = p("Il componente del telefono si compila fuori da " + c("cargo") + ", con uno script: " + c("javac") + ", poi D8. "
       "Il jar che ne esce va nel repository.", lead=True) + term("""
$ curl -L -o strumenti/r8.jar https://dl.google.com/android/maven2/com/android/tools/r8/9.4.26/r8-9.4.26.jar
$ telefono/aiuto/costruisci.sh
creato telefono/phonestra-aiuto.jar
""", "Compilare il componente") + \
    p("Lo script compila con " + c("javac --release 11") + " le finte classi Android di " + c("stub/") + " (solo "
      "firme: servono al compilatore e non finiscono nel jar; sul telefono ci sono quelle vere) e i sorgenti di "
      + c("src/phonestra/") + ", poi D8 li converte in dex con " + c("--min-api 34") + " e " + c("--lib")
      + " sul JDK.") + \
    p("Quando usi una classe Android pubblica nuova, aggiungi il suo stub con i soli metodi usati. Le API nascoste si "
      "chiamano per riflessione e di solito non hanno stub; fanno eccezione le classi nascoste che estendiamo, che il "
      "compilatore deve conoscere: " + c("android.app.TaskStackListener") + " (" + c("EventiApp.java") + ") e "
      + c("android.content.IOnPrimaryClipChangedListener") + " (" + c("Input.java") + ").") + \
    warn("dopo la compilazione il jar va committato insieme ai sorgenti: il programma lo incorpora con "
         + c("include_bytes!") + ", e un jar vecchio fa girare sul telefono il codice di prima.", "Il jar va nel commit.")

S4 = p("Un solo file che parte su tutte le distribuzioni diffuse: si compila su una glibc vecchia e porta con sé GTK, "
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

S5 = p(c("raccogli.sh") + " prende l'eseguibile compilato nel contenitore e ne fa un'AppImage, in cinque passi.",
       lead=True) + \
    flow([("Eseguibile", "e librerie con ldd", "navy"), ("Plugin", "GStreamer, immagini, GIO", "blue"),
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

S6 = p(c("costruzione/AppRun") + " prepara l'ambiente e lancia " + c("usr/bin/phonestra") + ": le librerie del "
       "sistema e quelle incluse non devono mescolarsi.", lead=True) + \
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

S7 = p(c("costruzione/prova-distribuzioni.sh") + " avvia l'AppImage in contenitori Ubuntu 22.04, Debian 12, Fedora 43 "
       "e Arch, collegati allo schermo Wayland e alla scheda grafica del PC, con una configurazione vuota: Phonestra "
       "deve aprire «Aggiungi un telefono». Con " + c("PHONESTRA_FOTO") + " la finestra si salva in PNG e lo script "
       "riporta gli errori del registro.", lead=True)

S8 = p("Una versione nuova si pubblica in quattro passi, sempre nello stesso ordine.", lead=True) + steps([
    "Versione in " + c("Cargo.toml") + " (per le candidate: " + c("1.0.0-rc.N") + "), "
    + c("python3 docs/sorgenti/build.py") + " (la versione compare nei manuali), commit e push.",
    "AppImage col contenitore, prova di avvio e di collegamento, impronta SHA-256.",
    "Tag annotato sul commit e " + c("gh release create … --prerelease") + " con l'AppImage e le note (novità, "
    "requisiti, impronta).",
    "Copia dell'AppImage nella home dell'utente (" + c("~/Phonestra-<versione>-x86_64.AppImage") + "): è quella che "
    "l'utente avvia. Su GitHub resta solo l'ultima candidata; le etichette git delle precedenti si tengono.",
])

CHAPTER = ("Costruzione e rilascio", [
    ("Strumenti necessari", S1),
    ("Comandi di tutti i giorni", S2),
    ("Compilare il componente", S3),
    ("Il contenitore dell'AppImage", S4),
    ("Cosa fa raccogli.sh", S5),
    ("AppRun", S6),
    ("Prova sulle distribuzioni", S7),
    ("Pubblicare una versione", S8),
])
