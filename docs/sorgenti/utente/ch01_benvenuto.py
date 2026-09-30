from build import VERSION, arrow, box, c, fig, note, p, rif, table, text, tip, ui, ul, zone


SCHEMA = fig(
    zone(20, 14, 360, 196, "PC Linux")
    + box(40, 46, 320, 44, "Drawer di Phonestra", "le app, le notifiche, lo schermo del telefono", "navy")
    + box(40, 100, 155, 44, "Finestra di un'app", "una per ogni app", "blue")
    + box(205, 100, 155, 44, "Finestra di un'app", "si usa col mouse", "blue")
    + box(40, 154, 320, 40, "Casse, mouse, tastiera e appunti del PC", "", "soft", 12)
    + zone(520, 14, 360, 196, "Telefono Android 14 o successivo")
    + box(540, 46, 320, 44, "Le app del telefono", "girano sul telefono, come sempre", "blue")
    + box(540, 100, 320, 44, "Debug wireless", "un'impostazione di Android, accesa una volta", "dark")
    + box(540, 154, 320, 40, "Nessuna app da installare", "", "soft", 12)
    + arrow(516, 70, 384, 70, "#0050C0") + text(450, 58, "immagini e suono", 11, "#003a90", "700")
    + arrow(384, 124, 516, 124, "#16a34a") + text(450, 112, "clic, tasti, file", 11, "#166534", "700")
    + text(450, 176, "stessa rete Wi-Fi", 11.5, "#334155", "700")
    + text(450, 192, "collegamento cifrato", 11, "#475569"),
    900, 224, "«FIG» — Phonestra mostra sul PC le app che girano sul telefono")

S1 = p("Phonestra è un programma per Linux che porta le app del telefono Android sul PC. Ogni app si apre nella sua "
       "finestra, come un programma del PC, e si usa con mouse e tastiera. Il telefono resta sul tavolo: basta che sia "
       "acceso, sbloccato e sulla stessa rete Wi-Fi del PC.", lead=True) + SCHEMA + \
    p("Le app continuano a girare sul telefono, con i loro dati e i loro account: Phonestra ne mostra l'immagine sul PC "
      "e manda al telefono i clic e i tasti. Con Phonestra si può:") + ul([
        "aprire le app del telefono in finestre del PC, anche più app insieme;",
        "scrivere con la tastiera del PC e copiare e incollare testo nei due sensi;",
        "sentire l'audio del telefono dalle casse del PC;",
        "vedere le notifiche del telefono sul PC;",
        "installare app, mandare file al telefono e riceverne;",
        "fare screenshot e registrare lo schermo di un'app.",
    ]) + \
    note("sul telefono non si installa nessuna app. Si accendono solo alcune impostazioni di Android, una volta sola "
         "(" + rif("Collegare il telefono") + "). Quello che Phonestra cambia sul telefono durante l'uso, lo rimette "
         "com'era alla fine (" + rif("Cosa cambia sul telefono") + ").", "Niente sul telefono.")

S2 = table(["Chi", "Cosa fa con Phonestra", "Capitoli"], [
    ["Chi usa Phonestra", "Scarica il programma, collega il telefono la prima volta e usa le app dal PC.", "2 - 14"],
    ["Chi vuole sapere cosa succede al telefono", "Legge che cosa cambia sul telefono, che cosa resta acceso e "
     "dove Phonestra tiene i suoi dati.", "15"],
    ["Chi ha un problema", "Cerca il sintomo o il messaggio e il rimedio.", "16"],
], "«TAB» — I lettori di questo manuale") + \
    p("Chi usa Phonestra per la prima volta può seguire il " + rif("Primi passi") + ": porta dal file scaricato alla "
      "prima app aperta sul PC in pochi passi. Il " + rif("Glossario") + " spiega le parole meno comuni.")

S3 = table(["Convenzione", "Significato"], [
    [ui("Aggiungi telefono"), "Testo che compare nell'interfaccia: pulsanti, voci di menu, etichette, messaggi."],
    [c("~/.config/Phonestra"), "Nomi di file e cartelle, comandi, valori da scrivere così come sono."],
    ["Riquadro blu <b>Nota</b>", "Informazione utile per capire."],
    ["Riquadro verde <b>Consiglio</b>", "Un modo più semplice o più sicuro di fare una cosa."],
    ["Riquadro giallo <b>Attenzione</b>", "Un comportamento che può sorprendere o far perdere dati."],
], "«TAB» — Le convenzioni del manuale") + \
    p("Nel manuale «telefono» vuol dire sempre il telefono Android collegato a Phonestra; «PC» il computer con "
      "Linux. I nomi delle impostazioni di Android (per esempio " + ui("Debug wireless") + ") possono cambiare un "
      "poco da una marca all'altra: il manuale usa quelli più comuni.") + \
    note("questo manuale descrive Phonestra " + VERSION + ". Il funzionamento interno è descritto nel Manuale "
         "Tecnico (" + c("docs/Phonestra_Manuale_Tecnico.html") + "). Alcune voci dell'interfaccia sono segnate "
         + ui("In arrivo") + ": non fanno ancora niente e il manuale le cita solo per dirlo.", "Versione.") + \
    tip("Phonestra segue il tema chiaro o scuro del desktop. Le figure del manuale sono schemi, non fotografie: "
        "colori e proporzioni sul proprio schermo possono essere diversi.", "Aspetto.")

CHAPTER = ("Benvenuto", [
    ("Che cos'è Phonestra", S1),
    ("A chi è rivolto questo manuale", S2),
    ("Convenzioni", S3),
])
