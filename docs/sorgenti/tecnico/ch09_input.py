from build import c, flow, key, note, p, rif, table

S1 = p("Mouse, touchpad e tastiera del PC diventano dita e tasti nello schermo dell'app; gli appunti passano in "
       "tutti e due i sensi, solo testo, mai le password.", lead=True) + \
    p("Fascia " + c("0x50–0x5f") + ", big-endian. Gli eventi hanno " + c("id") + " 0 e nessuna risposta: il PC non "
      "aspetta il telefono. " + c("larghezza") + "/" + c("altezza") + " sono la misura dell'immagine su cui il PC ha "
      "calcolato le coordinate.") + \
    table(["Tipo", "Nome", "Contenuto"], [
        [c("0x50"), c("TOCCHI"), c("display i32 · larghezza u16 · altezza u16 · n u8 · n × (dito i64 · azione u8 · x i32 · y i32 · pressione f32)")],
        [c("0x51"), c("ROTELLINA"), c("display i32 · x i32 · y i32 · larghezza u16 · altezza u16 · orizzontale f32 · verticale f32")],
        [c("0x52"), c("TASTO"), c("display i32 · azione u8 · codice u32 · ripetizione u32 · meta u32")],
        [c("0x53"), c("TESTO"), c("display i32 · testo UTF-8")],
        [c("0x54"), c("INDIETRO"), c("display i32 · azione u8")],
        [c("0x55"), c("APPUNTI_SCRIVI"), c("display i32 · incolla u8 · testo UTF-8") + "; risponde se " + c("id")
         + " non è 0"],
        [c("0x56"), c("APPUNTI_LEGGI"), "domanda vuota; risposta " + c("stato u8 · testo")],
        [c("0x57"), c("APPUNTI_ASCOLTA"), c("attivo u8") + "; risponde se " + c("id") + " non è 0"],
        [c("0x58"), c("APPUNTI_CAMBIATI"), "servizio → PC, spontaneo: " + c("stato u8 · testo")],
        [c("0x5c"), c("CONTEGGI"), "diagnosi: iniettati, falliti, scartati, avvisi degli appunti, "
         + c("ascolto_appunti") + ", ultimo errore"],
        [c("0x5d"), c("PROVA"), "comandi delle prove (" + c("InputProva.java") + "), su un thread loro ("
         + c("input-prova") + "): " + c("apri <l> <a> <dpi>") + ", " + c("avvia <display> <pacchetto>") + ", "
         + c("azione <display> <azione> [pacchetto]") + ", " + c("firma <display>") + " (32×56 luminosità), "
         + c("chiudi") + ", " + c("salva-appunti") + ", " + c("ripristina-appunti") + ", " + c("esterno <0|1> <testo>")],
    ], "«TAB» — I messaggi di input e appunti") + \
    p("Un test Rust in " + c("input_nostro.rs") + " legge " + c("Input.java") + " e " + c("Video.java")
      + " e controlla che nessun numero di messaggio sia usato due volte.")

S2 = p("Sul telefono l'input diventa eventi di Android iniettati nello schermo giusto, in ordine, da un solo "
       "thread.", lead=True) + flow([
    ("finestra.rs", "eventi di GTK", "navy"), ("InputNostro", "codifica, Mittente", "blue"),
    ("Canale comandi", "senza risposta", "dark"), ("Servizio", "Input.ricevi", "blue"),
    ("Thread «input»", "injectInputEvent", "light")],
    "«FIG» — Il percorso di un tocco, dalla finestra del PC allo schermo virtuale") + table(["Passo", "Come"], [
    ["Coda", "Il thread che legge i comandi non inietta; mette i messaggi in coda al thread «input», uno solo, così "
     "l'ordine resta."],
    ["Evento", c("InputEvent.setDisplayId") + " sempre, poi " + c("injectInputEvent") + " in modo asincrono. Un "
     "rifiuto si conta (" + c("falliti") + "); nel registro il primo errore e poi uno ogni 100."],
    ["Dita", "Ogni identificativo del PC (−1 mouse, −2 dito generico, 10 e 11 per il pizzico) diventa un dito con "
     "numero locale 0–9, al massimo 10. Clic e trascinamenti sono dita (" + c("SOURCE_TOUCHSCREEN") + "): trascinare "
     "scorre, la pressione lunga apre i menu. Ogni evento porta tutte le dita appoggiate."],
    ["Scalatura", "Coordinate × (misura dello schermo / misura del PC). Un evento calcolato su una misura diversa da "
     "quella dichiarata dal video si scarta: durante un ridimensionamento un clic finirebbe nel posto sbagliato. Per "
     "questo il PC arrotonda le misure a multipli di 8 come il telefono."],
    ["Rotellina", c("ACTION_SCROLL") + ", " + c("SOURCE_MOUSE") + ", valori frazionari, entro ±16."],
    ["Tasti", c("KeyEvent") + " da tastiera virtuale. Testo: la mappa dei tasti virtuale un carattere alla volta, in "
     "pratica solo ASCII; il resto (lettere accentate, simboli) il PC lo manda con «incolla»."],
    ["Indietro", c("KEYCODE_BACK") + "; sullo schermo principale spento, " + c("POWER") + " per riaccenderlo."],
    ["Incolla", "Appunti del telefono + " + c("KEYCODE_PASTE") + "."],
], "«TAB» — Come il servizio inietta l'input")

S3 = p(c("finestra.rs") + " traduce gli eventi di GTK (" + c("finestra::tastiera") + " per i tasti). I modificatori "
       "seguono i valori di " + c("KeyEvent.META_*") + " (" + c("META_SHIFT") + ", " + c("META_CTRL") + ").",
       lead=True) + \
    table(["Sul PC", "Sul telefono"], [
        ["Clic, trascinamento", "Dito"],
        ["Clic destro", "Pressione lunga (seleziona e apre il menu di Android)"],
        ["Rotellina, due dita sul touchpad", "Scorrimento nel punto del puntatore"],
        [key("Ctrl") + " + rotellina, " + key("Ctrl", "+") + " / " + key("Ctrl", "−") + ", pizzico sul touchpad",
         "Pizzico a due dita (zoom; coi tasti al centro della finestra)"],
        [key("Esc") + ", tasto «indietro» del mouse, pulsante Indietro", "Indietro (" + key("Esc")
         + " si può spegnere nelle preferenze)"],
        [key("↑") + " / " + key("↓"), "Uno scatto di scorrimento; tasti veri mentre si scrive (dopo una lettera, "
         + key("Backspace") + " o " + key("Canc") + ")"],
        [key("Pag↑") + " / " + key("Pag↓"), "Scorrimento dell'80% dell'altezza della finestra"],
        [key("Invio") + ", " + key("Backspace") + ", " + key("Canc") + ", " + key("Tab") + ", " + key("←") + " "
         + key("→") + ", " + key("Home") + ", " + key("Fine"), "I tasti Android corrispondenti (anche con "
         + key("Alt") + ": " + key("Alt", "←") + " è una freccia, non Indietro)"],
        ["Lettere ASCII", c("TESTO") + "; il resto con incolla"],
        [key("Ctrl") + " + lettera", "La scorciatoia Android (" + key("Ctrl", "C") + ", " + key("Ctrl", "A")
         + "…), premuta e rilasciata"],
        [key("Ctrl", "V") + ", " + key("Maiusc", "Ins"), "Gli appunti del PC vanno nell'app"],
        [key("Ctrl", "R") + ", " + key("Ctrl", "W") + ", " + key("Ctrl", "Maiusc", "C"), "Restano al PC: Ruota, "
         "Chiudi app, Copia screenshot (" + rif("Le finestre delle app") + ")"],
        [key("Alt") + " + altro", "Resta al desktop (" + key("Alt", "F4") + "…)"],
    ], "«TAB» — Tastiera e mouse")

S4 = p("Il servizio parla direttamente con " + c("IClipboard") + ", come pacchetto " + c("com.android.shell")
       + ": niente " + c("ClipboardManager") + ", quindi niente Looper da far girare e niente passaggio dal servizio "
       "Samsung " + c("semclipboard") + ", che col contesto sbagliato rifiuta la scrittura. Le firme cambiano tra le "
       "versioni: si sceglie la variante più lunga i cui parametri, dopo quelli fissi, sono solo testi e interi.",
       lead=True) + \
    table(["Verso", "Come", "Cosa non passa"], [
        ["Telefono → PC", c("APPUNTI_ASCOLTA 1") + " all'avvio; a ogni copia " + c("APPUNTI_CAMBIATI") + " ("
         + c("appunti::ascolta") + ") → " + c("Collegamento::appunti") + " → appunti di GTK", "Copie segnate come "
         "sensibili (stato 2) o di sensibilità sconosciuta (3); testi oltre 200 000 byte; i rimbalzi"],
        ["PC → telefono", "Solo con " + key("Ctrl", "V") + " in una finestra: " + c("APPUNTI_SCRIVI") + " con "
         "incolla=1", "Testi dei gestori di password (" + c("x-kde-passwordManagerHint") + "): avviso «Password non "
         "inviata al telefono»; testi troppo lunghi: «Testo troppo lungo: usa il trasferimento file»"],
    ], "«TAB» — Gli appunti nei due sensi") + \
    p("<b>Rimbalzi.</b> Quello che Phonestra mette negli appunti del telefono non deve tornare al PC. Il servizio "
      "ignora le proprie scritture (anche un testo uguale entro 3 s, perché l'avviso può arrivare dopo) e Samsung manda "
      "ogni avviso due volte (scartato entro 0,5 s); il PC ricorda i testi inviati (" + c("Collegamento::e_un_rimbalzo")
      + "). Il servizio rilegge gli appunti solo se il clip attuale è suo: leggere quello di un'altra app farebbe "
      "comparire l'avviso «ha incollato dagli appunti».") + \
    note("su GNOME gli appunti del PC si cambiano solo mentre una finestra del programma è attiva. " + c("main")
         + " mette subito la copia e, per sicurezza, la rimette alla prossima attivazione di una finestra di "
         "Phonestra; se nel frattempo sul PC si copia altro, quella in attesa si dimentica.", "Wayland.")

CHAPTER = ("Input e appunti", [
    ("Messaggi di input e appunti", S1),
    ("Iniezione", S2),
    ("Tastiera e mouse sul PC", S3),
    ("Appunti", S4),
])
