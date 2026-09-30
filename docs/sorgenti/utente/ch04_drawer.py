from build import c, fig, key, note, p, pill, rif, table, text, tip, ui, ul


def rett(x, y, w, h, fill, stroke="", r=8, extra=""):
    s = f' stroke="{stroke}"' if stroke else ""
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}"{s}{extra}/>'


def numero(x, y, n):
    return (f'<circle cx="{x}" cy="{y}" r="10" fill="#003a90"/>'
            + text(x, y + 4, str(n), 11, "#ffffff", "800", "middle", False))


def voce(x, y, t, colore="#334155", peso="400", size=11):
    return text(x, y, t, size, colore, peso, "start", False)


ICONE = ["#3b82f6", "#16a34a", "#d97706", "#8b5cf6", "#ef4444", "#0ea5e9", "#64748b"]


def icone(x, y, n, primo=0):
    s = ""
    for i in range(n):
        s += rett(x + i * 56, y, 32, 32, ICONE[(i + primo) % len(ICONE)], r=9)
        s += rett(x + i * 56 - 4, y + 38, 40, 5, "#cbd5e1", r=2)
    return s


DRAWER = fig(
    rett(30, 12, 840, 304, "#ffffff", "#e5e7eb", 12)
    # barra del titolo
    + rett(30, 12, 840, 42, "#eef2f7", "#e5e7eb", 12)
    + rett(46, 25, 16, 16, "#0050C0", r=4) + voce(70, 38, "Phonestra", "#003a90", "700", 13)
    + rett(335, 21, 230, 24, "#ffffff", "#cbd5e1", 12)
    + '<circle cx="352" cy="33" r="4" fill="#16a34a"/>'
    + voce(362, 37, "Telefono  · collegato via Wi-Fi", "#334155", "600", 11)
    # barra laterale
    + rett(38, 62, 168, 246, "#f8fafc", "#e5e7eb", 10)
    + rett(46, 70, 152, 22, "#3584e4", r=6) + voce(58, 85, "App", "#ffffff", "700")
    + voce(58, 110, "Notifiche", "#334155", "600") + rett(170, 99, 20, 15, "#3584e4", r=7)
    + text(180, 110, "3", 9.5, "#ffffff", "700", "middle", False)
    + voce(52, 136, "TELEFONI", "#94a3b8", "700", 9.5)
    + '<circle cx="62" cy="152" r="3.5" fill="#16a34a"/>' + voce(72, 156, "Telefono", "#334155", "600")
    + voce(150, 156, "attivo", "#64748b", "400", 9.5)
    + voce(58, 178, "+ Aggiungi telefono", "#3584e4", "600")
    + voce(52, 202, "STRUMENTI", "#94a3b8", "700", 9.5)
    + voce(58, 222, "Installa app…") + voce(58, 240, "Invia file…") + voce(58, 258, "Ricevi file…")
    + voce(58, 284, "Preferenze", "#334155", "600") + voce(58, 302, "Informazioni")
    # pagina App
    + rett(222, 64, 440, 26, "#ffffff", "#cbd5e1", 13) + voce(240, 81, "Cerca un'app", "#94a3b8")
    + rett(222, 98, 440, 72, "#f1f5f9", "", 12) + voce(236, 114, "PREFERITI", "#94a3b8", "700", 9.5)
    + icone(246, 122, 3)
    + '<circle cx="262" cy="166" r="2.5" fill="#3584e4"/>'
    + rett(222, 178, 440, 130, "#f1f5f9", "", 12) + voce(236, 194, "TUTTE LE APP", "#94a3b8", "700", 9.5)
    + icone(246, 202, 7, 2) + icone(246, 254, 7, 4)
    # telefono
    + rett(690, 62, 156, 246, "#1e1e22", "", 22)
    + rett(697, 69, 142, 232, "#7fb8ff", "", 16)
    + voce(708, 86, "12:30", "#ffffff", "700", 10)
    + text(768, 180, "lo schermo vero", 12, "#ffffff", "700", "middle", False)
    + text(768, 196, "del telefono", 12, "#ffffff", "700", "middle", False)
    # numeri
    + numero(582, 33, 1) + numero(214, 70, 2) + numero(670, 104, 3) + numero(856, 76, 4),
    900, 326, "«FIG» — Il drawer: pillola del telefono (1), barra laterale (2), pagina (3), schermo del telefono (4)")

S1 = p("Il drawer è la finestra principale di Phonestra: si apre all'avvio e raccoglie le app del telefono, le "
       "notifiche, le preferenze e lo schermo del telefono. Il nome viene dall'inglese <i>app drawer</i>, il "
       "cassetto delle app di Android.", lead=True) + DRAWER + \
    table(["N.", "Parte", "A cosa serve", "Dove"], [
        ["1", "Pillola del telefono", "Nome e stato del collegamento; con un clic apre il menu del telefono.",
         rif("La pillola del telefono")],
        ["2", "Barra laterale", "Le pagine " + ui("App") + ", " + ui("Notifiche") + " e " + ui("Preferenze") + ", i "
         "telefoni e gli strumenti.", rif("La barra laterale")],
        ["3", "Pagina", "Il contenuto della pagina scelta: qui la pagina " + ui("App") + ", con la ricerca, "
         + ui("PREFERITI") + " e " + ui("TUTTE LE APP") + ".", rif("Aprire un'app")],
        ["4", "Schermo del telefono", "Lo schermo principale del telefono, in diretta, da usare col mouse.",
         rif("Lo schermo del telefono nel drawer")],
    ], "«TAB» — Le parti del drawer") + ul([
        "Il drawer segue il tema chiaro o scuro del desktop.",
        "Scrivendo con la tastiera mentre il drawer è in primo piano, il testo va nella ricerca " + ui("Cerca un'app")
        + ": " + key("Invio") + " apre la prima app trovata.",
        "Chiudere il drawer non chiude le app aperte; Phonestra esce quando non resta nessuna finestra ("
        + rif("Chiudere Phonestra") + ").",
    ])

S2 = p("Al centro della barra del titolo c'è la pillola del telefono: un pallino colorato, il nome del telefono e lo "
       "stato del collegamento.", lead=True) + \
    table(["Pallino", "Stato nella pillola", "Nella barra laterale", "Significato"], [
        [pill("grigio", "off"), ui("collegamento…"), ui("collegamento…"), "Phonestra cerca il telefono in rete."],
        [pill("verde", "ok"), ui("collegato via Wi-Fi"), ui("attivo"), "Tutto funziona."],
        [pill("arancione", "snooze"), ui("bloccato: sbloccalo"), ui("bloccato"), "Il telefono è bloccato: va "
         "sbloccato, Phonestra si ricollega da solo."],
        [pill("arancione", "snooze"), ui("riconnessione…"), ui("riconnessione…"), "Il collegamento è caduto: "
         "Phonestra riprova da solo."],
        [pill("rosso", "wait"), ui("Phonestra non parte sul telefono"), ui("non parte"), "Collegato, ma la parte di "
         "Phonestra che mostra le app non si avvia sul telefono (" + rif("Problemi frequenti") + ")."],
        [pill("grigio", "off"), ui("chiuso"), ui("chiuso"), "Phonestra si sta chiudendo."],
    ], "«TAB» — Gli stati del collegamento") + \
    p("Il clic sulla pillola apre il menu del telefono:") + \
    table(["Voce", "Che cosa fa"], [
        ["Intestazione", "Nome del telefono, modello e versione di Android, rete Wi-Fi e batteria."],
        [ui("Riconnetti"), "Riprova subito a collegarsi, senza aspettare il tentativo successivo. Compare solo quando "
         "il telefono non è utilizzabile."],
        [ui("Rinomina…"), "Cambia il nome del telefono in Phonestra (" + rif("Rinominare un telefono") + ")."],
        [ui("Spegni il Debug wireless alla chiusura"), "Spenta, segnata " + ui("In arrivo") + ": oggi Phonestra non "
         "spegne il Debug wireless alla chiusura (" + rif("Cosa resta acceso sul telefono") + ")."],
        [ui("Dimentica questo telefono…"), "Toglie il telefono da Phonestra (" + rif("Dimenticare un telefono") + ")."],
    ], "«TAB» — Il menu del telefono")

S3 = table(["Voce", "Che cosa fa", "Dove"], [
    [ui("App"), "La pagina con le app del telefono.", rif("Aprire un'app")],
    [ui("Notifiche"), "La pagina con le notifiche del telefono; il numero accanto dice quante sono.",
     rif("La pagina Notifiche")],
    ["<b>" + ui("TELEFONI") + "</b>", "Il telefono attivo, con il suo stato; gli altri telefoni collegati in passato, "
     "segnati " + ui("non attivo") + ".", rif("Passare a un altro telefono")],
    [ui("Aggiungi telefono"), "Apre " + ui("Aggiungi un telefono") + " per collegare un altro telefono.",
     rif("Aggiungere un altro telefono")],
    ["<b>" + ui("STRUMENTI") + "</b>", "", ""],
    [ui("Installa app…"), "Installa sul telefono un'app da un file " + c(".apk") + ".", rif("Installare un'app")],
    [ui("Invia file…"), "Copia file dal PC al telefono.", rif("Inviare file al telefono")],
    [ui("Ricevi file…"), "Copia file dal telefono al PC.", rif("Ricevere file dal telefono")],
    [ui("Preferenze"), "La pagina delle preferenze.", rif("La pagina Preferenze")],
    [ui("Informazioni"), "Nome, versione e autore di Phonestra.", ""],
], "«TAB» — Le voci della barra laterale")

VELO = table(["Titolo sullo schermo", "Testo", "Che cosa fare"], [
    [ui("Collegamento…"), ui("Il telefono deve essere acceso, sbloccato e sulla stessa rete Wi-Fi."), "Aspettare; "
     "se dura, controllare telefono e rete (" + rif("Problemi frequenti") + ")."],
    [ui("Telefono bloccato"), ui("Sbloccalo per continuare: mi ricollego da solo."), "Sbloccare il telefono."],
    [ui("Collegamento perso"), ui("Riprovo da solo in sottofondo.") + " " + ui("Se il telefono è bloccato, sbloccalo."),
     "Aspettare, o premere " + ui("Riconnetti ora") + "."],
    [ui("Phonestra non parte sul telefono"), "Il telefono non riesce ad avviare la parte di Phonestra che mostra le "
     "app, con il motivo tra parentesi.", "Premere " + ui("Riconnetti ora") + "; se non basta, riavviare il telefono."],
], "«TAB» — I messaggi sullo schermo del telefono nel drawer")

S4 = p("A destra del drawer c'è il telefono disegnato. Quando il collegamento funziona, dentro c'è lo schermo vero del "
       "telefono, in diretta, anche se lo schermo fisico del telefono è spento.", lead=True) + ul([
    "Si usa col mouse come le finestre delle app: il clic è un tocco, il trascinamento è un dito che scorre ("
    + rif("Mouse e touchpad") + ").",
    "Serve per le cose che non sono app: la schermata Home, la tendina delle notifiche, le impostazioni rapide, le app "
    "recenti, i widget. Si aprono come sul telefono, col mouse al posto del dito.",
    "Riceve i tasti solo dopo un clic sullo schermo; un clic fuori riporta i tasti alla ricerca delle app.",
    "Trascinando dei file sul telefono disegnato si mandano al telefono (" + rif("Inviare file al telefono") + ").",
    "Sotto lo schermo compare la scheda di un trasferimento in corso (" + rif("Il trasferimento in corso") + ").",
]) + p("Quando il telefono non si può usare, un velo sopra lo schermo spiega perché:") + VELO + \
    note("sopra lo schermo del telefono disegnato l'ora è quella del PC; batteria e rete sono quelle del telefono, "
         "aggiornate ogni mezzo minuto.", "Ora e batteria.")

S5 = p("I messaggi brevi compaiono in basso nella finestra e spariscono dopo qualche secondo. Nel drawer, per esempio:",
       lead=True) + ul([
    ui("Aspetta la fine del trasferimento in corso") + ": un trasferimento di file è già in corso.",
    ui("Il telefono non è collegato") + ": l'azione chiesta ha bisogno del collegamento.",
    ui("Rilettura delle app del telefono…") + ": dopo " + ui("Aggiorna ora") + " nelle preferenze.",
    ui("<nome> aggiunto: lo trovi tra i telefoni") + ": dopo " + ui("Aggiungi telefono") + ".",
]) + tip("i messaggi che finiscono con il nome di una cartella (per esempio dopo uno screenshot o un file ricevuto) "
         "dicono dove si trova il file sul PC.", "Dove sono i file.")

CHAPTER = ("Il drawer in breve", [
    ("Com'è fatto il drawer", S1),
    ("La pillola del telefono", S2),
    ("La barra laterale", S3),
    ("Lo schermo del telefono nel drawer", S4),
    ("I messaggi brevi", S5),
])
