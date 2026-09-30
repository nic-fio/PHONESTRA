from build import VERSION, c, conta, numeri, p, righe, rif, table

S1 = p("Phonestra porta le app di un telefono Android sul PC Linux, ciascuna nella sua finestra, via Wi-Fi e senza "
       "installare niente sul telefono. " + c("SPECIFICHE.md") + " dice che cosa fa; questo manuale spiega come. Dopo "
       "averlo letto dovresti sapere dove mettere le mani in ogni file, compilare il programma e il componente del "
       "telefono, costruire l'AppImage, fare le prove e aggiungere un messaggio o un canale senza rompere il resto.",
       lead=True) + \
    p("Il manuale presuppone che tu conosca Rust e un po' di Java. I concetti di Android e di ADB sono spiegati dove "
      "servono; il " + rif("Glossario") + " li riassume. Il perché di ogni scelta sta nella cartella " + c("memoria/")
      + ": qui trovi il rimando giusto quando serve (per esempio «prove §49» è la sezione 49 di "
      + c("memoria/prove-collegamento.md") + ").") + \
    table(["Voce", "Valore"], [
        ["Versione", c(VERSION)],
        ["Linguaggi", "Rust 2024 sul PC, Java sul telefono"],
        ["Dimensione", f"{righe(conta('.rs'))} righe di Rust, {righe(conta('.java'))} di Java"],
        ["Destinazione", "Linux x86_64 (AppImage) · Android 14 e successivi"],
    ], "«TAB» — Phonestra in breve")

S2 = table(["Principio", "Che cosa vuol dire"], [
    ["<b>Niente sul telefono</b>", "Nessuna app installata. Il componente si copia in " + c("/data/local/tmp")
     + " a ogni collegamento e si cancella alla fine."],
    ["<b>Il telefono torna com'era</b>", "Ogni impostazione cambiata ha un custode che la rimette, anche se il PC "
     "sparisce o il servizio muore di colpo (" + rif("I due custodi") + ")."],
    ["<b>ADB nostro</b>", "Nessun " + c("adb") + " da installare: TLS, associazione col codice, mDNS e canali "
     "multipli sono scritti qui."],
    ["<b>Codice nostro</b>", "Il componente del telefono è scritto da zero: niente scrcpy, niente codice di terzi da "
     "cui dipendere."],
    ["<b>Misurare prima di scrivere</b>", "Studio, poi misure sul telefono vero, poi codice un pezzo alla volta. Le "
     "misure restano in " + c("memoria/") + "."],
    ["<b>Un file, nessuna traccia</b>", "Un'AppImage che contiene tutto; sul PC solo " + c("~/.config/Phonestra")
     + " e " + c("~/.cache/Phonestra") + "."],
], "«TAB» — I principi che hanno formato Phonestra")

PARTI = [
    ("Prove e misure (PC)", c("src/bin/prova*") + ", " + c("prova_input.rs") + ", " + c("misura_audio.rs") + ", "
     + c("video_nostro/prova.rs") + ", " + c("adb/misura.rs") + ", " + c("adb/prove.rs") + ", " + c("tests/"),
     "lo strumento " + c("phonestra-prova") + ", le misure dello studio, i test d'integrazione",
     lambda f: f.startswith(("src/bin/prova", "tests/")) or f in (
         "src/prova_input.rs", "src/misura_audio.rs", "src/video_nostro/prova.rs", "src/adb/misura.rs", "src/adb/prove.rs")),
    ("Prove e misure (telefono)", c("VideoProva.java") + ", " + c("InputProva.java") + ", " + c("Codificatori.java"),
     "strumenti di misura e comandi di prova del componente",
     lambda f: f.endswith(("/VideoProva.java", "/InputProva.java", "/Codificatori.java"))),
    ("Interfaccia", c("cassetto.rs") + ", " + c("finestra.rs") + ", " + c("ricevi.rs") + ", " + c("prepara.rs") + ", "
     + c("procedura.rs") + ", " + c("avvisi.rs") + ", " + c("foto.rs") + ", " + c("bin/phonestra.rs"),
     "drawer, finestre delle app, ricezione dei file, primo collegamento, avvisi",
     lambda f: f in ("src/cassetto.rs", "src/finestra.rs", "src/ricevi.rs", "src/prepara.rs", "src/procedura.rs",
                     "src/avvisi.rs", "src/foto.rs", "src/bin/phonestra.rs")),
    ("Client ADB", c("src/adb/"), "messaggi, TLS, canali, " + c("shell,v2") + ", " + c("sync:") + ", associazione",
     lambda f: f.startswith("src/adb/")),
    ("Componente, lato PC", c("componente.rs") + ", " + c("audio_nostro.rs") + ", " + c("video_nostro/") + ", "
     + c("input_nostro.rs") + ", " + c("appunti.rs"), "servizio, smistamento, audio, video, input, appunti",
     lambda f: f in ("src/componente.rs", "src/audio_nostro.rs", "src/input_nostro.rs", "src/appunti.rs")
     or f.startswith("src/video_nostro/")),
    ("Collegamento e dati", c("collegamento.rs") + ", " + c("rete.rs") + ", " + c("configurazione.rs") + ", "
     + c("telefono.rs") + ", " + c("usb.rs") + ", " + c("notifiche.rs") + ", " + c("azioni.rs") + ", " + c("app.rs")
     + ", " + c("lib.rs"), "collegamento, pannello, mDNS, configurazione, cavo, notifiche, azioni, elenco delle app",
     lambda f: f.startswith("src/")),
    ("Componente sul telefono", c("telefono/aiuto/src/phonestra/"),
     "servizio, custode, audio, video, input, appunti, pannello, miniature",
     lambda f: f.startswith("telefono/aiuto/src/")),
    ("Costruzione e strumenti", c("costruzione/") + ", " + c("costruisci.sh") + ", " + c("docs/sorgenti/"),
     "contenitore, AppImage, jar, generatore di questo manuale", lambda f: True),
]

S3 = numeri(PARTI, "«TAB» — Le parti del progetto e le loro righe") + \
    p("I numeri di questa tabella e della " + rif("Mappa dei file") + " non sono scritti a mano: il generatore del "
      "manuale li conta dai sorgenti a ogni generazione, e " + c("cargo test") + " fallisce se il manuale pubblicato "
      "non corrisponde più ai sorgenti (" + rif("Il manuale e i suoi controlli") + ").")

CHAPTER = ("Introduzione", [
    ("Cosa copre questo manuale", S1),
    ("Principi", S2),
    ("Il progetto in numeri", S3),
])
