from build import c, flow, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("Il primo collegamento si fa una volta sola per ogni telefono, senza cavo, in qualche minuto. Phonestra "
       "chiede di accendere sul telefono quattro cose; come si fa sul proprio modello lo si scopre con la ricerca "
       "delle Impostazioni del telefono.", lead=True) + ul([
    "Il telefono: Android 14 o successivo, carico, sbloccato, a portata di mano.",
    "Il PC e il telefono sulla stessa rete Wi-Fi.",
    "Phonestra avviato: al primo avvio si apre da solo " + ui("Aggiungi un telefono") + "; per un telefono in più, "
    "si apre dal drawer con " + ui("Aggiungi telefono") + " (" + rif("Aggiungere un altro telefono") + ").",
])

VOCI = table(["N.", "Voce nella finestra", "Che cosa fare sul telefono", "Parola da cercare nelle Impostazioni"], [
    ["1", ui("Telefono e PC sulla stessa rete Wi-Fi"), "Controllare che il telefono sia collegato alla stessa rete "
     "Wi-Fi del PC (non quella «ospiti», non i dati mobili).", "—"],
    ["2", ui("Sblocca le Opzioni sviluppatore"), "Aprire le informazioni sul telefono e toccare 7 volte "
     + ui("Numero build") + " (su alcune marche ha un altro nome); il telefono chiede il PIN e sblocca le "
     + ui("Opzioni sviluppatore") + ".", c("numero build")],
    ["3", ui("Spegni le protezioni che bloccano il collegamento"), "Solo se ci sono: " + ui("Blocco automatico")
     + " sui Samsung, " + ui("Protezione avanzata") + " sui Google Pixel. Se non si trovano, il telefono non le ha.",
     c("blocco automatico")],
    ["4", ui("Debug wireless: accendilo e associa questo PC"), "Accendere l'interruttore " + ui("Debug wireless")
     + " e toccare " + ui("Consenti") + " per la rete; poi toccare la scritta " + ui("Debug wireless") + " (non "
     "l'interruttore), poi " + ui("Associa dispositivo con codice di associazione") + ".", c("debug wireless")],
], "«TAB» — Le quattro voci di «Aggiungi un telefono»")

S2 = p("La finestra si chiama " + ui("Aggiungi un telefono") + ". A sinistra, sotto il titolo " + ui("Prepara il telefono")
       + ", c'è l'elenco delle quattro cose da fare; a destra i riquadri " + ui("COSA VEDO IN RETE") + " e "
       + ui("TI SEI BLOCCATO?") + " e il pulsante per la strada col cavo.", lead=True) + VOCI + \
    p("Ogni voce si apre con un clic e contiene:") + ul([
        "perché serve, in parole semplici;",
        "il riquadro " + ui("DOVE SI TROVA") + ", con la parola da scrivere nella ricerca delle Impostazioni del telefono;",
        ui("Fatto ›") + ", da premere quando la voce è fatta (le prime tre);",
        ui("Chiedi a Google ↗") + ", che apre nel browser la ricerca di Google con la domanda già scritta: spiega come "
        "si fa sul proprio modello, spesso con un video.",
    ]) + \
    table(["Scritta a destra della voce", "Significato"], [
        [ui("da fare sul telefono"), "Voce ancora da fare."],
        [ui("✓ fatto"), "Segnata come fatta con " + ui("Fatto ›") + "."],
        [ui("✓ visto da Phonestra"), "Phonestra vede il Debug wireless acceso: le voci prima sono per forza fatte."],
        [ui("me ne accorgo da solo"), "Phonestra aspetta di vedere in rete il Debug wireless acceso."],
        [ui("✓ acceso: ora il codice"), "Debug wireless acceso: manca il codice di associazione."],
        [ui("scrivi il codice"), "Il telefono mostra il codice: va scritto nel campo della voce 4."],
        [ui("associo…"), "Phonestra si sta associando al telefono."],
        [ui("✓ collegato"), "Fatto: il telefono è collegato."],
    ], "«TAB» — Lo stato di ogni voce") + \
    note("le voci che Phonestra vede in rete si spuntano da sole. Il riquadro " + ui("COSA VEDO IN RETE") + " dice "
         "cosa vede in quel momento: " + ui("○ Nessun telefono col Debug wireless acceso") + ", "
         + ui("● Un telefono col Debug wireless acceso") + ", " + ui("● Schermata del codice aperta") + ".",
         "Spunte automatiche.") + \
    tip("se ci si blocca, il riquadro " + ui("TI SEI BLOCCATO?") + " suggerisce di fare una foto della finestra e "
        "mandarla a chi ha consigliato Phonestra: la finestra dice a che punto si è.", "Farsi aiutare.")

FLUSSO = flow([
    ("Stessa rete", "Wi-Fi di casa", "soft"),
    ("Opzioni", "sviluppatore: 7 tocchi", "blue"),
    ("Protezioni", "solo se ci sono", "blue"),
    ("Debug wireless", "acceso", "blue"),
    ("Codice", "6 cifre", "navy"),
    ("Collegato", "una volta sola", "green"),
], "«FIG» — Il primo collegamento, dalla rete al telefono collegato", width=960)

S3 = FLUSSO + steps([
    "Controllare che il telefono sia sulla stessa rete Wi-Fi del PC; premere " + ui("Fatto ›") + ".",
    "Sbloccare le " + ui("Opzioni sviluppatore") + " (voce 2); premere " + ui("Fatto ›") + ".",
    "Se il telefono ha " + ui("Blocco automatico") + " o " + ui("Protezione avanzata") + " accesi, spegnerli "
    "(voce 3); premere " + ui("Fatto ›") + ".",
    "Nelle " + ui("Opzioni sviluppatore") + " accendere " + ui("Debug wireless") + " e toccare " + ui("Consenti")
    + " per la rete. Phonestra se ne accorge da solo e apre la voce 4.",
    "Toccare la scritta " + ui("Debug wireless") + ", poi " + ui("Associa dispositivo con codice di associazione")
    + ": il telefono mostra un codice di 6 cifre.",
    "Scrivere le 6 cifre nel campo della voce 4. Non serve premere niente: con la sesta cifra Phonestra si associa, "
    "si collega e salva il telefono.",
    "Quando compare " + ui("Fatto! «<nome del telefono>» è collegato via Wi-Fi.") + ", premere "
    + ui("Inizia a usare il telefono") + ": si apre il drawer.",
]) + note("indirizzo e porta del telefono non vanno scritti da nessuna parte: Phonestra li trova da solo in rete.",
          "Niente indirizzi.")

S4 = table(["Messaggio sotto il campo", "Che cosa fare"], [
    [ui("Il campo si attiva quando apri la schermata del codice sul telefono."), "Sul telefono aprire "
     + ui("Associa dispositivo con codice di associazione") + ". Se il campo resta grigio, controllare che telefono "
     "e PC siano sulla stessa rete."],
    [ui("Vedo la schermata del codice: scrivi le 6 cifre, al resto penso io."), "Scrivere le 6 cifre mostrate dal "
     "telefono."],
    [ui("Codice non accettato (…). Riapri «Associa dispositivo con codice di associazione» sul telefono e scrivi il "
        "codice nuovo."), "Il codice era sbagliato o scaduto: sul telefono chiudere e riaprire la schermata del "
     "codice, poi scrivere quello nuovo."],
], "«TAB» — Il campo del codice") + \
    p("Dopo l'associazione Phonestra toglie la scadenza all'autorizzazione di questo PC, così il collegamento non "
      "va rifatto dopo qualche giorno di non uso (" + rif("Cosa resta acceso sul telefono") + ").")

S5 = p("Il pulsante " + ui("Android 10 o precedente? Collega col cavo") + ", in basso a destra, apre la strada di "
       "riserva: il primo collegamento con il cavo USB, pensato per i telefoni che non hanno il Debug wireless. "
       "Dopo, il cavo si stacca e Phonestra funziona via Wi-Fi come sempre.", lead=True) + \
    warn("le app nelle finestre richiedono comunque Android 14 o successivo: con un telefono più vecchio Phonestra "
         "non può mostrarle, qualunque strada si usi per il primo collegamento.", "Android 14.") + \
    table(["Passo", "Che cosa chiede la finestra"], [
        [ui("Collega il cavo USB"), "Collegare il telefono al PC con un cavo che trasmette dati (alcuni cavi servono "
         "solo a ricaricare). Se non succede niente: dalla tendina del telefono, notifica " + ui("USB") + ", scegliere "
         + ui("Trasferimento file") + "."],
        [ui("Attiva il Debug USB"), "Sbloccare le " + ui("Opzioni sviluppatore") + " e accendere " + ui("Debug USB")
         + ". La finestra mostra i passaggi per la marca del telefono, da sfogliare con le frecce sotto il telefono "
         "disegnato."],
        [ui("Consenti il collegamento"), "Sul telefono compare una richiesta: spuntare " + ui("Consenti sempre da "
         "questo computer") + " e toccare " + ui("Consenti") + "."],
        [ui("Consenti la rete Wi-Fi"), "Phonestra accende il Debug wireless; la prima volta per ogni rete il telefono "
         "chiede il permesso: toccare " + ui("Consenti") + "."],
        [ui("Fatto"), "Il telefono è configurato e si collega via Wi-Fi: si può scollegare il cavo."],
    ], "«TAB» — I passi della strada col cavo") + ul([
        "La finestra va avanti da sola: si accorge di ogni passo fatto sul telefono.",
        "Senza la spunta " + ui("Consenti sempre da questo computer") + " il cavo funziona ma il Wi-Fi no: compare "
        + ui("Manca «Consenti sempre»") + " e la richiesta va ripetuta scollegando e ricollegando il cavo.",
        "Sui telefoni Xiaomi, Redmi e Poco c'è un passaggio in più, " + ui("Un passaggio in più per Xiaomi") + ": "
        "accendere " + ui("Debug USB (impostazioni di sicurezza)") + ", altrimenti le app si vedono ma non rispondono "
        "a mouse e tastiera.",
        "Se dopo 20 secondi la finestra è ancora al primo passo, compare il riquadro " + ui("Cosa vede il PC") + " con "
        "l'elenco dei dispositivi USB: è utile da fotografare per chi aiuta a distanza.",
    ])

S6 = p("Dopo il primo collegamento non c'è più niente da fare: all'avvio Phonestra cerca il telefono in rete e si "
       "collega da solo.", lead=True) + ul([
    "Il telefono deve essere acceso, sbloccato, sulla stessa rete Wi-Fi e con " + ui("Debug wireless") + " acceso.",
    "Android a volte spegne il Debug wireless da solo, per esempio cambiando rete Wi-Fi. Basta riaccenderlo nelle "
    + ui("Opzioni sviluppatore") + ": il codice non va rifatto, il PC resta associato.",
    "Su una rete nuova il telefono chiede di nuovo " + ui("Consenti") + " per quella rete.",
]) + tip("molti telefoni permettono di aggiungere " + ui("Debug wireless") + " ai riquadri rapidi della tendina: così "
         "si riaccende con un tocco.", "Riaccendere in fretta.")

CHAPTER = ("Collegare il telefono", [
    ("Prima di cominciare", S1),
    ("La finestra «Aggiungi un telefono»", S2),
    ("Passo per passo", S3),
    ("Il codice a 6 cifre", S4),
    ("La strada col cavo", S5),
    ("I collegamenti successivi", S6),
])
