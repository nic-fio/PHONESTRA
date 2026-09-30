from build import c, note, p, rif, table, term, tip, tree, warn

S1 = p("Il repository " + c("nic-fio/PHONESTRA") + " è l'unica copia completa del progetto: codice, componente "
       "compilato, documenti, mockup e misure.", lead=True) + tree([
    "PHONESTRA/",
    "├── Cargo.toml  # il pacchetto phonestra: una libreria e due eseguibili",
    "├── src/  # il programma per il PC",
    "│   ├── adb/  # il client ADB",
    "│   ├── video_nostro/  # lato PC del video del componente",
    "│   └── bin/  # phonestra.rs (il programma) e prova.rs (phonestra-prova)",
    "├── telefono/",
    "│   ├── aiuto/src/phonestra/  # il componente per il telefono, in Java",
    "│   ├── aiuto/stub/  # finte classi Android, solo per il compilatore",
    "│   └── phonestra-aiuto.jar  # il componente compilato, incorporato nell'eseguibile",
    "├── dati/istruzioni.toml  # istruzioni per marca della procedura col cavo",
    "├── costruzione/  # contenitore, script e AppRun dell'AppImage",
    "├── docs/  # questo manuale e i suoi sorgenti (docs/sorgenti/)",
    "├── memoria/  # decisioni, studi, misure, registro dei problemi",
    "├── tests/  # prove d'integrazione (tests/manuale.rs)",
    "├── grafica/, mockup/  # logo e icone; proposte grafiche e canvas dell'interfaccia",
    "├── prove/  # piccoli script di prova fuori dal programma",
    "├── SPECIFICHE.md  # cosa fa Phonestra; i «§» del codice rimandano qui",
    "└── LICENZA.md  # uso personale gratuito",
], "«FIG» — Le cartelle del repository") + \
    table(["Percorso", "Contenuto"], [
        [c("src/"), "Il programma per il PC: interfaccia, collegamento, client ADB (" + c("src/adb/")
         + "), lato PC del componente (" + c("componente.rs") + ", " + c("audio_nostro.rs") + ", "
         + c("video_nostro/") + ", " + c("input_nostro.rs") + ")."],
        [c("telefono/aiuto/"), "Il componente per il telefono: sorgenti Java in " + c("src/phonestra/")
         + ", finte classi Android per il compilatore in " + c("stub/") + ", " + c("costruisci.sh") + "."],
        [c("telefono/phonestra-aiuto.jar"), "Il componente compilato (dex dentro un jar). È nel repository e viene "
         "incorporato nell'eseguibile."],
        [c("dati/istruzioni.toml"), "Istruzioni per famiglia di marca della procedura col cavo, incorporate alla "
         "compilazione (" + "formato in " + rif("Il primo collegamento") + ")."],
        [c("docs/"), c("manuale-tecnico.html") + " (generato, mai a mano) e i suoi sorgenti in " + c("docs/sorgenti/")
         + ": " + c("build.py") + ", " + c("stile.css") + ", un file per capitolo in " + c("capitoli/") + "."],
        [c("memoria/"), "Decisioni, studi, misure, registro dei problemi: il perché del progetto."],
        [c("SPECIFICHE.md"), "Cosa fa Phonestra, sezione per sezione; i numeri «§» del codice rimandano qui."],
        [c("LICENZA.md"), "Licenza proprietaria: uso personale gratuito, niente modifiche, redistribuzione, uso "
         "commerciale o in azienda."],
    ], "«TAB» — Che cosa c'è nel repository")

S2 = table(["Strumento", "Per cosa", "Note"], [
    ["Rust stabile (edizione 2024)", "Tutto il PC", c("cargo") + " sta in " + c("~/.cargo/bin") + ": aggiungilo al "
     + c("PATH") + " nei comandi."],
    ["GTK 4.12+, libadwaita 1.5+, GStreamer 1.x (con " + c("-dev") + ")", "Compilare e avviare sul PC di sviluppo",
     "Plugin necessari: base, good, bad, " + c("gstreamer1.0-libav") + " (per " + c("avdec_aac") + ") e "
     + c("gst-plugin-gtk4") + " (" + c("gtk4paintablesink") + ")."],
    ["JDK (" + c("javac") + ")", "Compilare il componente", "Compilato con " + c("--release 11") + "; va bene un JDK recente."],
    ["D8 di R8 9.4.26", "Da classi Java a dex", "In " + c("strumenti/r8.jar") + " (non nel repository: indirizzo e "
     "impronta SHA-256 in " + c("telefono/aiuto/costruisci.sh") + ")."],
    [c("podman"), "Costruire l'AppImage", "Contenitore " + c("phonestra-appimage") + " (" + rif("Costruire l'AppImage") + ")."],
    ["Python 3 con " + c("pygments"), "Generare questo manuale", c("python3 docs/sorgenti/build.py") + "; pacchetto "
     + c("python3-pygments") + ". Lo usa anche " + c("cargo test") + "."],
], "«TAB» — Gli strumenti che servono sul PC") + \
    p("Il sistema di riferimento è Debian 13 «trixie». Per le prove serve un telefono con Android 14 o successivo, "
      "il Debug wireless acceso e il PC sulla stessa rete Wi-Fi.")

S3 = term("""
$ export PATH=$HOME/.cargo/bin:$PATH
$ cargo build   # programma e strumento delle prove
$ cargo test   # prove sul PC, senza telefono; controlla anche questo manuale
$ cargo clippy --all-targets   # nessun avviso ammesso
$ cargo run --bin phonestra   # avvia Phonestra dai sorgenti
$ telefono/aiuto/costruisci.sh   # ricompila il componente del telefono
$ python3 docs/sorgenti/build.py   # rigenera questo manuale
""", "Comandi di tutti i giorni") + \
    tip("cambi il codice → " + c("cargo test") + " (secondi) → se hai toccato il componente, " + c("costruisci.sh")
        + " e la prova " + c("phonestra-prova") + " del pezzo → Phonestra vero sul telefono. Prima di ogni commit: "
        + c("cargo build") + ", " + c("cargo test") + " e " + c("cargo clippy") + " senza avvisi.",
        "Il giro di tutti i giorni.") + \
    note("tre cose stanno fuori dal repository. " + c("strumenti/r8.jar") + " si scarica con il comando scritto in "
         + c("costruisci.sh") + ". L'immagine del contenitore " + c("phonestra-appimage") + " si ricostruisce con "
         + c("podman build") + ". La configurazione di chi sviluppa (" + c("~/.config/Phonestra") + ": chiave ADB e "
         "telefoni associati) resta sul PC: su un PC nuovo il telefono va associato di nuovo. Le Release di GitHub "
         "con le AppImage si rifanno dal codice.", "Cosa non porta un clone.") + \
    warn("il manuale si cambia nei sorgenti di " + c("docs/sorgenti/") + ", mai in " + c("docs/manuale-tecnico.html")
         + ": la generazione successiva cancellerebbe la modifica, e " + c("cargo test") + " fallisce finché il file "
         "pubblicato non corrisponde ai sorgenti.", "Il manuale è generato.")

CHAPTER = ("Sorgenti e strumenti", [
    ("Il repository", S1),
    ("Strumenti necessari", S2),
    ("Comandi di tutti i giorni", S3),
])
