from build import arrow, box, c, diamond, fig, note, p, rif, seq, table, term, text, ul, warn

S1 = p("Tutto quello che Phonestra fa sul telefono passa da un servizio Java scritto da zero, avviato come la shell "
       "di ADB e cancellato alla fine. Questo capitolo ne descrive l'infrastruttura; i capitoli seguenti i pezzi: "
       "video, pannello, audio, input.", lead=True) + \
    p(c("telefono/phonestra-aiuto.jar") + " contiene un " + c("classes.dex") + ". Il PC lo incorpora ("
      + c("app::AIUTO") + ", " + c("include_bytes!") + "), lo copia in " + c("/data/local/tmp") + " e lo avvia con "
      + c("app_process") + ": gira con l'uid della shell (2000) e i suoi permessi (catturare lo schermo e l'audio, "
      "iniettare eventi, leggere gli appunti). Non è un'app, non si installa, non compare nelle impostazioni.") + \
    p("Lo stesso jar ha due modi d'uso, scelti dal primo argomento di " + c("phonestra.Aiuto") + ": il "
      "<b>servizio</b>, il processo di lunga durata, uno per collegamento, che fa audio, video, input, appunti e "
      "pannello; e i <b>comandi brevi dell'aiutante</b>, che stampano e finiscono. Per ogni comando breve il PC copia "
      "il jar con un nome suo (" + c("phonestra-aiuto.jar.<8 cifre esadecimali>") + ", perché più richieste possono "
      "arrivare insieme), lo esegue e lo cancella (" + c("app.rs") + ").") + \
    table(["Comando", "Che cosa stampa", "Chi lo usa"], [
        [c("app [lato]"), "l'elenco delle app del launcher con le icone PNG in base64, chiuso da una riga "
         + c("fine\\t<n>") + "; senza argomenti vale " + c("app 96"), "drawer (" + c("app::elenco") + ")"],
        [c("sfondo [larghezza]"), "lo sfondo del telefono in PNG (predefinita 540)", "telefono disegnato nel drawer"],
        [c("miniature <lato> <percorsi in base64>"), "una riga " + c("indice\\tJPEG in base64") + " (qualità 80) o "
         + c("indice\\t-") + " per ogni file", "«Ricevi file…» (" + rif("Ricevere file dal telefono") + ")"],
        [c("pannello [0|1]"), c("schermi=<quanti>") + " dopo aver acceso (1, predefinito) o spento il pannello",
         "il custode (" + rif("I due custodi") + ")"],
        [c("codificatori"), "i codificatori audio e video del telefono", c("phonestra-prova codificatori")],
        [c("audio …"), "audio catturato, per lo studio (" + c("sorgente=submix|loopback|render") + ", "
         + c("formato") + ", " + c("priorita") + ", " + c("voce") + ")", c("phonestra-prova audio-nostro")],
        [c("video-prova …"), "lo strumento di misura del video dello studio", c("phonestra-prova video-prova")],
        [c("servizio"), "la riga di pronto, poi resta vivo", c("Componente::avvia")],
    ], "«TAB» — I comandi del jar. Un comando sconosciuto esce con codice 2") + \
    p("Java solo dove Android lo impone: le API che servono (" + c("MediaCodec") + ", " + c("VirtualDisplay") + ", "
      + c("AudioPolicy") + ", " + c("InputManager") + ") esistono solo in Java. La codifica audio e video la fanno "
      "comunque i codificatori del telefono.")

AVVIO = seq([("PC", "Componente::avvia", "navy"), ("adbd", "", "dark"), ("Servizio", "Java, uid 2000", "blue"),
             ("Custode", "sh", "amber")], [
    (0, 1, "sync: phonestra-servizio-<casuale>.jar"),
    (0, 1, "shell,v2,raw: exec app_process … servizio"),
    (1, 2, "avvio (uid 2000)"),
    (0, 2, "segreto (32 cifre esadecimali) sull'ingresso"),
    (2, 3, "setsid sh -c … (pipe delle azioni)"),
    (2, 2, "cancella il jar, contesto, autotest"),
    (2, 2, "LocalServerSocket phonestra_<32 hex>"),
    (2, 0, "riga di pronto (uscita del processo)", True),
    (0, 2, "localabstract: preambolo «comandi»"),
    (2, 0, "CIAO (versioni, autotest)", True),
    ("sep", "ogni secondo, nei due sensi"),
    (0, 2, "BATTITO"),
], "«FIG» — Dall'avvio al primo battito", width=900)

S2 = p("Il PC avvia il servizio su un canale " + c("shell,v2") + ": copia il jar, lo lancia con " + c("app_process")
       + ", gli passa il segreto sull'ingresso e aspetta la riga di pronto. Poi apre il canale comandi e riceve il "
       + c("CIAO") + ".", lead=True) + AVVIO + \
    p("La riga di pronto è " + c("phonestra-servizio pronto protocollo=1 socket=phonestra_<32 hex> pid=<pid>") + "; se "
      "l'avvio fallisce, " + c("phonestra-servizio errore <causa>") + " e codice 1. Il PC aspetta il pronto 20 s e il "
      + c("CIAO") + " 10 s; se il servizio non parte cancella lui il jar. " + c("exec") + " fa prendere al servizio il "
      "posto di " + c("sh") + ", così il SIGHUP di adbd arriva proprio a lui; " + c("--nice-name") + " lo fa comparire "
      "in " + c("ps") + " come " + c("phonestra-servizio") + ".")

S3 = p("Ogni canale è un " + c("localabstract:phonestra_<32 hex>") + " aperto dal PC. I primi byte sono il preambolo: "
       + c("segreto (16 byte) · lunghezza del tipo u8 · tipo ASCII") + ". Il servizio lo legge con 3 s di tempo "
       "massimo e sceglie il gestore dalla parte del tipo prima dei due punti (" + c("Servizio.TIPI") + ").",
       lead=True) + \
    table(["Tipo", "Gestore", "Contenuto"], [
        [c("comandi"), c("Servizio.comandi"), "Messaggi in tutti e due i sensi. Uno solo per servizio: un secondo "
         "canale " + c("comandi") + " viene rifiutato"],
        [c("audio") + ", " + c("audio:aac") + ", " + c("audio:pcm"), c("CanaleAudio.gestisci"), "Pacchetti audio "
         "verso il PC (" + rif("La ricetta") + "); un formato sconosciuto riceve il testo " + c("errore formato sconosciuto")],
        [c("video:<id>"), c("Video.canale"), "Pacchetti video di una sessione (" + rif("Il canale video:<id>") + ")"],
    ], "«TAB» — I tipi di canale") + \
    p("Segreto sbagliato, uid diverso da 2000 o tipo sconosciuto: il socket si chiude senza risposta.")

S4 = p("Il canale comandi porta tutti i messaggi brevi tra PC e servizio, nei due sensi: domande con risposta, "
       "eventi spontanei, il battito.", lead=True) + \
    p("Messaggi con un'intestazione di 8 byte big-endian, " + c("tipo u8 · bandiere u8 · id u16 · lunghezza u32")
       + ", e il contenuto (al massimo 16 MB: oltre, il flusso è rovinato e il canale si chiude). " + c("id")
       + " lega la risposta alla domanda (0 = messaggio spontaneo); la bandiera " + c("0x01") + " dice che è una "
       "risposta. Formato uguale in " + c("Protocollo.java") + " e " + c("componente.rs") + ", provato da un test con "
       "gli stessi byte.") + \
    table(["Fascia", "Chi", "Dove"], [
        [c("0x01–0x0f"), "Infrastruttura", c("componente::tipo") + ", " + c("Protocollo.java")],
        [c("0x10–0x1f"), "Prove e diagnosi", c("PROVA_CUSTODE")],
        [c("0x40–0x4f"), "Video e pannello", rif("Messaggi video")],
        [c("0x50–0x5f"), "Input e appunti", rif("Messaggi di input e appunti")],
    ], "«TAB» — Le fasce dei tipi di messaggio") + \
    table(["Tipo", "Nome", "Verso", "Contenuto"], [
        [c("0x01"), c("CIAO"), "servizio → PC, primo messaggio", "Righe " + c("chiave=valore") + ": protocollo, "
         "android, sdk, produttore, modello, pid, " + c("avvio_ms") + ", " + c("autotest_ms") + ", "
         + c("autotest.<voce>=…") + " (" + c("ok") + " o il motivo; per il custode " + c("ok setsid") + ", "
         + c("morto setsid") + ", " + c("ok senza-setsid") + ")"],
        [c("0x02"), c("BATTITO"), "nei due sensi, ogni secondo", "vuoto"],
        [c("0x03"), c("FINE"), "PC → servizio; risposta uguale", "vuoto; dopo la risposta il servizio esce con 0"],
        [c("0x04"), c("ERRORE"), "servizio → PC, come risposta", "testo, per esempio «tipo sconosciuto 0x2a»"],
        [c("0x10"), c("PROVA_CUSTODE"), "PC → servizio", "crea un file che il custode toglie alla fine (solo per le prove)"],
    ], "«TAB» — I messaggi dell'infrastruttura") + \
    p("Un tipo che il servizio non conosce riceve " + c("ERRORE") + ": il PC capisce che il jar è vecchio. "
      + c("PROTOCOLLO") + " (oggi 1) cambia solo se un messaggio esistente cambia significato.")

S5 = p("Qualsiasi messaggio vale come segno di vita. Dopo 5 s di silenzio ciascuna parte considera l'altra sparita: "
       "il PC chiude il canale comandi (" + c("ricevi") + " dà " + c("None") + "; controlla ogni 200 ms), il servizio "
       "esce. Serve perché adbd sul Debug wireless non si accorge da sé di un PC sparito (Wi-Fi perso, PC spento).",
       lead=True) + \
    table(["Codice", "Quando"], [
        ["0", c("FINE") + " chiesto dal PC"],
        ["1", "Errore d'avvio o eccezione non gestita"],
        ["3", "Nessun messaggio dal PC per 5 s"],
        ["4", "Canale comandi chiuso dal PC, o scrittura impossibile"],
        ["5", "Segreto non arrivato o nessun canale comandi entro 10 s"],
        ["128+n", "Ucciso dal segnale n (129 = SIGHUP: canale d'avvio chiuso; 137 = " + c("kill -9") + ")"],
    ], "«TAB» — I codici d'uscita del servizio (" + c("componente::descrivi_uscita") + ")") + \
    p("Il servizio esce con " + c("System.exit") + " e, dopo 2 s, " + c("Runtime.halt") + " se qualcosa si blocca. "
      "Il ripristino del telefono lo fa il custode, che c'è sia alla fine ordinata sia a quella improvvisa. Il "
      "servizio rimette da sé solo quello che può rimettere subito: la politica dell'audio (gancio di chiusura "
      + c("audio-fine") + "), il pannello quando lo riaccende e " + c("min_refresh_rate") + " appena spento il "
      "pannello; in quei casi toglie anche l'azione corrispondente dal custode.")

S6 = p("Phonestra cambia alcune cose sul telefono e deve rimetterle anche quando qualcosa va storto. Lo fanno due "
       "processi di shell, ognuno legato alla vita di qualcos'altro:", lead=True) + \
    table(["", "Custode del collegamento", "Custode del servizio"], [
        ["Chi lo avvia", "Il PC, " + c("collegamento.rs") + ", con " + c("exec:"), "Il servizio, " + c("Custode.java")
         + ", con " + c("setsid sh -c")],
        ["Legato a", "Il collegamento ADB: finisce quando adbd chiude il suo ingresso (" + c("cat >/dev/null") + ")",
         "Il processo del servizio: legge un pipe che solo il servizio tiene aperto"],
        ["Cosa rimette", "Tempo di spegnimento dello schermo, volume multimediale", "Pannello fisico, frequenza "
         "minima del display, task delle prove, copie del jar"],
        ["Sopravvive con", c("trap '' HUP TERM PIPE"), c("setsid") + " e " + c("trap '' HUP INT TERM PIPE")],
    ], "«TAB» — I due custodi") + \
    p("Il tempo di spegnimento va al massimo (" + c("SPEGNIMENTO_LUNGO") + ", cioè mai) finché Phonestra è aperto: a "
      "telefono addormentato le app negli schermi virtuali non ricevono input, e i tocchi dal PC non contano come "
      "attività del telefono (con 30 minuti si addormentava durante l'uso, prove §55). Il volume va al massimo "
      "perché con il volume a 0 l'app di Facebook non avvia l'audio dei reel; l'audio esce comunque solo dal PC.") + \
    p("Tutti e due i valori si salvano anche in " + c("telefoni.toml") + ": se Phonestra cade senza rimetterli, il "
      "collegamento successivo rimette quelli salvati. Il tempo di spegnimento si legge dal telefono all'avvio e si "
      "rimette alla fine solo se è ancora quello di Phonestra: se l'utente lo cambia durante il collegamento, resta "
      "il suo (prove §56). Un valore sotto i 5 s si considera non valido; 30 minuti ("
      + c("SPEGNIMENTO_LUNGO_VECCHIO") + ", il valore delle versioni fino alla rc.3) si tratta come residuo di un "
      "Phonestra caduto.") + \
    p("Il custode del servizio riceve l'elenco intero delle azioni a ogni cambiamento, tra una riga " + c("#inizio")
      + " e una " + c("#fine") + "; un elenco a metà non sostituisce il precedente. Alla morte del servizio le esegue "
      "per ordine crescente (a parità d'ordine, l'ultima aggiunta per prima), una per riga con " + c("sh -c")
      + " (una che fallisce non ferma le altre), poi cancella il jar del servizio e le copie dimenticate più vecchie "
      "di un minuto, e termina.") + \
    table(["Ordine", "Azione", "Chi la imposta"], [
        ["400", "Riaccendere il pannello: " + c("app_process … phonestra.Aiuto pannello 1") + " con una copia del jar "
         "tenuta per il custode (" + c("phonestra-custode-<pid>.jar") + "); se la copia manca, il vecchio ripiego "
         + c("KEYCODE_SLEEP") + "/" + c("WAKEUP"), c("Pannello.java")],
        ["410", "Rimettere " + c("min_refresh_rate") + " com'era (solo nell'attesa, al massimo 1 s, prima dello "
         "spegnimento del pannello)", c("Pannello.java")],
        ["500", "Togliere i task avviati dalle prove dell'input", c("InputProva.java")],
        ["900", "Togliere il file di " + c("PROVA_CUSTODE"), c("Servizio.java")],
    ], "«TAB» — Le azioni del custode del servizio") + \
    note("all'inizio il custode riaccendeva lo schermo addormentando e risvegliando il telefono (" + c("KEYCODE_SLEEP")
         + "/" + c("WAKEUP") + "). Sui Samsung questo bloccava il telefono e faceva cadere il Debug wireless a ogni "
         "riavvio di Phonestra. Ora il custode chiama " + c("setDisplayPowerMode") + " come il servizio (misure §51).",
         "Perché il pannello si riaccende in Java.") + \
    p("Le schermate virtuali non hanno bisogno di un'azione: le chiude Android quando il processo muore. Le app non si "
      "tolgono dalle recenti a una caduta, di proposito: al ritorno del collegamento tornano nella finestra col loro "
      "stato. La politica dell'audio la toglie Android (" + rif("Chi toglie la cattura") + ").")

S7 = p("Molte API che servono sono nascoste (" + c("IDisplayManager") + ", " + c("IWindowManager") + ", "
       + c("InputManagerGlobal") + ", " + c("IClipboard") + ", " + c("AudioPolicy") + "). In " + c("app_process")
       + " la politica delle API nascoste è spenta: si chiamano per riflessione.", lead=True) + \
    table(["Parte", "Come"], [
        ["Nascoste", "Servizi da " + c("ServiceManager") + " + " + c("Stub.asInterface") + ", metodi cercati per nome e "
         "numero di parametri tra le varianti note. Regola: si guarda cosa c'è, non la versione."],
        ["Contesto", c("app_process") + " non ha un contesto Android. Si preparano, un passo alla volta, Looper "
         "principale, un " + c("ActivityThread") + " di sistema, il suo " + c("ConfigurationController") + " (senza, "
         "sui Samsung " + c("DisplayManagerGlobal") + " va in errore) e il contesto del pacchetto "
         + c("com.android.shell") + ": da Android 16 i permessi della shell valgono solo col pacchetto giusto."],
        ["Autotest", "All'avvio controlla che ogni API nascosta ci sia e con quale firma, senza usarla. L'esito va al PC "
         "nel " + c("CIAO") + " (" + c("Ciao::mancanti()") + ")."],
        ["Sistema", c("Sistema.java") + " raccoglie i pezzi comuni: il display manager per ogni schermo virtuale, "
         "l'avvio delle app (" + c("startActivityAsUser") + " a 11 parametri, con ripiego " + c("am start")
         + "), " + c("togliTask") + " alla chiusura di una finestra, i comandi di shell interni."],
    ], "«TAB» — Contesto e API nascoste") + \
    table(["Voce", "Cosa controlla"], [
        [c("contesto"), "Contesto della shell pronto, pacchetto " + c("com.android.shell")],
        [c("permessi"), "I permessi della shell che servono ai pezzi"],
        [c("display_manager"), c("createVirtualDisplay") + " e " + c("DisplayManagerGlobal")],
        [c("capture_display"), c("IWindowManager.captureDisplay") + " e la classe dei suoi argomenti"],
        [c("task_stack_listener"), c("TaskStackListener") + " e la sua registrazione"],
        [c("inject_input_event"), c("injectInputEvent") + " (2 o 3 parametri), " + c("InputEvent.setDisplayId")],
        [c("audio_policy"), c("AudioPolicy") + ", " + c("AudioMix") + ", " + c("createAudioRecordSink") + ", registrazione"],
        [c("appunti"), "Metodi di " + c("IClipboard") + "; presenza di " + c("semclipboard")],
        [c("custode"), "Custode vivo, con o senza " + c("setsid")],
    ], "«TAB» — Le voci dell'autotest")

S8 = p("Il servizio gira coi permessi della shell: nessun altro deve poterlo comandare, e alla fine non deve "
       "restare niente sul telefono.", lead=True) + ul([
    "Socket astratto dal nome casuale a 128 bit; ogni canale deve venire dall'uid 2000 (" + c("getPeerCredentials")
    + ") e cominciare col segreto (confronto a tempo costante). Nessuna porta TCP aperta sul telefono.",
    "Il segreto passa sull'ingresso del processo, non sulla riga di comando: " + c("/proc/<pid>/cmdline") + " è "
    "leggibile da altri processi dello stesso uid.",
    "Nessun comando generico: il PC non può far eseguire al servizio comandi di shell arbitrari. Le azioni del custode "
    "le decide il servizio; i comandi di prova accettano solo pacchetti e azioni fatti di lettere, cifre, punti e "
    "trattini bassi.",
    "Niente resta: jar cancellato all'avvio e dal custode, servizio che esce dopo 5 s senza PC, custode che termina "
    "dopo il ripristino.",
])

SMISTA = fig(
    box(20, 55, 150, 50, "Messaggio", "dal servizio", "navy") + arrow(172, 80, 218, 80)
    + diamond(290, 80, 140, 64, "RISPOSTA?")
    + arrow(290, 113, 290, 158) + text(302, 140, "sì", 11, "#334155", "700", "start")
    + box(200, 160, 180, 50, "A chi ha domandato", "stesso id; ERRORE → errore", "blue")
    + arrow(361, 80, 428, 80) + text(394, 72, "no", 11, "#334155", "700")
    + diamond(510, 80, 160, 64, "VIDEO_EVENTO?")
    + arrow(510, 113, 510, 158) + text(522, 140, "sì", 11, "#334155", "700", "start")
    + box(420, 160, 180, 50, "Alla sessione id=…", "evento=fine la chiude", "blue")
    + arrow(591, 80, 668, 80) + text(629, 72, "no", 11, "#334155", "700")
    + box(670, 55, 210, 50, "Agli iscritti al tipo", "per esempio APPUNTI_CAMBIATI", "blue"),
    900, 230, "«FIG» — Lo smistamento dei messaggi del canale comandi")

S9 = p(c("Componente") + " (in " + c("componente.rs") + ") è un servizio avviato: " + c("avvia") + ", "
       + c("apri_canale") + ", " + c("manda") + ", " + c("richiesta") + ", " + c("ricevi") + ", " + c("chiudi")
       + ". In Phonestra lo si usa sempre attraverso " + c("Condiviso") + ", che lo tiene in un compito (" + c("smista")
       + ") e si clona: un servizio per collegamento, tante finestre. Quando sparisce l'ultima copia di "
       + c("Condiviso") + ", il servizio si chiude.", lead=True) + SMISTA + \
    table(["Metodo di " + c("Condiviso"), "A cosa serve"], [
        [c("domanda(tipo, dati)"), "Una domanda con risposta (5 s di tempo massimo; le domande scadute si controllano "
         "ogni 500 ms)"],
        [c("manda(tipo, dati)"), "Un messaggio senza risposta"],
        [c("apri_sessione"), c("VIDEO_APRI") + ": risposta ed eventi della sessione; gli eventi arrivati prima della "
         "risposta si tengono 2 s"],
        [c("dimentica(sessione)"), "Smettere di smistare gli eventi di una sessione chiusa"],
        [c("iscrivi(tipo)"), "Ricevere i messaggi spontanei di un tipo"],
        [c("mittente()"), "Un " + c("Mittente") + " clonabile: tocchi e tasti vanno direttamente in coda al canale "
         "comandi, senza passare dal compito"],
        [c("apritore()"), "Un " + c("Apritore") + " per aprire i canali " + c("audio") + " e " + c("video:<id>")],
        [c("nome_dispositivo()"), "Il modello dal " + c("CIAO") + " («telefono» se manca)"],
        [c("finito()") + ", " + c("vivo()"), "Sapere se il servizio è ancora vivo"],
        [c("chiudi()"), c("FINE") + ", attesa dell'uscita, chiusura dei canali"],
    ], "«TAB» — I metodi di " + c("Condiviso")) + \
    p(c("Collegamento::gira_componente") + " riavvia il servizio se muore col telefono ancora collegato, dopo 2 s. "
      "Dopo " + c("CADUTE_MASSIME") + " (3) avvii falliti o cadute smette di provare e pubblica il motivo ("
      + c("Collegamento::guasto") + "): drawer e finestre mostrano «Phonestra non parte sul telefono» con «Riconnetti ora».")

CHAPTER = ("Il componente sul telefono", [
    ("Che cos'è il componente", S1),
    ("Avvio del servizio", S2),
    ("Canali e preambolo", S3),
    ("Il canale comandi", S4),
    ("Battito e codici d'uscita", S5),
    ("I due custodi", S6),
    ("Contesto, API nascoste e autotest", S7),
    ("Sicurezza del servizio", S8),
    ("Lato PC: Componente e Condiviso", S9),
])
