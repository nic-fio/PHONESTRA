from build import key, note, p, rif, table, tip, ui, ul, warn


S1 = p("Nelle finestre delle app, e sullo schermo del telefono nel drawer, il mouse fa da dito.", lead=True) + \
    table(["Sul PC", "Sul telefono"], [
        ["Clic", "Tocco."],
        ["Trascinamento col tasto sinistro", "Dito che scorre: sposta le pagine, trascina gli oggetti."],
        ["Tasto sinistro tenuto premuto", "Pressione lunga del dito."],
        ["Clic destro", "Pressione lunga: seleziona la parola sotto il puntatore e apre il menu di Android (Copia, "
         "Seleziona tutto…)."],
        ["Rotellina, scorrimento a due dita sul touchpad", "Scorre la pagina nel punto sotto il puntatore."],
        [key("Ctrl") + " + rotellina", "Zoom nel punto sotto il puntatore, come un pizzico a due dita."],
        ["Pizzico sul touchpad", "Zoom, come il pizzico a due dita sullo schermo del telefono."],
        ["Tasto «indietro» del mouse (se c'è)", "Indietro."],
    ], "«TAB» — Mouse e touchpad") + \
    note("alcune app hanno uno zoom proprio con la rotellina semplice, senza " + key("Ctrl") + ": per esempio le "
         "mappe.", "Mappe.")

S2 = p("La tastiera del PC scrive nell'app che ha il fuoco. Alcuni tasti fanno cose in più:", lead=True) + \
    table(["Tasto", "Che cosa fa"], [
        "Nelle app",
        [key("Esc"), "Indietro (si può spegnere: " + ui("Preferenze") + " › " + ui("Esc torna indietro") + ")."],
        [key("Invio") + ", " + key("Backspace") + ", " + key("Canc") + ", " + key("Tab"), "Come sulla tastiera di un "
         "telefono o di un tablet."],
        [key("←") + " " + key("→") + ", " + key("Home") + ", " + key("Fine"), "Spostano il cursore nel testo."],
        [key("↑") + " / " + key("↓"), "Scorrono la pagina di un passo; mentre si scrive spostano il cursore."],
        [key("Pag↑") + " / " + key("Pag↓"), "Scorrono la pagina di una schermata."],
        [key("Ctrl") + " + lettera", "La scorciatoia dell'app: " + key("Ctrl", "A") + " seleziona tutto, "
         + key("Ctrl", "C") + " copia, " + key("Ctrl", "X") + " taglia, " + key("Ctrl", "Z") + " annulla (se l'app le "
         "conosce)."],
        [key("Ctrl", "V") + ", " + key("Maiusc", "Ins"), "Incolla nell'app il testo copiato sul PC ("
         + rif("Gli appunti") + ")."],
        [key("Ctrl", "+") + " / " + key("Ctrl", "−"), "Zoom al centro della finestra."],
        "Comandi della finestra",
        [key("Ctrl", "R"), ui("Ruota") + ": scambia larghezza e altezza della finestra."],
        [key("Ctrl", "Maiusc", "C"), ui("Copia screenshot") + ": copia l'immagine dell'app negli appunti."],
        [key("Ctrl", "W"), ui("Chiudi app") + ": chiude la finestra e l'app."],
        "Del desktop",
        [key("Alt") + " + tasto", "Restano al desktop (per esempio " + key("Alt", "F4") + " chiude la finestra, "
         + key("Alt", "Tab") + " cambia finestra)."],
    ], "«TAB» — Tastiera e scorciatoie") + \
    tip("nel drawer i tasti vanno alla ricerca delle app. Per scrivere sullo schermo del telefono nel drawer, fare "
        "prima un clic sullo schermo.", "Tasti nel drawer.")

S3 = p("La tastiera sullo schermo del telefono non compare: si scrive con quella del PC, con la sua disposizione dei "
       "tasti (italiana, se il PC è in italiano).", lead=True) + ul([
    "Lettere, numeri e segni comuni arrivano all'app come se fossero scritti sul telefono.",
    "Le lettere accentate (à, è, ì, ò, ù) e i simboli meno comuni passano dagli appunti del telefono: Phonestra li "
    "mette negli appunti del telefono e li incolla nell'app.",
    "Le scorciatoie con " + key("Ctrl") + " arrivano all'app come su una tastiera collegata al telefono.",
]) + warn("scrivendo una lettera accentata, gli appunti del telefono vengono sostituiti da quella lettera. Se sul "
          "telefono era stato copiato qualcosa da incollare, conviene incollarlo prima.", "Appunti del telefono.")

S4 = p("Gli appunti passano nei due sensi, in automatico, solo per il testo semplice.", lead=True) + \
    table(["Verso", "Come funziona", "Che cosa non passa"], [
        ["Dal telefono al PC", "Il testo copiato sul telefono (anche in una finestra di Phonestra) è subito negli "
         "appunti del PC: si incolla in qualsiasi programma.", "Testi segnati come riservati (per esempio le "
         "password copiate da un gestore di password), testi molto lunghi, immagini e file."],
        ["Dal PC al telefono", "Con " + key("Ctrl", "V") + " (o " + key("Maiusc", "Ins") + ") in una finestra di "
         "Phonestra, il testo copiato sul PC va nell'app.", "Le password copiate da un gestore di password del PC ("
         + ui("Password non inviata al telefono") + "); testi troppo lunghi (" + ui("Testo troppo lungo: usa il "
         "trasferimento file") + "); immagini e file."],
    ], "«TAB» — Gli appunti nei due sensi") + ul([
        "Se negli appunti del PC non c'è testo, " + key("Ctrl", "V") + " mostra " + ui("Negli appunti del PC non c'è "
        "testo") + ".",
        "Il testo copiato sul PC va al telefono solo al momento di incollarlo, e solo al telefono collegato.",
        "Per le immagini e i file si usano " + ui("Invia file…") + " e " + ui("Ricevi file…") + " ("
        + rif("App e file") + "). Uno screenshot di Phonestra finisce comunque negli appunti del PC come immagine.",
    ]) + note("su alcuni desktop (per esempio GNOME) un programma può cambiare gli appunti solo mentre una sua "
              "finestra è attiva. Se il testo copiato sul telefono non si incolla sul PC, fare un clic su una finestra "
              "di Phonestra e riprovare.", "GNOME.")

CHAPTER = ("Mouse, tastiera e appunti", [
    ("Mouse e touchpad", S1),
    ("Tastiera e scorciatoie", S2),
    ("Scrivere con la tastiera del PC", S3),
    ("Gli appunti", S4),
])
