from build import c, note, p, rif, table, term, ul

S1 = p("Due livelli: le prove sul PC, che non vogliono il telefono e girano a ogni commit, e le prove sul telefono "
       "vero con " + c("phonestra-prova") + ".", lead=True) + \
    p(c("cargo test") + " prova le parti pure: formati dei messaggi a pezzi (comandi, audio, video, input), preambolo, "
      "riga di pronto, " + c("CIAO") + ", battito, pacchetti " + c("shell,v2") + ", smistamento senza rete, orari e "
      "margine dell'audio, AAC vero decodificato e scritto in MP4, controllo di flusso contro un finto adbd, mDNS, "
      "lettura di " + c("dumpsys") + ", numeri dei messaggi uguali tra Java e Rust, e questo manuale ("
      + c("tests/manuale.rs") + ", " + rif("Il manuale e i suoi controlli") + ").")

S2 = p("Un eseguibile a parte, senza interfaccia, che usa lo stesso codice del programma. Le prove del componente si "
       "fanno col telefono sbloccato e Phonestra chiuso; ognuna ripulisce da sé e controlla che sul telefono non resti "
       "niente.", lead=True) + \
    table(["Comando", "Cosa fa"], [
        "Collegamento",
        [c("cerca") + ", " + c("collega") + ", " + c("banner"), "Ricerca mDNS, collegamento al telefono salvato, "
         "funzioni annunciate da adbd"],
        [c("abbina <codice> [ip:porta]"), "Associazione col codice a 6 cifre; senza indirizzo cerca da sola la "
         "schermata del codice con mDNS"],
        [c("usb") + ", " + c("prepara") + ", " + c("shell-usb <comando>") + ", " + c("procedura"), "Il cavo: stato, "
         "preparazione del Wi-Fi, shell, la procedura di riserva"],
        [c("shell <comando>"), "Un comando di shell via Wi-Fi (al posto di " + c("adb shell") + ")"],
        [c("throughput [MB] [--senza-delayed-ack] [--payload N] [--finestra N] [--latenza] [--exec] [--alla-lettura]"),
         "Velocità e latenza del trasporto ADB"],
        [c("canali"), "Più comandi insieme sullo stesso collegamento e copia di un file con " + c("sync:") + " (prova vecchia)"],
        "Aiutante e drawer",
        [c("app") + ", " + c("sfondo") + ", " + c("notifiche") + ", " + c("codificatori"), "Comandi dell'aiutante "
         "e letture del drawer"],
        [c("file <cartella>") + ", " + c("file ricevi <percorso> <destinazione>") + ", " + c("file miniature <percorsi>"),
         "«Ricevi file…» da riga di comando: elenco, copia sul PC, miniature"],
        "Componente",
        [c("servizio [secondi] [--sparisci]"), "Lo scheletro del componente: avvio, " + c("CIAO") + ", battito, "
         "custode, uscita; " + c("--sparisci") + " simula un PC che sparisce"],
        [c("custode [abbandona]"), "Un custode di prova come quello del collegamento (tempo di spegnimento a 1234 s, "
         "senza il volume): chiude il canale e controlla che il valore torni com'era; con " + c("abbandona")
         + " esce senza chiuderlo, e il ripristino si controlla a mano (" + c("shell settings get system screen_off_timeout") + ")"],
        [c("audio-componente <secondi> [aac|pcm] [--ascolta] [--uccidi]"), "Audio dal componente in un file ("
         + c("phonestra-prova.aac") + "/" + c(".wav") + "), controllo degli orari e delle politiche"],
        [c("video-componente app|schermo [--app P] [--secondi N] [--codec C] [--senza-pannello]"), "Schermo virtuale o "
         "specchio, fotogrammi chiave, ridimensionamento, pannello, eventi; salva il flusso per " + c("ffprobe")],
        [c("input-componente appunti|tocchi|testo|tutte [--misura LxA] [--dpi D] [--app P] [--azione A[:P]] [--tocca X,Y]"),
         "Appunti, tocchi, rotellina, pizzico, testo, verificati con la «firma» dello schermo di prova"],
        "Studio",
        [c("video-prova schermo|chiave|istanze|protetto|task|permessi|codificatori"), "Lo strumento di misura del "
         "video dello studio"],
        [c("audio-nostro <secondi> [submix|loopback|render] [pcm|aac] [senza-priorita] [voce]"), "Le misure "
         "dell'audio dello studio"],
    ], "«TAB» — I comandi di " + c("phonestra-prova")) + \
    term("""
$ ./target/debug/phonestra-prova servizio 10
$ ./target/debug/phonestra-prova video-componente app --secondi 15
$ PHONESTRA_DEBUG=1 ./target/debug/phonestra-prova input-componente tutte
""", "Esempi")

S3 = ul([
    "Prima " + c("phonestra-prova") + "; l'" + c("adb") + " di sistema solo per diagnosi che lo strumento non sa fare "
    "(usa un'altra chiave: il telefono chiede una nuova autorizzazione).",
    "A fine prova: chiudere il server " + c("adb") + " eventualmente avviato e le sessioni aperte; non lasciare "
    "impostazioni del telefono cambiate.",
    "Prima di riavviare Phonestra per una prova, controllare tutte le righe " + c("mCallState") + " (una per SIM): mai "
    "durante una chiamata (" + rif("Chiamate") + ").",
    "Le misure e l'esito vanno in " + c("memoria/prove-collegamento.md") + ", con il numero della sezione (§).",
])

S4 = table(["Variabile", "Effetto"], [
    [c("PHONESTRA_DEBUG=1"), "Diagnosi sul terminale (vale con qualsiasi valore): misure dell'audio, fotogrammi "
     "disegnati e scartati, appunti, scala delle finestre"],
    [c("PHONESTRA_FOTO=<cartella>"), "Ogni finestra si salva in PNG 6 s dopo l'apertura e poi ogni 6 s ("
     + c("foto.rs") + ")"],
    [c("PHONESTRA_PROVA_PASSO=…"), "Mostra un passo del primo collegamento senza telefono (" + rif("Il primo collegamento") + ")"],
    [c("PHONESTRA_PROVA_RICEVI=<posto>"), "Apre da sola «Ricevi file…» al collegamento, sul posto indicato"],
    [c("PHONESTRA_AUDIO_CODEC=pcm"), "Audio PCM invece di AAC (anche " + c("raw") + ")"],
    [c("PHONESTRA_VIDEO_FPS") + ", " + c("PHONESTRA_VIDEO_PRIORITA") + ", " + c("PHONESTRA_VIDEO_PROTETTA=0"),
     "Interruttori di prova del codificatore e del controllo delle schermate protette (diventano " + c("max_fps=")
     + ", " + c("priorita=") + ", " + c("protetta=") + " di " + c("VIDEO_APRI") + ")"],
    [c("PHONESTRA_ADB_DELAYED_ACK") + ", " + c("PHONESTRA_ADB_PAYLOAD") + ", " + c("PHONESTRA_ADB_FINESTRA"),
     "Parametri del trasporto ADB (" + rif("Canali e controllo di flusso") + ")"],
], "«TAB» — Le variabili d'ambiente") + \
    p("Il registro di Phonestra avviato dall'AppImage è nel journal del PC: " + c("journalctl --user --since today | "
      "grep -i phonestra") + " (" + rif("Il registro dei cambi") + ").")

S5 = p("Questo manuale è generato: i sorgenti sono in " + c("docs/sorgenti/") + ", uno per capitolo in "
       + c("capitoli/") + ", e " + c("build.py") + " li raccoglie in " + c("docs/manuale-tecnico.html") + ", un file "
       "unico senza script, leggibile anche scaricato da solo. È lo stesso sistema dei manuali di AMS.", lead=True) + \
    p(c("tests/manuale.rs") + " lancia " + c("python3 docs/sorgenti/build.py --controlla") + ", che fallisce se:") + ul([
        "il file pubblicato non corrisponde ai sorgenti (il manuale non si modifica a mano);",
        "un sorgente manca dalla " + rif("Mappa dei file") + " o la mappa cita un file che non c'è più;",
        "un simbolo Rust citato (come " + c("Collegamento::mantieni") + ") non esiste nei sorgenti, o un "
        "metodo Java citato (come " + c("Servizio.comandi") + ") non è nel file della classe;",
        "un file citato non esiste nel repository;",
        "una variabile " + c("PHONESTRA_*") + " del codice non è documentata, o il manuale ne cita una che non esiste;",
        "un comando di " + c("phonestra-prova") + " o dell'aiutante non è documentato;",
        "un collegamento interno porta a una sezione che non c'è, o la versione di " + c("Cargo.toml") + " non compare.",
    ]) + note("i controlli trovano i nomi spariti, non i comportamenti cambiati: quando cambi il modo in cui una cosa "
              "funziona, cerca nel manuale la sezione che la descrive e aggiornala nello stesso commit.",
              "Cosa i controlli non vedono.")

CHAPTER = ("Prove", [
    ("Prove sul PC", S1),
    ("Lo strumento phonestra-prova", S2),
    ("Regole delle prove sul telefono", S3),
    ("Variabili per la diagnosi", S4),
    ("Il manuale e i suoi controlli", S5),
])
