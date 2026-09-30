from build import c, key, note, p, rif, steps, table, tip, ui, ul


S1 = p("Lo screenshot fotografa l'app di una finestra alla risoluzione del telefono, senza la barra della finestra.",
       lead=True) + table(["Comando", "Dove", "Risultato"], [
    [ui("Screenshot (salvato e copiato)"), "Pulsante con la macchina fotografica nella barra della finestra",
     "Salva l'immagine in " + c("Immagini/Phonestra") + " e la copia negli appunti del PC: "
     + ui("Screenshot in Immagini/Phonestra e negli appunti") + "."],
    [ui("Copia screenshot"), ui("Altri comandi") + " (⋮), o " + key("Ctrl", "Maiusc", "C"), "La copia soltanto negli "
     "appunti: " + ui("Screenshot copiato negli appunti") + "."],
], "«TAB» — Screenshot") + ul([
    "Il file si chiama col nome dell'app, la data e l'ora: per esempio " + c("Mappe 2026-09-30 10.42.05.png") + ".",
    "L'immagine negli appunti si incolla subito in un altro programma del PC (una mail, una chat, un documento).",
    "Se l'app non mostra ancora niente compare " + ui("Nessuna immagine da fotografare") + ".",
]) + note("una schermata protetta (" + rif("Schermate protette") + ") risulta nera anche nello screenshot.",
          "Schermate protette.")

S2 = steps([
    "Nella finestra dell'app premere " + ui("Registra lo schermo") + " (il pulsante col pallino rosso).",
    "Il pulsante diventa una pillola rossa con il tempo trascorso, per esempio " + ui("● 0:42") + ".",
    "Per fermare, premere di nuovo il pulsante: compare " + ui("Registrazione salvata in Video/Phonestra") + ".",
]) + ul([
    "Il video è un file MP4 in " + c("Video/Phonestra") + ", col nome dell'app, la data e l'ora, per esempio "
    + c("Mappe 2026-09-30 10.42.05.mp4") + ".",
    "Il video si salva così come arriva dal telefono, senza ricodificarlo: pesa poco e non rallenta il PC.",
    "C'è anche l'audio, che è quello di tutto il telefono, non della sola app.",
    "Durante la registrazione l'app non cambia forma: ridimensionando la finestra l'immagine si adatta, ma il video "
    "resta della misura iniziale. " + ui("Ruota") + " non funziona.",
    "Chiudendo la finestra durante una registrazione, il video si salva prima che la finestra si chiuda.",
]) + tip("per registrare con l'audio, conviene aspettare qualche secondo dopo il collegamento: l'audio parte poco dopo "
         "le immagini (" + rif("L'audio dal PC") + ").", "Con l'audio.")

CHAPTER = ("Screenshot e registrazione", [
    ("Fare uno screenshot", S1),
    ("Registrare lo schermo", S2),
])
