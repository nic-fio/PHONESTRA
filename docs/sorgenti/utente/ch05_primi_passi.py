from build import flow, key, note, p, rif, steps, table, tip, ui


PERCORSO = flow([
    ("Scaricare", "l'AppImage", "soft"),
    ("Collegare", "il telefono, una volta", "blue"),
    ("Aprire", "un'app dal drawer", "blue"),
    ("Usare", "mouse e tastiera", "blue"),
    ("Chiudere", "il telefono torna com'era", "navy"),
], "«FIG» — Dal file scaricato alla prima app sul PC")

S1 = p("Questo percorso porta da Phonestra appena scaricato alla prima app usata sul PC, in cinque tappe. Ogni tappa "
       "rimanda al capitolo che la spiega nel dettaglio.", lead=True) + PERCORSO + \
    table(["Tappa", "Che cosa fare", "Dove"], [
        ["Scaricare", "Scaricare l'AppImage, renderla eseguibile e avviarla.", rif("Installazione e primo avvio")],
        ["Collegare", "Seguire " + ui("Aggiungi un telefono") + ": quattro impostazioni sul telefono e un codice di "
         "6 cifre.", rif("Collegare il telefono")],
        ["Aprire", "Un clic su un'app nel drawer.", rif("La prima app")],
        ["Usare", "Il mouse fa da dito, la tastiera del PC scrive nell'app.", rif("Mouse, tastiera e appunti")],
        ["Chiudere", "Chiudere tutte le finestre di Phonestra.", rif("Chiudere Phonestra")],
    ], "«TAB» — Le tappe del percorso")

S2 = steps([
    "Con il telefono sbloccato e il drawer aperto, aspettare che la pillola del telefono dica "
    + ui("collegato via Wi-Fi") + " e che compaiano le app.",
    "Fare clic su un'app, per esempio l'app dei messaggi. Oppure scriverne il nome e premere " + key("Invio") + ".",
    "L'app si apre in una sua finestra. Lo schermo del telefono è spento (Phonestra lo spegne appena collegato): il "
    "telefono resta acceso e sbloccato e Phonestra lo usa senza schermo (" + rif("Lo schermo del telefono") + ").",
    "Usare l'app col mouse: il clic è un tocco, la rotellina scorre, la tastiera del PC scrive.",
    "Per tornare alla schermata precedente dell'app premere il pulsante " + ui("Indietro") + " in alto a sinistra, "
    "oppure " + key("Esc") + ".",
    "Per chiudere l'app chiudere la sua finestra: l'app si chiude anche sul telefono.",
]) + tip("sotto l'icona di un'app aperta in una finestra compare un pallino blu. Un secondo clic sull'icona non apre "
         "un'altra finestra: porta davanti quella già aperta.", "Il pallino blu.")

S3 = table(["Sul telefono", "Con Phonestra"], [
    ["Tasto Indietro", "Il pulsante " + ui("Indietro") + " della finestra, " + key("Esc") + " o il tasto «indietro» "
     "del mouse."],
    ["Schermata Home", "Il drawer: è la «Home» di Phonestra. Le finestre delle app sono già nella barra del desktop."],
    ["App recenti", "La barra del desktop con le finestre aperte; oppure lo schermo del telefono nel drawer."],
    ["Tendina e impostazioni rapide", "Lo schermo del telefono nel drawer: trascinare dall'alto col mouse."],
    ["Tasti del volume", "Il volume del PC: l'audio del telefono esce dalle casse del PC (" + rif("L'audio dal PC")
     + ")."],
], "«TAB» — Dove si trovano i comandi del telefono") + \
    note("le finestre delle app non hanno i pulsanti Home e App recenti di proposito: in Phonestra ogni app è una "
         "finestra del PC, e le finestre si gestiscono col desktop.", "Perché mancano Home e recenti.")

CHAPTER = ("Primi passi", [
    ("Il percorso", S1),
    ("La prima app", S2),
    ("Home, Indietro e tendina", S3),
])
