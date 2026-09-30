from build import VERSION, arrow, box, c, conta, dl, fig, note, numeri, p, righe, rif, table, tree, zone

INSIEME = fig(
    zone(20, 12, 300, 214, "PC Linux (Rust, GTK4)")
    + box(40, 42, 260, 50, "Drawer", "app, notifiche, schermo del telefono", "navy")
    + box(40, 102, 260, 50, "Finestra di un'app", "video, mouse, tastiera", "blue")
    + box(40, 162, 260, 50, "Finestra di un'app", "una per ogni app aperta", "blue")
    + box(362, 102, 176, 50, "ADB su Wi-Fi", "una connessione cifrata", "dark")
    + arrow(322, 127, 360, 127) + arrow(540, 127, 578, 127)
    + zone(580, 12, 300, 214, "Telefono Android 14+ (Java)")
    + box(600, 42, 260, 50, "Specchio", "lo schermo principale", "light")
    + box(600, 102, 260, 50, "Schermo virtuale", "l'app della finestra", "light")
    + box(600, 162, 260, 50, "Schermo virtuale", "un'altra app", "light"),
    900, 240, "«FIG» — Phonestra: ogni finestra del PC mostra uno schermo del telefono, tutto su un solo collegamento")

S1 = p("<b>Phonestra</b> porta le app di un telefono Android sul PC Linux, ciascuna nella sua finestra, via Wi-Fi e "
       "senza installare niente sul telefono. " + c("SPECIFICHE.md") + " dice che cosa fa; questo manuale spiega come.",
       lead=True) + \
    p("Ogni finestra di app è uno schermo virtuale del telefono, grande quanto la finestra; il drawer, la finestra "
      "principale, mostra anche lo schermo vero del telefono. Video, audio, tocchi, tasti e appunti passano da un "
      "componente scritto da zero, copiato sul telefono a ogni collegamento e cancellato alla fine.") + INSIEME + \
    p("Dopo aver letto il manuale dovresti sapere dove mettere le mani in ogni file, compilare il programma e il "
      "componente del telefono, costruire l'AppImage, fare le prove e aggiungere un messaggio o un canale senza "
      "rompere il resto.") + \
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

S2 = p("Pochi concetti tornano in tutto il manuale. Questa tabella li presenta in una riga ciascuno, con il capitolo "
       "che li approfondisce.", lead=True) + \
    table(["Concetto", "Che cos'è", "Dove", "Approfondimento"], [
        ["<b>Collegamento</b>", "Il telefono attivo e la sua connessione ADB, mantenuta finché Phonestra resta aperto; "
         "ricomincia da capo a ogni caduta", c("collegamento.rs"), rif("Vita di un collegamento")],
        ["<b>Client ADB</b>", "Il protocollo di ADB scritto da noi: tutti i canali su una sola connessione TCP cifrata",
         c("src/adb/"), rif("Il client ADB")],
        ["<b>Componente</b>", "Il jar Java copiato sul telefono; il <i>servizio</i> è il suo processo di lunga durata, "
         "uno per collegamento", c("telefono/aiuto/") + ", " + c("componente.rs"), rif("Il componente sul telefono")],
        ["<b>Sessione video</b>", "Uno schermo virtuale per la finestra di un'app, o lo specchio dello schermo "
         "principale per il drawer, col suo canale " + c("video:<id>"), c("video_nostro/"),
         rif("Sessioni: schermo virtuale e specchio")],
        ["<b>Custode</b>", "Processo di shell che rimette a posto il telefono quando finisce ciò a cui è legato",
         c("collegamento.rs") + ", " + c("Custode.java"), rif("I due custodi")],
        ["<b>Pannello «in mano»</b>", "Lo schermo fisico si spegne mentre si usa il telefono dal PC e si riaccende "
         "quando l'utente lo riprende in mano", c("Pannello.java") + ", " + c("collegamento.rs"),
         rif("Il telefono in mano")],
        ["<b>Drawer</b>", "La finestra principale: app, notifiche, telefoni, strumenti e lo schermo del telefono "
         "disegnato", c("cassetto.rs"), rif("Il drawer")],
        ["<b>Aiutante</b>", "Lo stesso jar usato per comandi brevi che stampano e finiscono (elenco delle app, sfondo, "
         "miniature)", c("app.rs") + ", " + c("Aiuto.java"), rif("Che cos'è il componente")],
    ], "«TAB» — I concetti di base") + \
    note("Phonestra usa un telefono alla volta; gli altri telefoni configurati restano in attesa nella barra laterale "
         "del drawer (" + rif("Più telefoni") + ").", "Un telefono alla volta.")

S3 = p("Il codice è diviso in parti coese. Questa è la mappa che il manuale approfondisce capitolo per capitolo; le "
       "righe di ogni file sono nell'" + rif("Appendice B — Mappa dei file") + ".", lead=True) + dl([
    (c("src/bin/phonestra.rs") + ", " + c("src/collegamento.rs"), c("main") + ", la vita del collegamento, il giro dei "
     "3 s, la chiusura, i dati sul PC (" + rif("Avvio e ciclo di vita") + ")."),
    (c("src/adb/") + ", " + c("src/rete.rs"), "Il client ADB: messaggi, TLS, canali, " + c("shell,v2") + ", "
     + c("sync:") + ", associazione col codice, ricerca mDNS (" + rif("Il client ADB") + ")."),
    (c("telefono/aiuto/src/phonestra/") + ", " + c("src/componente.rs"), "Il componente: servizio, canali, canale "
     "comandi, battito, custodi, autotest, e il suo lato PC (" + rif("Il componente sul telefono") + ")."),
    (c("src/video_nostro/") + ", " + c("SessioneVideo.java") + ", " + c("Codifica.java"), "Schermi virtuali, specchio, "
     "codificatore H.264, ridimensionamento, eventi delle app (" + rif("Video") + ")."),
    (c("Pannello.java"), "Lo schermo fisico spento durante l'uso dal PC, le chiamate, le cadute ("
     + rif("Il pannello del telefono") + ")."),
    (c("src/audio_nostro.rs") + ", " + c("CanaleAudio.java"), "Cattura loopback, AAC, riproduzione e registrazione ("
     + rif("Audio") + ")."),
    (c("src/input_nostro.rs") + ", " + c("src/appunti.rs") + ", " + c("Input.java") + ", " + c("Appunti.java"),
     "Tocchi, tasti, testo e appunti nei due sensi (" + rif("Input e appunti") + ")."),
    (c("src/cassetto.rs") + ", " + c("src/finestra.rs") + ", " + c("src/prepara.rs") + ", " + c("src/procedura.rs")
     + ", " + c("src/avvisi.rs"), "Drawer, finestre delle app, preferenze, primo collegamento, notifiche ("
     + rif("L'interfaccia") + ")."),
    (c("src/azioni.rs") + ", " + c("src/ricevi.rs"), "Installare app, inviare e ricevere file (" + rif("App e file")
     + ")."),
    (c("src/bin/prova.rs") + ", " + c("tests/"), "Le prove sul PC e sul telefono (" + rif("Prove e diagnosi") + ")."),
    (c("costruzione/") + ", " + c("telefono/aiuto/costruisci.sh"), "Il componente compilato, l'AppImage, le versioni ("
     + rif("Costruzione e rilascio") + ")."),
])

S4 = p("Il repository " + c("nic-fio/PHONESTRA") + " è l'unica copia completa del progetto: codice, componente "
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
    "├── docs/  # i due manuali e i loro sorgenti (docs/sorgenti/)",
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
         "compilazione (formato in " + rif("Il primo collegamento") + ")."],
        [c("docs/"), c("Phonestra_Manuale_Tecnico.html") + " e " + c("Phonestra_Manuale_Utente.html") + " (generati, "
         "mai a mano) e i loro sorgenti in " + c("docs/sorgenti/") + ": " + c("build.py") + ", " + c("stile.css")
         + ", un file per capitolo in " + c("tecnico/") + " e " + c("utente/") + "."],
        [c("memoria/"), "Decisioni, studi, misure, registro dei problemi: il perché del progetto."],
        [c("SPECIFICHE.md"), "Cosa fa Phonestra, sezione per sezione; i numeri «§» del codice rimandano qui."],
        [c("LICENZA.md"), "Licenza proprietaria: uso personale gratuito, niente modifiche, redistribuzione, uso "
         "commerciale o in azienda."],
    ], "«TAB» — Che cosa c'è nel repository")

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
     "contenitore, AppImage, jar, generatore dei manuali", lambda f: True),
]

S5 = p("Quanto è grande Phonestra, parte per parte. Le righe si contano dai sorgenti a ogni generazione del manuale.",
       lead=True) + numeri(PARTI, "«TAB» — Le parti del progetto e le loro righe") + \
    p("I numeri di questa tabella e della " + rif("Appendice B — Mappa dei file") + " non sono scritti a mano, e "
      + c("cargo test") + " fallisce se il manuale pubblicato non corrisponde più ai sorgenti ("
      + rif("Il manuale e i suoi controlli") + ").")

CHAPTER = ("Introduzione e concetti", [
    ("Che cos'è Phonestra", S1),
    ("I concetti di base", S2),
    ("Sottosistemi in breve", S3),
    ("Il repository", S4),
    ("Il progetto in numeri", S5),
])
