from build import c, code, flow, note, p, rif, steps, table

S1 = p("Ogni finestra di app è uno schermo virtuale sul telefono, grande quanto la finestra, con l'app avviata lì. Lo "
       "schermo del telefono disegnato nel drawer è invece lo specchio dello schermo principale. Il telefono codifica "
       "in H.264 con l'hardware, il PC decodifica con GStreamer.", lead=True) + \
    table(["", "Schermo virtuale (finestra di un'app)", "Specchio (drawer)"], [
        ["Come si crea", c("createVirtualDisplay(nome, l, a, dpi, null, flag)") + " con i flag di "
         + c("Sistema.FLAG_PROPOSTI") + ": PUBLIC, PRESENTATION, OWN_CONTENT_ONLY, SUPPORTS_TOUCH, "
         "ROTATES_WITH_CONTENT, DESTROY_CONTENT_ON_REMOVAL, TRUSTED, OWN_DISPLAY_GROUP, OWN_FOCUS, "
         "TOUCH_FEEDBACK_DISABLED", c("DisplayManager.createVirtualDisplay") + " statica nascosta (permesso "
         + c("CAPTURE_VIDEO_OUTPUT") + ")"],
        ["Misura", "Quella chiesta dal PC (720×1280 a 320 dpi se manca), allineata a 8 e all'allineamento del "
         "codificatore prima di crearlo", "Quella dello schermo principale, ridotta a 1920 di lato"],
        ["Orientamento", "Bloccato: " + c("cmd window set-ignore-orientation-request") + " e " + c("user-rotation lock 0")
         + " sul display", "Segue il telefono: ogni 500 ms si rilegge la misura; se ruota, nuovo codificatore e nuovo specchio"],
        ["Ridimensionabile", "Sì (" + c("VirtualDisplay.resize") + ")", "No: la finestra scala l'immagine; "
         + c("VIDEO_RIDIMENSIONA") + " risponde «specchio: misura dello schermo del telefono»"],
        ["Numero del display", "Nella risposta di apertura", "0 (lo schermo principale); niente eventi di "
         "orientamento, ma quelli della schermata protetta sì"],
    ], "«TAB» — Le due specie di sessione video") + \
    p("Sul PC la misura in pixel viene da " + c("finestra::pixel") + ": la misura della finestra in punti (1 punto del "
      "PC = 1 dp del telefono) per la densità del telefono, entro 2560 di lato e in multipli di 8. Il display ha la "
      "stessa densità del telefono, perché alcune app (Facebook) disegnano certi elementi con la densità dello "
      "schermo vero e con densità diverse uscirebbero giganti. Per le app solo verticali in una finestra larga il "
      "display è una colonna con la forma del telefono (" + c("finestra::misura") + ").")

S2 = p("Contenuto a righe " + c("chiave=valore") + ". Le domande si eseguono in ordine su un thread video del "
       "servizio: il canale comandi (battito, input) non aspetta mai il video.") + \
    table(["Tipo", "Nome", "Domanda (PC → servizio)", "Risposta"], [
        [c("0x40"), c("VIDEO_APRI"), c("larghezza altezza dpi codec") + " oppure " + c("specchio=1 lato_massimo codec")
         + "; facoltativi " + c("app=") + ", " + c("informazioni=") + "; interruttori di prova, "
         "validi per tutto il servizio: " + c("max_fps=") + ", " + c("priorita=") + ", " + c("protetta="),
         c("id display codec larghezza altezza") + " (misura già allineata), più " + c("avvio=<esito>")
         + " se c'era " + c("app=") + " o " + c("informazioni=")],
        [c("0x41"), c("VIDEO_CHIUDI"), c("id [togli_task=1]"), "vuota, a chiusura fatta"],
        [c("0x42"), c("VIDEO_AVVIA_APP"), c("id app=<pacchetto>") + " o " + c("id informazioni=<pacchetto>"), "esito dell'avvio"],
        [c("0x43"), c("VIDEO_RIDIMENSIONA"), c("id larghezza altezza"), "misura " + c("LxA") + " o misura invariata"],
        [c("0x44"), c("VIDEO_CHIAVE"), c("id"), "vuota"],
        [c("0x45"), c("VIDEO_PANNELLO"), c("acceso=0|1"), c("schermi=<quanti>")],
        [c("0x46"), c("VIDEO_EVENTO"), "—", "spontaneo: " + c("evento=<nome> id=<sessione> …")],
    ], "«TAB» — I messaggi video") + \
    p("Il PC aspetta la risposta solo di " + c("APRI") + " e " + c("CHIUDI") + "; gli altri comandi non la aspettano "
      "e un errore finisce nel registro. " + c("informazioni=<pacchetto>") + " apre la pagina «Informazioni sull'app» "
      "delle Impostazioni invece dell'app. Il codec predefinito è " + c("h264") + "; il componente accetta anche "
      + c("h265") + ", usato solo dalle prove.")

S3 = p("Il PC lo apre subito dopo " + c("VIDEO_APRI") + " (entro 10 s). Il codificatore parte quando il canale è "
       "aperto, così il primo pacchetto è la misura, poi i parametri, poi il primo fotogramma chiave, e non si perde "
       "niente. Il PC non ci scrive; se lo chiude, la sessione si chiude (senza togliere l'app dalle recenti). Ogni "
       "pacchetto ha un'intestazione di 12 byte big-endian:") + \
    table(["Pacchetto", "Intestazione", "Poi"], [
        ["Misura nuova", c("0x80000000 · larghezza u32 · altezza u32"), "niente"],
        ["Dati", c("pts u64") + " (µs dal primo fotogramma; bit 62 = parametri del codec, bit 61 = fotogramma "
         "chiave) · " + c("lunghezza u32"), "i dati in Annex B, come escono da " + c("MediaCodec")],
    ], "«TAB» — I pacchetti del canale video") + \
    p("Lato PC " + c("video_nostro::flusso::leggi_pacchetto") + " li legge in un compito dedicato: una lettura "
      "interrotta a metà dentro un " + c("select!") + " perderebbe byte. Il thread di lettura del telefono scrive ogni "
      "pacchetto intero con una sola " + c("write") + ".")

S4 = p(c("Codifica.java") + " prende il primo codificatore hardware (non alias) per il tipo, con i valori misurati "
       "(misure §43): 8 Mbit/s, 60 fotogrammi al secondo dichiarati, fotogramma chiave ogni 10 s, ripetizione dopo "
       "100 ms, priorità tempo reale, gamma limitata, più " + c("prepend-sps-pps-to-idr-frames") + " perché ogni "
       "fotogramma chiave porti davanti i parametri; " + c("max-fps-to-encoder") + " solo con l'interruttore di prova "
       + c("max_fps") + ".") + \
    p("Se " + c("configure") + " rifiuta il formato, si riprova in quest'ordine: hardware senza "
      + c("prepend-sps-pps-to-idr-frames") + ", poi il codificatore predefinito di Android con e senza.") + \
    p("Il fotogramma chiave serve quando una finestra riparte o comincia una registrazione ("
      + c("ricomincia_video") + " → " + c("VIDEO_CHIAVE") + "). Si chiede con " + c("REQUEST_SYNC_FRAME")
      + ", senza ricreare niente: circa 0,1 s invece delle ripartenze di 1–2 s di scrcpy.") + \
    note("il codificatore Qualcomm (" + c("c2.qti.avc.encoder") + ") ignora " + c("repeat-previous-frame-after")
         + ": a schermo fermo non esce niente, e la richiesta del fotogramma chiave aspetterebbe il prossimo "
         "cambiamento (una finestra appena aperta resterebbe nera). Se il fotogramma chiave non esce entro 80 ms, "
         + c("SessioneVideo") + " stacca e riattacca la " + c("Surface") + " del codificatore ("
         + c("VirtualDisplay.setSurface(null)") + " e di nuovo la sua), che fa comporre subito un fotogramma; se "
         "ancora niente, una seconda volta dopo altri 160 ms.", "Schermo fermo: il ridisegno forzato.")

S5 = steps([
    "<b>La finestra cambia misura.</b> Una funzione legata al ridisegno di GTK manda la misura nuova alla sessione a "
    "ogni cambiamento, senza aspettare.",
    "<b>Il PC decide.</b> Durante una registrazione la misura non cambia (" + rif("Registrazione") + "); per lo "
    "specchio non si fa niente. Se col tetto dei 2560 pixel la scala giusta si allontana di oltre il 15% da quella "
    "del display, il display non basta ridimensionarlo: la sessione finisce con " + c("FineSessione::Ricrea")
    + " e si ricrea dopo 300 ms, quando la finestra ha smesso di cambiare misura. Altrimenti "
    + c("VIDEO_RIDIMENSIONA") + ".",
    "<b>Il telefono ridimensiona.</b> Una misura = un codificatore. Se la misura allineata non cambia, niente. Se "
    "cambia: si prepara il codificatore nuovo, si manda la misura al PC, " + c("VirtualDisplay.resize") + " + "
    + c("setSurface") + ", poi si chiude il vecchio; i suoi pacchetti ancora in volo si scartano.",
]) + p("Un evento di input calcolato sulla misura vecchia viene scartato dal telefono (" + rif("Iniezione") + ").")

S6 = p(c("EventiApp.java") + " registra un " + c("TaskStackListener") + " finché c'è almeno una sessione. Ogni evento "
       "programma un controllo 150 ms dopo (gli eventi arrivano a gruppi); in più un controllo ogni 3 s, perché una "
       "finestra protetta può comparire senza eventi dei task.") + \
    table(["Evento", "Coppie", "Quando", "Cosa fa il PC"], [
        [c("orientamento"), c("display verticale=0|1 valore=N"), "Al primo controllo, poi quando l'app passa da "
         "verticale a orizzontale o viceversa (non a ogni cambio di valore); mai per lo specchio",
         "Finestra a misura fissa 9:16 o colonna"],
        [c("protetta"), c("display protetta=0|1"), "Al primo controllo, poi quando cambia",
         "Messaggio al posto dell'immagine nera"],
        [c("spostata"), c("task display"), "Un task passa su un altro schermo (app aperta sul telefono)",
         "Niente (solo diagnosi)"],
        [c("rimosso"), c("task"), "Un task dello schermo si chiude", "Niente (solo diagnosi)"],
        [c("fine"), c("motivo"), "Il telefono chiude la sessione da sé", "La sessione riparte come dopo una caduta"],
    ], "«TAB» — Gli eventi delle app") + \
    p("La schermata protetta si riconosce senza " + c("dumpsys") + " (" + c("Protetta.java") + "): "
      + c("captureDisplay") + " rimpicciolita al 5 % e " + c("containsSecureLayers()") + ", con 2 s di tempo "
      "massimo. Phonestra non aggira le protezioni: mostra un messaggio.")

S7 = code("""
let SessioneNostra { display, video: mut flusso, mut comandi, mut eventi, .. } =
    SessioneNostra::avvia(&servizio, &Opzioni { display: (l, a, dpi), ..Opzioni::default() }).await?;
comandi.avvia_app("com.android.chrome").await?;
while let Ok(p) = leggi_pacchetto(&mut flusso).await { /* Pacchetto::Dimensione o Pacchetto::Dati */ }
comandi.ridimensiona(l, a).await?;
comandi.ricomincia_video().await?;
while let Some(e) = eventi.recv().await { /* Evento::Orientamento, Protetta, Spostata, Rimosso, Fine */ }
comandi.chiudi(true).await?;   // true = via dalle recenti (l'utente ha chiuso la finestra)
""", "rust", "Una sessione video dal PC") + \
    flow([("SessioneVideo", "display → codificatore", "navy"), ("leggi_pacchetto", "compito tokio", "blue"),
          ("appsrc", "h264parse", "blue"), ("decodebin", "videoconvert", "blue"),
          ("gtk4paintablesink", "GtkPicture", "light")],
         "«FIG» — Il percorso di un fotogramma, dal telefono alla finestra; dal primo fotogramma chiave una copia va "
         "alla registrazione (" + c("h264parse ! mp4mux") + ")") + \
    p(c("finestra::vista") + " tiene la pipeline e rifà la sessione quando serve: collegamento caduto, componente "
      "riavviato, display da ricreare, orientamento cambiato (" + c("FineSessione::{Chiusa, Ricrea, Caduta}")
      + "). I parametri del codec (SPS/PPS) vanno uniti al fotogramma successivo prima dell'" + c("appsrc") + ".")

CHAPTER = ("Video", [
    ("Sessioni: schermo virtuale e specchio", S1),
    ("Messaggi video", S2),
    ("Il canale video:<id>", S3),
    ("Codificatore e fotogramma chiave", S4),
    ("Ridimensionamento", S5),
    ("Eventi delle app", S6),
    ("Lato PC", S7),
])
