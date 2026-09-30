from build import arrow, box, c, fig, note, p, path, rif, steps, table, term, text, warn

S1 = p("Mentre si usano le app dal PC lo schermo del telefono si spegne, ma il telefono resta sveglio e sbloccato: "
       "così le app negli schermi virtuali continuano a girare e a ricevere i tocchi. Quando l'utente riprende in "
       "mano il telefono, il pannello si riaccende; quando lo lascia lì, si rispegne.", lead=True) + \
    p(c("Pannello.java") + " chiama " + c("SurfaceControl.setDisplayPowerMode(token, 0|2)") + " su ogni schermo "
      "fisico: agisce solo sul compositore, Android crede lo schermo acceso e qualunque cambio di stato (tasto di "
      "accensione) lo riaccende. Da Android 14 i token stanno in " + c("DisplayControl") + ", dentro "
      + c("services.jar") + ", caricato con un class loader sul " + c("SYSTEMSERVERCLASSPATH") + " e la libreria "
      + c("android_servers") + ".") + \
    p("Il PC lo comanda con " + c("VIDEO_PANNELLO") + " (" + rif("Messaggi video") + "): "
      + c("video_nostro::pannello") + " lo manda senza aspettare la risposta, " + c("ComandiVideo::pannello")
      + " lo manda da una sessione. Il pannello è uno per telefono, non uno per finestra: chi decide se spegnerlo è "
      "il " + c("Collegamento") + ", con la regola del telefono «in mano» descritta più sotto. Quando il servizio "
      "muore, l'azione 400 del custode lo riaccende (" + rif("I due custodi") + ").")

S2 = p("I Samsung cambiano da soli la frequenza del display (10–120 Hz): dopo un attimo di calma è a 24 Hz. Allo "
       "spegnimento SurfaceFlinger passa a 60 Hz, ma conferma il cambio solo coi vsync del pannello, che non arrivano "
       "più: il suo modello resta a 24 Hz, prende a quel ritmo i fotogrammi degli schermi virtuali e le app che "
       "disegnano più in fretta restano ferme ad aspettarlo (Facebook, coi reel AV1 decodificati in software, lascia "
       "a secco l'audio). Prove §59.", lead=True) + steps([
    "Prima di spegnere, " + c("Pannello.java") + " legge " + c("mDisplayModePtr") + " e "
    + c("mPeriodConfirmationInProgress") + " da " + c("dumpsys SurfaceFlinger") + ".",
    "Se il modello non è confermato ad almeno 60 Hz, mette " + c("min_refresh_rate=60") + " e registra presso il "
    "custode l'azione 410 che rimette il valore di prima.",
    "Aspetta la conferma, al massimo 1 s (" + c("ATTESA_60_MS") + "), poi spegne.",
    "Rimette subito " + c("min_refresh_rate") + " com'era e toglie l'azione del custode.",
])

STATI = fig(
    box(60, 90, 270, 70, "Pannello spento", "si usa il telefono dal PC", "navy")
    + box(570, 90, 270, 70, "Pannello acceso, «in mano»", "Collegamento::pannello_a_mano()", "amber")
    + path([(330, 108), (570, 108)], "#d97706") + text(450, 40, "sblocco a mano · chiamata in arrivo", 11, "#9a3412", "600")
    + text(450, 58, "fine chiamata · ultima sessione chiusa", 11, "#9a3412", "600")
    + text(450, 76, "ricollegamento dopo un blocco", 11, "#9a3412", "600")
    + path([(570, 142), (330, 142)], "#0050C0") + text(450, 184, "un tocco, un tasto o un clic dal PC (usa_dal_pc)", 11,
                                                          "#003a90", "600")
    + text(450, 202, "fermo per il tempo di spegnimento dell'utente, senza chiamate", 11, "#003a90", "600")
    + text(195, 240, "le sessioni che si aprono spengono il pannello", 11, "#475569", "400")
    + text(705, 240, "le sessioni che si aprono lo lasciano acceso", 11, "#475569", "400"),
    900, 260, "«FIG» — I due stati del pannello e che cosa li cambia")

S3 = p("Lo stato è un solo valore nel " + c("Collegamento") + ": " + c("a_mano") + ", letto con "
       + c("Collegamento::pannello_a_mano") + ". Con " + c("a_mano") + " falso le sessioni che si aprono spengono il "
       "pannello; con " + c("a_mano") + " vero lo lasciano acceso. Dura quanto il processo, attraverso i "
       "ricollegamenti: una caduta non lo azzera.", lead=True) + STATI + \
    table(["Evento", "Dove", "Effetto"], [
        ["Il telefono, bloccato durante l'uso, viene sbloccato", c("Collegamento::sbloccato"), "«in mano»: il pannello "
         "resta acceso"],
        ["Primo controllo dopo un collegamento, telefono sbloccato", c("Collegamento::sbloccato"), "«in mano» solo se "
         "prima c'era stato un blocco, o una caduta col telefono addormentato (" + rif("Cadute e ricollegamenti") + ")"],
        ["Chiamata in arrivo", "giro dei 3 s", "pannello acceso, «in mano» (" + rif("Chiamate") + ")"],
        ["Fine di una chiamata a pannello spento", "giro dei 3 s", "pannello acceso, «in mano»"],
        ["Chiusura dell'ultima sessione", c("finestra.rs") + ", " + c("pannello_acceso_senza_finestre"),
         "pannello acceso, «in mano»"],
        ["Tocco, tasto, scorrimento, testo, incolla, zoom, pressione lunga o Indietro da una finestra",
         c("Collegamento::usa_dal_pc"), "se era «in mano»: pannello spento, non più «in mano»"],
        ["Telefono «in mano» fermo per il tempo di spegnimento dell'utente, senza chiamate in quel tempo",
         "giro dei 3 s", "pannello spento, non più «in mano»"],
    ], "«TAB» — Chi accende e chi spegne il pannello") + \
    p("Lo spegnimento senza tocchi esiste perché il tempo di spegnimento del telefono, durante il collegamento, è al "
      "massimo (" + rif("I due custodi") + "): un telefono sbloccato a mano e poi lasciato sul tavolo non si "
      "spegnerebbe più. Il giro dei 3 s, finché il telefono è «in mano», chiede " + c("dumpsys power | grep -m1 "
      "lastUserActivityTime=") + " e legge «(N ms ago)» (" + c("fermo_da") + "): passato il tempo scelto dall'utente "
      "(quello salvato all'inizio del collegamento), spegne il pannello. Vale anche senza finestre aperte: il telefono "
      "resta sveglio e sbloccato (prove §58 e §60).") + \
    warn("il contatore delle sessioni (" + c("Collegamento::sessioni") + ") conta anche lo specchio del drawer. "
         "«Ultima sessione» vuol dire quindi l'ultima finestra <i>e</i> il drawer: col drawer aperto, chiudere "
         "l'ultima finestra di un'app non riaccende il pannello. Anche l'apertura dello specchio spegne il pannello, "
         "come quella di una finestra.", "Lo specchio conta come una sessione.")

S4 = p("Il giro dei 3 s legge lo stato delle chiamate con " + c("dumpsys telephony.registry | grep -o "
       "'mCallState=[12]'") + ": 1 vuol dire che squilla, 2 che è in corso. Con due SIM c'è una riga per SIM, e "
       "basta una riga per contare.", lead=True) + \
    table(["Momento", "Regola", "Perché"], [
        ["Comincia a squillare", "Se il telefono non è già «in mano», pannello acceso e «in mano»", "Per rispondere "
         "col telefono in mano (prove §58)."],
        ["Durante la chiamata", "Il tempo senza tocchi si conta dall'ultimo giro con una chiamata "
         "(" + c("chiamata_alle") + "): il pannello non si spegne", "Col telefono all'orecchio non ci sono tocchi."],
        ["Finisce, e il telefono non era «in mano»", "Pannello acceso e «in mano»; poi vale lo spegnimento senza "
         "tocchi", "Risposta con un clic dal PC: durante la chiamata Android riaccende il pannello da solo (sensore "
         "di prossimità), e Phonestra lo crederebbe spento; restava acceso per sempre (prove §60)."],
    ], "«TAB» — Le chiamate e il pannello") + \
    warn("le variabili delle chiamate (" + c("squillava") + ", " + c("in_chiamata") + ", " + c("chiamata_alle")
         + ") ripartono da zero a ogni collegamento. Prima di riavviare Phonestra per una prova, guarda tutte le "
         "righe " + c("mCallState") + ": un riavvio durante una chiamata lascia lo specchio nero (prove §61).",
         "Niente riavvii in chiamata.")

S5 = p("Sui Samsung il blocco del telefono fa cadere il Debug wireless: il collegamento si riapre allo sblocco. Ma "
       "il collegamento cade anche per la rete, col telefono ancora sbloccato sul tavolo. Nei due casi il pannello "
       "al ritorno è acceso (l'ha riacceso il custode o l'utente), e " + c("Collegamento::sbloccato") + " deve capire "
       "chi è stato.", lead=True) + steps([
    "Alla caduta, " + c("mantieni") + " annota l'ora (" + c("caduto") + "), solo la prima volta e solo se il "
    "collegamento era davvero aperto.",
    "Al primo controllo del blocco dopo il ricollegamento, se il telefono è sbloccato: se prima della caduta il giro "
    "aveva visto il blocco (" + c("bloccato_durante_uso") + "), l'ha sbloccato l'utente: «in mano».",
    "Altrimenti " + c("dumpsys power | grep -m1 mLastSleepTime=") + " dice da quanto il telefono si è addormentato "
    "(" + c("dormito_da") + "). Se è successo da non più del tempo trascorso dalla caduta più "
    + c("MARGINE_CADUTA") + " (15 s, abbondanti rispetto agli 8 s entro cui il PC si accorge della caduta: 5 s di "
    "attesa della risposta più 3 s fra i controlli), il telefono ha "
    "dormito e l'ha sbloccato l'utente: «in mano».",
    "Se ha dormito prima, la caduta era di rete: il pannello si rispegne con le sessioni che ripartono. Se la riga "
    "manca o il comando non risponde, nel dubbio «in mano».",
]) + p("«Sbloccato a mano» si decide dal primo controllo del blocco dopo il collegamento, non dal solo "
       "ricollegamento (prove §55 e §57).")

S6 = p("Ogni cambio del pannello lascia una riga nel registro di Phonestra, che finisce nel journal del PC:",
       lead=True) + term("""
$ journalctl --user --since today | grep -i phonestra
""") + table(["Riga", "Quando"], [
    [c("[collegamento] sbloccato a mano: il pannello resta acceso"), "sblocco a mano, anche dopo una caduta"],
    [c("[collegamento] caduta senza blocco: il pannello si rispegne"), "ricollegamento dopo una caduta di rete"],
    [c("[collegamento] chiamata in arrivo: pannello acceso"), "comincia a squillare"],
    [c("[collegamento] fine della chiamata: pannello acceso, si rispegne senza tocchi"), "fine di una chiamata a "
     "pannello spento"],
    [c("[collegamento] telefono in mano non toccato da N s: pannello spento"), "spegnimento senza tocchi"],
    [c("[finestra] usato dal PC: pannello spento"), "primo tocco dal PC col telefono «in mano»"],
    [c("[finestra] ultima finestra chiusa: pannello acceso"), "chiusura dell'ultima sessione"],
], "«TAB» — Le righe del registro dei cambi del pannello") + \
    note("lo spegnimento all'apertura di una sessione non lascia una riga: avviene di continuo. Le righe del "
         "servizio arrivano nello stesso registro precedute da " + c("[servizio]") + ": quando serve portare il "
         "display a 60 Hz (" + rif("Frequenza del display allo spegnimento") + ") compare "
         + c("phonestra-servizio: video: frequenza del display a 60 Hz prima dello spegnimento") + ".")

CHAPTER = ("Il pannello del telefono", [
    ("Spegnere il pannello", S1),
    ("Frequenza del display allo spegnimento", S2),
    ("Il telefono in mano", S3),
    ("Chiamate", S4),
    ("Cadute e ricollegamenti", S5),
    ("Il registro dei cambi", S6),
])
