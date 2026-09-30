from build import fig, key, note, p, rif, steps, table, text, tip, ui, ul, warn


def rett(x, y, w, h, fill, stroke="", r=8, extra=""):
    s = f' stroke="{stroke}"' if stroke else ""
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}"{s}{extra}/>'


def filo(x1, y1, x2, y2):
    return (f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="#94a3b8" stroke-width="1.3"/>'
            f'<circle cx="{x2}" cy="{y2}" r="2.5" fill="#94a3b8"/>')


FINESTRA = fig(
    rett(330, 14, 240, 286, "#ffffff", "#cbd5e1", 12)
    + rett(330, 14, 240, 48, "#eef2f7", "#cbd5e1", 12)
    + rett(340, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + text(353, 43, "‹", 18, "#334155", "700", "middle", False)
    + text(426, 35, "Nome dell'app", 12, "#0f172a", "700", "middle", False)
    + text(426, 51, "Telefono", 10, "#64748b", "400", "middle", False)
    + rett(476, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + rett(482, 33, 14, 10, "#475569", r=2)
    + rett(506, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + '<circle cx="519" cy="38" r="5.5" fill="#e01b24"/>'
    + rett(536, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + text(549, 43, "⋮", 15, "#334155", "700", "middle", False)
    + rett(340, 72, 220, 218, "#f1f5f9", "#cbd5e1", 8, ' stroke-dasharray="4 3"')
    + text(450, 172, "l'app del telefono", 13, "#475569", "700", "middle", False)
    + text(450, 192, "clic = tocco, rotellina = scorrere", 11, "#64748b", "400", "middle", False)
    # etichette a sinistra
    + filo(300, 38, 340, 38) + text(292, 42, "Indietro (Esc)", 12, "#003a90", "700", "end")
    + filo(300, 96, 400, 52) + text(292, 92, "Nome dell'app e", 12, "#003a90", "700", "end")
    + text(292, 108, "stato del collegamento", 12, "#003a90", "700", "end")
    + filo(300, 180, 340, 180) + text(292, 176, "L'app: si ridimensiona", 12, "#003a90", "700", "end")
    + text(292, 192, "insieme alla finestra", 12, "#003a90", "700", "end")
    # etichette a destra
    + filo(600, 96, 549, 51) + text(608, 92, "Altri comandi: Ruota,", 12, "#003a90", "700", "start")
    + text(608, 108, "Copia screenshot, Chiudi app", 12, "#003a90", "700", "start")
    + filo(600, 146, 519, 51) + text(608, 150, "Registra", 12, "#003a90", "700", "start")
    + filo(600, 190, 489, 51) + text(608, 194, "Screenshot", 12, "#003a90", "700", "start"),
    900, 310, "«FIG» — La finestra di un'app")

S1 = p("Ogni app del telefono si apre nella sua finestra. Si può aprire in molti modi, e si possono tenere aperte più "
       "app insieme.", lead=True) + ul([
    "Un clic sull'icona dell'app nella pagina " + ui("App") + " del drawer.",
    "Scrivendo il nome nella ricerca " + ui("Cerca un'app") + " (basta cominciare a scrivere col drawer davanti) e "
    "premendo " + key("Invio") + ": si apre la prima app trovata.",
    "Dalla scheda " + ui("PREFERITI") + ", in cima alla pagina, con le app scelte come preferite.",
    "Con un clic su una notifica, nella pagina " + ui("Notifiche") + " o nell'avviso del sistema ("
    + rif("Notifiche") + ").",
]) + p("Se l'app è già aperta, la sua finestra torna davanti: ogni app ha una sola finestra. Nell'elenco compaiono le "
       "app che hanno un'icona nella schermata delle app del telefono, comprese " + ui("Impostazioni") + " e "
       + ui("Fotocamera") + ".") + \
    note("l'elenco delle app si legge al collegamento e dopo ogni installazione fatta da Phonestra. Se manca un'app "
         "appena installata sul telefono, " + ui("Preferenze") + " › " + ui("Aggiorna ora") + " lo rilegge.",
         "App appena installate.")

S2 = p("Il clic destro su un'app nel drawer apre il suo menu:", lead=True) + \
    table(["Voce", "Che cosa fa"], [
        [ui("Apri"), "Apre l'app nella sua finestra. Se è già aperta la voce diventa " + ui("Porta in primo piano")
         + " e sotto il nome compare " + ui("aperta in una finestra") + "."],
        [ui("Aggiungi ai preferiti") + " / " + ui("Togli dai preferiti"), "Mette o toglie l'app dalla scheda "
         + ui("PREFERITI") + ". I preferiti si ricordano per ogni telefono; durante una ricerca sono nascosti."],
        [ui("Informazioni sull'app"), "Apre in una finestra, " + ui("Informazioni · <nome dell'app>") + ", la pagina "
         "di Android con le informazioni dell'app: permessi, memoria, notifiche, arresto forzato."],
        [ui("Chiudi app"), "Chiude la finestra dell'app (solo se è aperta)."],
        [ui("Disinstalla…"), "Disinstalla l'app, con conferma (" + rif("Disinstallare un'app") + "). Spenta per le app "
         "di sistema: " + ui("App di sistema: non si può disinstallare") + "."],
    ], "«TAB» — Il menu di un'app")

S3 = p("La finestra di un'app ha in alto una barra con pochi pulsanti; tutto il resto è l'app del telefono.",
       lead=True) + FINESTRA + \
    table(["Pulsante", "Scorciatoia", "Che cosa fa"], [
        [ui("Indietro") + " (freccia a sinistra)", key("Esc"), "Come il tasto Indietro di Android."],
        ["Titolo", "", "Il nome dell'app e, sotto, il nome del telefono o lo stato del collegamento ("
         + ui("collegamento…") + ", " + ui("scollegato") + ", " + ui("Telefono bloccato: sbloccalo per continuare")
         + ")."],
        [ui("Screenshot (salvato e copiato)"), "", "Salva un'immagine dell'app e la copia negli appunti ("
         + rif("Screenshot e registrazione") + ")."],
        [ui("Registra lo schermo"), "", "Avvia e ferma la registrazione di un video dell'app."],
        [ui("Altri comandi") + " (⋮) › " + ui("Ruota"), key("Ctrl", "R"), "Gira la finestra: scambia larghezza e "
         "altezza."],
        [ui("Altri comandi") + " (⋮) › " + ui("Copia screenshot"), key("Ctrl", "Maiusc", "C"), "Copia l'immagine "
         "dell'app negli appunti, senza salvarla."],
        [ui("Altri comandi") + " (⋮) › " + ui("Chiudi app"), key("Ctrl", "W"), "Chiude la finestra e l'app."],
    ], "«TAB» — I comandi della finestra di un'app") + \
    p("Le finestre delle app sono finestre normali del desktop: si spostano, si affiancano, si mettono a tutto "
      "schermo e compaiono nella barra del desktop col nome dell'app.")

S4 = p("L'app si adatta alla finestra: allargando la finestra, l'app ha più spazio, come su un tablet; stringendola, "
       "torna simile al telefono. Lo stesso vale ingrandendo la finestra a tutto schermo.", lead=True) + ul([
    ui("Ruota") + " (" + key("Ctrl", "R") + ") scambia larghezza e altezza della finestra: una finestra alta e stretta "
    "diventa larga e bassa. Non funziona con la finestra ingrandita né durante una registrazione.",
    "Per ingrandire il contenuto dell'app (una mappa, una foto) si usa lo zoom: " + key("Ctrl") + " + rotellina, "
    + key("Ctrl", "+") + " e " + key("Ctrl", "−") + ", o il pizzico sul touchpad (" + rif("Mouse e touchpad") + ").",
]) + note("alcune app accettano solo la forma verticale del telefono (per esempio alcuni social). Per queste la finestra "
          "ha una misura fissa: non si ingrandisce e non si ridimensiona trascinando i bordi. Se era larga, torna "
          "verticale da sola.", "App solo verticali.")

S5 = steps([
    "Chiudere la finestra con la " + ui("×") + " in alto a destra, oppure " + ui("Altri comandi") + " › "
    + ui("Chiudi app") + ", oppure " + key("Ctrl", "W") + ".",
    "L'app si chiude anche sul telefono: viene tolta dalle app recenti.",
    "Se l'app stava suonando musica o un video, viene messa in pausa, così il telefono non continua a suonare da "
    "solo.",
]) + note("l'app non viene arrestata a forza: continua a ricevere le sue notifiche, come quando la si chiude dalle "
          "recenti sul telefono.", "Chiusa, non arrestata.")

VELO = table(["Titolo", "Testo", "Pulsanti"], [
    [ui("Riconnessione…"), ui("Riprovo da solo. Se il telefono è bloccato, sbloccalo: l'app torna qui dov'era."),
     ui("Riconnetti ora") + ", " + ui("Chiudi")],
    [ui("Phonestra non parte sul telefono"), "Il telefono non riesce ad avviare la parte di Phonestra che mostra le "
     "app: " + ui("Riconnetti ora") + " per riprovare, e se non basta riavviare il telefono.",
     ui("Riconnetti ora") + ", " + ui("Chiudi")],
], "«TAB» — Il velo sulla finestra di un'app")

S6 = p("Se il collegamento cade (il Wi-Fi si interrompe, il telefono si blocca), le finestre non si chiudono. L'ultima "
       "immagine dell'app resta, sfocata, con un velo che spiega che cosa succede.", lead=True) + VELO + ul([
    "Phonestra riprova da solo, a intervalli di pochi secondi. " + ui("Riconnetti ora") + " riprova subito.",
    "Quando il collegamento torna, l'app ricompare nella sua finestra, dov'era.",
    ui("Chiudi") + " chiude la finestra.",
    "Se il telefono è solo bloccato, il velo non compare: cambia il sottotitolo in "
    + ui("Telefono bloccato: sbloccalo per continuare") + ". Dopo lo sblocco Phonestra si ricollega da solo.",
])

S7 = p("Alcune app proteggono certe schermate: le app delle banche, le password, i video a pagamento. Android non le "
       "lascia mostrare fuori dal telefono.", lead=True) + \
    p("Al posto dell'immagine nera Phonestra mostra " + ui("Schermata protetta") + ": " + ui("Questa app non permette "
      "di mostrare questa schermata fuori dal telefono. Le altre schermate dell'app funzionano normalmente.")) + \
    tip("per le schermate protette si usa il telefono in mano: sbloccandolo, lo schermo si riaccende ("
        + rif("Il telefono in mano") + ").", "Col telefono in mano.") + \
    warn("Phonestra non aggira le schermate protette: è una scelta di sicurezza delle app e di Android.", "Nessun trucco.")

CHAPTER = ("Le finestre delle app", [
    ("Aprire un'app", S1),
    ("Il menu di un'app", S2),
    ("Com'è fatta una finestra", S3),
    ("Dimensioni, rotazione e app verticali", S4),
    ("Chiudere un'app", S5),
    ("Quando il collegamento cade", S6),
    ("Schermate protette", S7),
])
