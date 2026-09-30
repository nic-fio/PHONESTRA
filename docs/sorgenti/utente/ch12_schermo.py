from build import box, fig, note, p, path, rif, steps, table, text, tip, ui, ul, warn


STATI = fig(
    box(40, 80, 280, 70, "Schermo spento", "si usa il telefono dal PC", "navy")
    + box(580, 80, 280, 70, "Schermo acceso: telefono «in mano»", "si usa il telefono con le mani", "amber")
    + path([(320, 98), (580, 98)], "#d97706")
    + text(450, 34, "sblocco a mano dopo un blocco", 11.5, "#9a3412", "600")
    + text(450, 52, "chiamata in arrivo", 11.5, "#9a3412", "600")
    + text(450, 70, "fine di una chiamata", 11.5, "#9a3412", "600")
    + path([(580, 132), (320, 132)], "#0050C0")
    + text(450, 172, "un clic, un tasto o la rotellina da una finestra di Phonestra", 11.5, "#003a90", "600")
    + text(450, 190, "oppure telefono non toccato per il suo tempo di spegnimento", 11.5, "#003a90", "600"),
    900, 206, "«FIG» — Quando lo schermo del telefono si accende e quando si rispegne")

S1 = p("Appena collegato, Phonestra spegne lo schermo del telefono. Il telefono resta acceso e sbloccato: le app "
       "continuano a girare e a rispondere al mouse del PC.", lead=True) + ul([
    "Lo schermo spento fa risparmiare batteria e tiene riservato quello che si fa dal PC.",
    "Android permette di mostrare le app sul PC solo col telefono sbloccato: per questo Phonestra non lo lascia "
    "addormentare. Durante il collegamento porta al massimo il tempo di spegnimento dello schermo e alla fine lo "
    "rimette com'era.",
    "Le notifiche che arrivano non riaccendono lo schermo: il telefono vibra soltanto.",
    "Lo schermo del telefono nel drawer continua a mostrare, in diretta, quello che il telefono avrebbe sullo schermo.",
]) + warn("con lo schermo spento il telefono non è addormentato: il vetro può ancora ricevere tocchi. Non metterlo in "
          "tasca o in borsa mentre Phonestra è collegato.", "Tocchi a schermo spento.")

S2 = p("Quando si riprende in mano il telefono, lo schermo si riaccende e resta acceso; quando lo si lascia di nuovo, "
       "si rispegne.", lead=True) + STATI + \
    table(["Che cosa succede", "Lo schermo del telefono"], [
        ["Il telefono, bloccato durante l'uso, viene sbloccato a mano", "Resta acceso: il telefono è «in mano»."],
        ["Arriva una chiamata", "Si accende, per rispondere dal telefono."],
        ["Finisce una chiamata", "Resta acceso, poi vale la regola dei tocchi."],
        ["Si chiude Phonestra", "Si riaccende, e il telefono torna com'era (" + rif("Cosa cambia sul telefono") + ")."],
        ["Col telefono «in mano», un clic, un tasto o la rotellina in una finestra di Phonestra", "Si rispegne: si "
         "torna a usare il telefono dal PC."],
        ["Col telefono «in mano», nessun tocco per il tempo di spegnimento scelto sul telefono", "Si rispegne, come "
         "farebbe il telefono da solo."],
    ], "«TAB» — Chi accende e chi spegne lo schermo del telefono") + \
    note("finché il drawer è aperto, chiudere l'ultima app non riaccende lo schermo del telefono: il drawer mostra "
         "ancora lo schermo del telefono e lo tiene spento.", "Col drawer aperto.")

S3 = p("Se il telefono si blocca (per esempio col tasto di accensione, o con un doppio tocco sullo schermo spento, un "
       "gesto di alcune marche), le app non si possono più mostrare: è una regola di sicurezza di Android.",
       lead=True) + steps([
    "Il drawer mostra " + ui("Telefono bloccato") + " con " + ui("Sbloccalo per continuare: mi ricollego da solo.")
    + "; le finestre delle app dicono " + ui("Telefono bloccato: sbloccalo per continuare") + ".",
    "Sbloccare il telefono con il PIN, l'impronta o il volto.",
    "Phonestra si ricollega da solo e le app tornano nelle loro finestre, dov'erano. Lo schermo del telefono resta "
    "acceso, perché il telefono è «in mano»; si rispegne al primo clic dal PC.",
]) + note("su alcuni telefoni (per esempio i Samsung) il blocco fa cadere il collegamento: per qualche secondo compare "
          + ui("riconnessione…") + ". È normale.", "Qualche secondo di attesa.")

S4 = p("Le chiamate si fanno e si ricevono col telefono in mano (" + rif("Le chiamate") + ").", lead=True) + ul([
    "Quando il telefono squilla, Phonestra riaccende lo schermo: si risponde dal telefono come sempre.",
    "Durante la chiamata lo schermo non si rispegne da solo, anche se il telefono non viene toccato (all'orecchio non "
    "lo si tocca).",
    "Finita la chiamata lo schermo resta acceso; si rispegne al primo clic dal PC o dopo il tempo di spegnimento del "
    "telefono.",
]) + tip("una chiamata di un'app aperta in una finestra (per esempio WhatsApp) si può accettare anche col clic sul "
         "PC; la voce però passa sempre dal telefono.", "Rispondere dal PC.")

CHAPTER = ("Lo schermo del telefono", [
    ("Perché lo schermo si spegne", S1),
    ("Il telefono in mano", S2),
    ("Telefono bloccato", S3),
    ("Le chiamate in arrivo", S4),
])
