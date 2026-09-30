from build import c, note, p, rif, table, ui


S1 = p("La pagina " + ui("Preferenze") + " si apre dalla barra laterale del drawer. Ogni cambiamento vale subito e "
       "si salva da solo: non c'è un pulsante per salvare.", lead=True) + \
    table(["Scheda", "Voce", "Che cosa fa", "All'inizio"], [
        [ui("FINESTRE DELLE APP"), ui("Esc torna indietro"), ui("Il tasto Esc fa come «Indietro» di Android. "
         "Disattivalo se un'app usa Esc per altro."), "acceso"],
        [ui("NOTIFICHE"), ui("Avviso a comparsa"), "Un avviso del sistema per ogni notifica nuova del telefono ("
         + rif("Gli avvisi a comparsa") + ").", "acceso"],
        [ui("NOTIFICHE"), ui("Solo il nome dell'app"), "Negli avvisi niente mittente né testo.", "spento"],
        [ui("NOTIFICHE"), ui("App che possono avvisare"), "Un interruttore per app: quali possono fare avvisi ("
         + rif("Scegliere gli avvisi") + ").", ui("tutte ›")],
        [ui("FILE"), ui("File inviati al telefono"), ui("Cartella del telefono in cui arrivano.") + " Una tra "
         + ui("Download") + ", " + ui("Documenti") + ", " + ui("Immagini") + ", " + ui("Fotocamera") + ", "
         + ui("Musica") + ", " + ui("Video") + ".", ui("Download")],
        [ui("FILE"), ui("File ricevuti dal telefono"), ui("Cartella del PC in cui arrivano.") + " Il pulsante apre "
         "la finestra " + ui("Dove salvare i file ricevuti dal telefono") + ".", ui("Scaricati")],
        [ui("APP DEL TELEFONO"), ui("Elenco delle app"), ui("Si aggiorna da solo; usa il pulsante se manca un'app "
         "appena installata.") + " Il pulsante è " + ui("Aggiorna ora") + ".", "—"],
    ], "«TAB» — La pagina Preferenze") + \
    note("le preferenze valgono per tutti i telefoni. I preferiti e il nome invece sono di ogni telefono.",
         "Per tutti i telefoni.")

S2 = p("Le preferenze stanno nel file " + c("preferenze.toml") + ", nella cartella " + c("~/.config/Phonestra")
       + " (" + rif("Le cartelle sul PC") + "). Non serve aprirlo: la pagina " + ui("Preferenze") + " basta.",
       lead=True) + \
    p("Per tornare alle preferenze iniziali si chiude Phonestra e si cancella " + c("preferenze.toml") + ": al "
      "prossimo avvio ogni voce riprende il valore di partenza. Il telefono collegato non si perde, perché sta in un "
      "altro file.") + \
    note("scegliendo di nuovo la cartella " + ui("Scaricati") + " in " + ui("File ricevuti dal telefono") + ", "
         "Phonestra segue la cartella dei download del desktop anche se in futuro cambia.", "Scaricati.")

CHAPTER = ("Preferenze", [
    ("La pagina Preferenze", S1),
    ("Dove si salvano le preferenze", S2),
])
