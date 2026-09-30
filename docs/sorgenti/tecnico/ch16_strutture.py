from build import c, note, p, rif, table

S1 = p("Questa appendice raccoglie le strutture dati centrali del programma per il PC, con il file che le definisce e "
       "la sezione che le approfondisce. Sono i contratti che attraversano Phonestra; le classi Java del componente "
       "sono elencate nell'" + rif("Appendice B — Mappa dei file") + ".", lead=True) + \
    table(["Struttura", "Definita in", "Ruolo", "Sezione"], [
        "Collegamento e dati",
        [c("Collegamento"), c("collegamento.rs"), "Il telefono attivo: stato, " + c("Adb") + ", componente, pannello "
         "«in mano», notifiche", rif("Vita di un collegamento")],
        [c("Stato"), c("collegamento.rs"), c("Cerco") + ", " + c("Collegato") + ", " + c("Bloccato") + ", "
         + c("Perso") + ", " + c("Chiuso"), rif("Vita di un collegamento")],
        [c("Telefoni") + ", " + c("Telefono"), c("configurazione.rs"), "I telefoni configurati, in "
         + c("telefoni.toml"), rif("Configurazione e dati sul PC")],
        [c("Preferenze"), c("configurazione.rs"), "Le scelte dell'utente, in " + c("preferenze.toml"),
         rif("Preferenze")],
        [c("Notifica") + ", " + c("Info"), c("notifiche.rs"), "Notifiche, batteria e rete lette dai " + c("dumpsys"),
         rif("Notifiche e avvisi")],
        [c("App"), c("app.rs"), "Un'app del launcher: pacchetto, attività, nome, icona", rif("Il drawer")],
        "Client ADB",
        [c("Adb"), c("adb/mod.rs"), "Il collegamento: si clona, apre canali, esegue comandi brevi",
         rif("Canali e controllo di flusso")],
        [c("Canale") + ", " + c("Chiusore"), c("adb/mod.rs"), "Un canale verso un servizio; la sua chiusura da un "
         "altro compito", rif("Canali e controllo di flusso")],
        [c("Trasporto"), c("adb/mod.rs"), c("delayed_ack") + ", " + c("max_payload") + ", finestra",
         rif("Canali e controllo di flusso")],
        [c("Messaggio"), c("adb/messaggio.rs"), "Un messaggio ADB: intestazione di 24 byte e dati",
         rif("I messaggi di ADB")],
        [c("ShellV2"), c("adb/shell.rs"), "Un processo sul canale " + c("shell,v2"), rif("Il servizio shell,v2")],
        [c("Voce"), c("adb/sync.rs"), "Una voce di una cartella del telefono: nome, cartella o file, dimensione, "
         "data di modifica", rif("Copiare file: sync:")],
        "Componente, lato PC",
        [c("Componente"), c("componente.rs"), "Il servizio avviato: canale comandi, domande, chiusura",
         rif("Lato PC: Componente e Condiviso")],
        [c("Condiviso"), c("componente.rs"), "Il componente tenuto in un compito e clonabile: un servizio, tante "
         "finestre", rif("Lato PC: Componente e Condiviso")],
        [c("Mittente") + ", " + c("Apritore"), c("componente.rs"), "Messaggi senza risposta in coda al canale "
         "comandi; apertura dei canali " + c("audio") + " e " + c("video:<id>"), rif("Lato PC: Componente e Condiviso")],
        [c("Pronto") + ", " + c("Ciao"), c("componente.rs"), "La riga di pronto del servizio; il suo " + c("CIAO"),
         rif("Avvio del servizio")],
        [c("Messaggio"), c("componente.rs"), "Un messaggio del canale comandi: tipo, bandiere, id, contenuto",
         rif("Il canale comandi")],
        [c("SessioneNostra") + ", " + c("ComandiVideo"), c("video_nostro/mod.rs"), "Una sessione video e i suoi "
         "comandi", rif("Lato PC: dalla sessione alla finestra")],
        [c("Evento"), c("video_nostro/mod.rs"), "Gli eventi di una sessione: orientamento, protetta, spostata, "
         "rimosso, fine", rif("Eventi delle app")],
        [c("Opzioni") + ", " + c("Pacchetto"), c("video_nostro/flusso.rs"), "Le opzioni dello schermo; i pacchetti "
         "del canale video", rif("Il canale video:<id>")],
        [c("InputNostro"), c("input_nostro.rs"), "Tocchi, rotellina, tasti, testo e appunti codificati per il "
         + c("Mittente"), rif("Messaggi di input e appunti")],
        [c("Durate") + ", " + c("Orari") + ", " + c("Margine"), c("audio_nostro.rs"), "Durata, orario e momento di "
         "riproduzione di ogni pacchetto audio", rif("Riproduzione sul PC")],
        [c("Vista"), c("finestra.rs"), "Video, mouse e tastiera: il cuore di una finestra di app, usato anche per lo "
         "schermo nel drawer", rif("Le finestre delle app")],
    ], "«TAB» — Le strutture dati principali")

S2 = p("Tutti i messaggi del canale comandi in una tabella, per numero. Il formato di ciascuno è descritto nella "
       "sezione indicata.", lead=True) + \
    table(["Tipo", "Nome", "Verso", "Sezione"], [
        "Infrastruttura (0x01–0x0f) e prove (0x10–0x1f)",
        [c("0x01"), c("CIAO"), "servizio → PC", rif("Il canale comandi")],
        [c("0x02"), c("BATTITO"), "nei due sensi", rif("Battito e codici d'uscita")],
        [c("0x03"), c("FINE"), "PC → servizio", rif("Il canale comandi")],
        [c("0x04"), c("ERRORE"), "servizio → PC, come risposta", rif("Il canale comandi")],
        [c("0x10"), c("PROVA_CUSTODE"), "PC → servizio", rif("Il canale comandi")],
        "Video e pannello (0x40–0x4f)",
        [c("0x40") + "–" + c("0x45"), c("VIDEO_APRI") + ", " + c("VIDEO_CHIUDI") + ", " + c("VIDEO_AVVIA_APP") + ", "
         + c("VIDEO_RIDIMENSIONA") + ", " + c("VIDEO_CHIAVE") + ", " + c("VIDEO_PANNELLO"), "PC → servizio",
         rif("Messaggi video")],
        [c("0x46"), c("VIDEO_EVENTO"), "servizio → PC, spontaneo", rif("Eventi delle app")],
        "Input e appunti (0x50–0x5f)",
        [c("0x50") + "–" + c("0x54"), c("TOCCHI") + ", " + c("ROTELLINA") + ", " + c("TASTO") + ", " + c("TESTO")
         + ", " + c("INDIETRO"), "PC → servizio, senza risposta", rif("Messaggi di input e appunti")],
        [c("0x55") + "–" + c("0x57"), c("APPUNTI_SCRIVI") + ", " + c("APPUNTI_LEGGI") + ", " + c("APPUNTI_ASCOLTA"),
         "PC → servizio", rif("Appunti")],
        [c("0x58"), c("APPUNTI_CAMBIATI"), "servizio → PC, spontaneo", rif("Appunti")],
        [c("0x5c") + ", " + c("0x5d"), c("CONTEGGI") + ", " + c("PROVA"), "PC → servizio (diagnosi e prove)",
         rif("Messaggi di input e appunti")],
    ], "«TAB» — Indice dei messaggi del canale comandi") + \
    note("il test " + c("nessun_tipo_usato_da_due_moduli") + " controlla che ogni numero sia unico, nella fascia "
         "giusta e uguale in Java e in Rust (" + rif("Un messaggio nuovo sul canale comandi") + ").")

S3 = p("Ogni flusso di byte di Phonestra comincia con un'intestazione fissa. Qui sono messe a confronto.", lead=True) + \
    table(["Flusso", "Intestazione", "Ordine dei byte", "Sezione"], [
        ["Messaggio ADB", "24 byte: comando, " + c("arg0") + ", " + c("arg1") + ", lunghezza, somma, comando xor "
         + c("0xffffffff"), "little-endian", rif("I messaggi di ADB")],
        ["Pacchetto " + c("shell,v2"), c("id u8 · lunghezza u32"), "little-endian", rif("Il servizio shell,v2")],
        ["Preambolo di un canale del servizio", c("segreto (16 byte) · lunghezza del tipo u8 · tipo ASCII"), "—",
         rif("Canali e preambolo")],
        ["Canale comandi", "8 byte: " + c("tipo u8 · bandiere u8 · id u16 · lunghezza u32"), "big-endian",
         rif("Il canale comandi")],
        ["Canale " + c("video:<id>"), "12 byte: " + c("pts u64 · lunghezza u32") + ", o " + c("0x80000000")
         + " e la misura nuova", "big-endian", rif("Il canale video:<id>")],
        ["Canale audio", "12 byte: " + c("orario u64 · lunghezza u32"), "big-endian", rif("Pacchetti audio")],
    ], "«TAB» — Le intestazioni dei flussi")

CHAPTER = ("Appendice A — Strutture dati e messaggi", [
    ("Indice delle strutture dati", S1),
    ("Indice dei messaggi", S2),
    ("Le intestazioni dei flussi", S3),
])
