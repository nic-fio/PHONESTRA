from build import c, p, rif, table, term, ul, warn

S1 = p("Il codice di Phonestra si legge come i suoi documenti: in italiano, con il perché accanto al come.",
       lead=True) + ul([
    "Tutto in italiano: nomi, commenti, messaggi del programma, documenti. Parole semplici, frasi brevi.",
    "I commenti spiegano il perché e rimandano alla fonte: " + c("SPECIFICHE §7.3") + ", " + c("prove §49") + " ("
    + c("memoria/prove-collegamento.md") + "), " + c("memoria/componente.md") + ".",
    "Rust: " + c("anyhow") + " per gli errori, con " + c("context") + " che dice cosa si stava facendo; niente "
    + c("unwrap") + " dove un telefono diverso può rispondere altro.",
    "Java: una classe per pezzo, riflessione solo in " + c("Nascoste") + " e nei pezzi che la usano, errori catturati "
    "nel thread che li produce.",
    "Codice di terzi: mai copiato. scrcpy e AOSP si leggono come documentazione.",
])

S2 = p("Ogni commit lascia il progetto compilato, provato e documentato.", lead=True) + term("""
$ cargo build && cargo test && cargo clippy --all-targets
""") + p("Nessun avviso di clippy. Se hai cambiato il componente, il jar ricompilato va nello stesso commit. Se hai "
         "cambiato un comportamento, aggiorna il capitolo che lo descrive in " + c("docs/sorgenti/tecnico/") + " (e, se lo vede l'utente, in " + c("docs/sorgenti/utente/") + ")"
         + " e rigenera i manuali (" + c("python3 docs/sorgenti/build.py") + "); se hai aggiunto un file di "
         "sorgente, dagli una riga nella mappa (" + c("ch17_mappa.py") + ", " + rif("Appendice B — Mappa dei file") + "): "
         + c("cargo test") + " lo pretende.")

S3 = p("Il repository è pubblico: niente che identifichi persone, telefoni o reti.", lead=True) + warn("il repository è pubblico. Niente nomi di persone, numeri di serie, nomi di reti Wi-Fi, indirizzi, "
          "schermate vere non sfocate. Nei test si usano valori finti (per esempio " + c("R5CT0000000") + "); i file "
          "prodotti dalle prove (" + c("phonestra-prova.*") + ") sono ignorati da git.", "Dati personali.")

S4 = p("Il perché del progetto sta in " + c("memoria/") + ", un file per tema. Le decisioni si scrivono lì, "
       "con il loro perché.", lead=True) + table(["File", "Cosa ci va"], [
    [c("memoria/prossima-sessione.md"), "Da dove ripartire: si legge all'inizio di ogni sessione di lavoro"],
    [c("memoria/decisioni-utente.md"), "Le decisioni e il loro perché, comprese le alternative scartate"],
    [c("memoria/registro-problemi.md"), "Problema → causa → soluzione → stato"],
    [c("memoria/prove-collegamento.md"), "Le misure sul telefono, per sezioni numerate (§)"],
    [c("memoria/componente.md") + ", " + c("adb.md") + ", " + c("api-android.md") + ", " + c("studio/"),
     "Progetto e studio del componente e del trasporto"],
    [c("memoria/interfaccia.md"), "Regole dell'interfaccia"],
], "«TAB» — Dove si registrano le decisioni") + \
    p("Prima di proporre un'alternativa, controlla che non sia già stata scartata (per esempio Flatpak, icone nel "
      "menu di sistema, temi scuri come stile, più telefoni attivi insieme).")

CHAPTER = ("Convenzioni", [
    ("Lingua e stile", S1),
    ("Prima di ogni commit", S2),
    ("Dati personali", S3),
    ("Registrare le decisioni", S4),
])
