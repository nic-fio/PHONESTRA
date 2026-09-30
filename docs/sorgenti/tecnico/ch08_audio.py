from build import arrow, box, c, fig, flow, p, path, rif, table, text, warn

FILI = fig(
    box(20, 40, 150, 60, "audio-lettura", "priorità −19", "navy")
    + box(200, 40, 150, 60, "audio-codifica", "AAC 192 kbit/s", "blue")
    + box(380, 40, 150, 60, "Coda", "256 pacchetti, ~5 s", "amber")
    + box(560, 40, 150, 60, "audio-spedizione", "scrive sul socket", "blue")
    + box(740, 40, 140, 60, "Canale audio", "verso il PC", "dark")
    + arrow(172, 70, 198, 70) + arrow(352, 70, 378, 70) + arrow(532, 70, 558, 70) + arrow(712, 70, 738, 70)
    + path([(95, 102), (95, 170), (455, 170), (455, 104)], "#475569", True) + text(275, 190, "PCM: senza codifica", 11)
    + box(620, 140, 260, 54, "audio-sentinella", "legge dal socket: vede la chiusura", "light")
    + arrow(810, 102, 810, 138, "#475569", True),
    900, 206, "«FIG» — I quattro thread dell'audio sul telefono (CanaleAudio.java)")

S1 = p("L'audio del telefono suona dalle casse del PC, senza interruzioni e in sincrono col video. La ricetta è venuta "
       "dalle misure: cattura loopback, AAC, orari dal conteggio dei campioni e un ordine d'avvio preciso.",
       lead=True) + \
    table(["Scelta", "Perché"], [
        ["Cattura loopback: " + c("AudioPolicy") + " con " + c("ROUTE_FLAG_LOOP_BACK") + " sugli usi dei suoni "
         "(media, giochi, assistente, navigazione, suoni di sistema…: " + c("Audio.USI") + ")",
         "Il telefono intanto tace; tolta la politica, torna a suonare da sé. Nessuna impostazione da rimettere."],
        ["AAC-LC 192 kbit/s, 48 kHz stereo (codificatore software di Android)", "Pulito quanto il PCM (misure §42) e "
         "molto più leggero sul Wi-Fi. PCM come riserva di prova."],
        ["Orari dal conteggio dei campioni (campioni × 10⁶ / 48000)", "Regolari: l'ora di uscita dal codificatore "
         "arriva a raffiche (1–3 ms e 30–40 ms invece di 21)."],
        ["Lettura a priorità −19, e lettura, codifica e spedizione su thread separati", "Se il Wi-Fi o il codificatore "
         "rallentano, la lettura non si ferma e non si perdono campioni."],
    ], "«TAB» — La ricetta dell'audio") + \
    p("Sul telefono (" + c("CanaleAudio.java") + ", che usa le classi dello strumento di misura " + c("Audio.java")
      + ") ci sono quattro thread: " + c("audio-lettura") + ", " + c("audio-codifica") + ", " + c("audio-spedizione")
      + " (coda di 256 pacchetti, circa 5 s, che scarta i più vecchi, contati in " + c("persi") + ") e "
      + c("audio-sentinella") + ", che legge dal socket solo per accorgersi della chiusura. Ogni thread cattura i "
      "propri errori: un problema dell'audio chiude il canale con una riga " + c("errore") + ", mai il servizio. Un "
      "solo canale audio alla volta: uno nuovo ferma il vecchio e aspetta (al massimo 3 s) che abbia tolto la sua "
      "politica.") + FILI

S2 = p("Il canale audio va solo dal telefono al PC: " + c("orario u64 BE · lunghezza u32 BE · dati") + ". Il PC "
       "non manda niente; chiudere il canale ferma la cattura.", lead=True) + \
    table(["Bandiera nell'orario", "Contenuto"], [
        ["bit 61", "Testo UTF-8 tipo " + c("chiave=valore …") + ". Il primo pacchetto è sempre " + c("inizio")
         + " (" + c("formato=aac frequenza=48000 canali=2 bitrate=192000 sorgente=loopback buffer_ms=… "
         "registrazione=istanza|statica") + ") oppure " + c("errore …") + "; poi " + c("lettura tid=… nice=…")
         + ", " + c("misura") + " (una al secondo), " + c("avviso") + ", " + c("errore") + "."],
        ["bit 62", "Configurazione del codec (" + c("AudioSpecificConfig") + ", 2 byte " + c("11 90")
         + "), prima di qualsiasi dato."],
        ["nessuna", "Dati: un frame AAC di 1024 campioni (21,333 ms), o 1024 campioni PCM."],
    ], "«TAB» — I pacchetti del canale audio")

S3 = p("Sul PC tre pezzi danno a ogni pacchetto il suo orario e decidono quando suonarlo; poi lo decodifica e lo "
       "suona GStreamer.", lead=True) + flow([("Flusso::apri", "canale audio", "navy"), ("Orari + Durate", "orari regolari", "blue"),
           ("Margine", "quando suonare", "blue"), ("appsrc", "AAC raw, codec_data", "blue"),
           ("avdec_aac", "audioconvert, resample", "blue"), ("autoaudiosink", "casse del PC", "light")],
          "«FIG» — L'audio dal canale alle casse; una copia dei pacchetti va alla registrazione", width=960) + \
    table(["Pezzo", "Che cosa fa"], [
        [c("Durate"), "La durata di ogni pacchetto dal conteggio dei campioni (21 333 o 21 334 µs, senza errore accumulato)."],
        [c("Orari"), "Tiene gli orari regolari e riallinea solo oltre 60 ms di scostamento."],
        [c("Margine"), "Decide quando suonare ogni pacchetto: orario del telefono + uno scarto fissato dal primo "
         "pacchetto. Parte da 80 ms; un pacchetto in ritardo (meno di 10 ms prima di adesso) sposta tutto più avanti "
         "(un attimo di silenzio, poi niente buchi) e allarga il margine di 40 ms, fino a 300. Un telefono più avanti "
         "di 200 ms oltre il margine fa riallineare."],
        [c("Margine::scendi"), "Dopo 10 s di calma riduce il margine saltando un pacchetto di «silenzio» (meno del 40 % "
         "della media dei byte). Con l'AAC la dimensione dei pacchetti è quasi costante e non scatta mai ("
         + rif("Appendice C — Problemi noti") + ")."],
    ], "«TAB» — I pezzi della riproduzione") + \
    p(c("audio_nostro::riproduci(apritore)") + " è la funzione che il collegamento chiama; finisce se il canale si "
      "chiude e, annullata, lo chiude. " + c("PHONESTRA_AUDIO_CODEC=pcm") + " (o " + c("raw") + ") usa il PCM al "
      "posto dell'AAC.")

S4 = p("La cattura audio non parte insieme al collegamento: aspetta lo specchio del drawer. È una regola trovata "
       "con le misure.", lead=True) + warn("la cattura audio parte dopo lo specchio dello schermo del drawer, e 5 s dopo che lo specchio è aperto ("
          + c("ATTESA_SPECCHIO") + " 10 s al massimo, " + c("ASSESTAMENTO") + " 5 s, in " + c("collegamento.rs")
          + "). Se lo specchio si ricrea, la cattura riparte dopo 300 ms. Con la cattura avviata prima che le sessioni "
          "iniziali del collegamento fossero partite, il lettore dei reel di Facebook nelle finestre restava a secco "
          "e l'audio aveva micro-interruzioni; verificato a prove alternate (misure §48–49). Il meccanismo interno di "
          "Android non è ancora capito: non cambiare quest'ordine senza rifare quelle prove.",
          "Regola da non perdere.") + \
    p("Il drawer segnala l'apertura dello specchio con " + c("Collegamento::specchio_aperto()") + ", che incrementa "
      "un contatore " + c("watch") + " seguito dal compito dell'audio. Il compito guarda solo gli specchi aperti "
      "dopo l'avvio del servizio attuale: dopo un ricollegamento aspetta lo specchio nuovo (prove §53).")

S5 = p("La cattura si toglie da sola in tutti i modi in cui il servizio può finire, anche il più brusco.",
       lead=True) + table(["Come finisce", "Chi toglie la politica audio"], [
    ["Il PC chiude il canale, o se ne apre uno nuovo", c("CanaleAudio") + ": ferma il registratore e chiama "
     + c("unregisterAudioPolicy")],
    ["Il servizio esce con " + c("System.exit"), "Un gancio di chiusura (" + c("audio-fine") + ")"],
    ["Il servizio muore di colpo (" + c("kill -9") + ")", "Android: la politica è legata con " + c("linkToDeath")
     + " al nostro processo"],
], "«TAB» — Chi toglie la cattura") + \
    p("Il custode non ha un'azione per l'audio: nessun comando di shell toglie la politica di un altro processo, e non "
      "serve. Alla chiusura di Phonestra, prima di staccare l'audio, il collegamento mette in pausa i media che stanno "
      "suonando, altrimenti ripartirebbero dall'altoparlante del telefono (" + rif("La chiusura") + ").")

S6 = p("«Registra» nella finestra di un'app scrive un MP4: il video H.264 così com'è (" + c("h264parse ! mp4mux")
       + ") e l'audio AAC così com'è, senza ricodificare. " + c("audio_nostro::ascolta()") + " dà una copia dei "
       "pacchetti (" + c("broadcast") + "), " + c("caps_registrazione()") + " le caps con il " + c("codec_data")
       + " dell'audio in corso. Si comincia dal primo fotogramma chiave; senza audio AAC in corso il file resta senza "
       "audio.", lead=True) + \
    p("Durante la registrazione il display dell'app non cambia misura e non si ricrea per l'orientamento (la finestra "
      "scala l'immagine), e il pulsante mostra il tempo trascorso (" + c("● m:ss") + "). Il file va in "
      + c("<Video di XDG>/Phonestra/<app> AAAA-MM-GG HH.MM.SS.mp4") + ".")

CHAPTER = ("Audio", [
    ("La ricetta", S1),
    ("Pacchetti audio", S2),
    ("Riproduzione sul PC", S3),
    ("L'ordine d'avvio", S4),
    ("Chi toglie la cattura", S5),
    ("Registrazione", S6),
])
