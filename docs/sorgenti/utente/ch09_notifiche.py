from build import note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("La pagina " + ui("Notifiche") + " del drawer mostra le notifiche del telefono, dalla più recente. Il numero "
       "accanto a " + ui("Notifiche") + " nella barra laterale dice quante sono.", lead=True) + ul([
    "Le notifiche sono raggruppate per app, in schede col nome dell'app e il loro numero.",
    "Di ogni app si vedono le ultime 2; il pulsante " + ui("altre N notifiche di <app> ›") + " mostra le altre.",
    "Ogni notifica ha icona, titolo, testo e ora. Un clic sulla notifica apre l'app nella sua finestra.",
    "La " + ui("×") + " accanto a una notifica la nasconde: " + ui("Nascondi (sul telefono resta)") + ". "
    + ui("Nascondi tutte") + ", in alto, le nasconde tutte.",
    "Senza notifiche la pagina dice " + ui("Nessuna notifica") + ".",
    "Le notifiche si aggiornano ogni pochi secondi, finché il telefono è collegato.",
]) + note("nascondere una notifica la toglie solo da Phonestra: sul telefono resta. Se l'app la aggiorna (per "
          "esempio arriva un nuovo messaggio nella stessa chat), ricompare.", "Nascondere non cancella.") + \
    p("Non compaiono le notifiche fisse, quelle che restano finché un'app lavora (per esempio la musica in "
      "riproduzione o un navigatore), né le notifiche senza titolo e senza testo.")

S2 = p("Quando sul telefono arriva una notifica nuova, Phonestra mostra un avviso del sistema, come quelli degli altri "
       "programmi del PC: compare nell'angolo dello schermo e resta nell'elenco delle notifiche del desktop.",
       lead=True) + \
    table(["Parte dell'avviso", "Contenuto"], [
        ["Programma", "Phonestra"],
        ["Icona", "L'icona dell'app del telefono"],
        ["Titolo", "Il nome dell'app e il titolo della notifica (per esempio il mittente)"],
        ["Testo", "Il testo della notifica"],
        ["Azione", ui("Apri") + ", o un clic sull'avviso: apre l'app nella sua finestra"],
    ], "«TAB» — Com'è fatto un avviso") + ul([
        "Gli avvisi rispettano il «Non disturbare» del desktop.",
        "Gli avvisi arrivano solo mentre Phonestra è aperto e collegato al telefono.",
    ]) + warn("all'apertura del drawer, le notifiche già presenti sul telefono possono produrre un avviso ciascuna, "
              "come se fossero nuove. È un problema noto: gli avvisi successivi riguardano solo le notifiche nuove.",
              "Avvisi all'avvio.")

S3 = p("Nella pagina " + ui("Preferenze") + ", scheda " + ui("NOTIFICHE") + ", si sceglie che cosa mostrano gli "
       "avvisi.", lead=True) + \
    table(["Preferenza", "Che cosa fa", "All'inizio"], [
        [ui("Avviso a comparsa"), ui("Un avviso del sistema quando arriva una notifica sul telefono.") + " Spento: "
         "le notifiche restano solo nella pagina " + ui("Notifiche") + ".", "acceso"],
        [ui("Solo il nome dell'app"), ui("Negli avvisi niente mittente né testo: utile se altri vedono il tuo "
         "schermo.") + " L'avviso dice solo il nome dell'app e " + ui("Nuova notifica") + ".", "spento"],
        [ui("App che possono avvisare"), ui("Scegli da quali app ricevere gli avvisi.") + " Il pulsante dice "
         + ui("tutte ›") + " o " + ui("tutte tranne N ›") + ".", "tutte"],
    ], "«TAB» — Le preferenze degli avvisi") + steps([
        "Aprire " + ui("Preferenze") + " nella barra laterale.",
        "Nella scheda " + ui("NOTIFICHE") + " premere il pulsante accanto a " + ui("App che possono avvisare") + ".",
        "Nella finestra " + ui("App che possono avvisare") + " spegnere l'interruttore delle app che non devono "
        "fare avvisi. La scelta si salva subito.",
    ]) + tip("le app silenziate qui continuano a comparire nella pagina " + ui("Notifiche") + ": la scelta vale solo "
             "per gli avvisi a comparsa.", "Silenziate, non nascoste.") + \
    note("Phonestra legge titolo e testo di tutte le notifiche del telefono, anche di quelle che il telefono nasconde "
         "sulla schermata di blocco. Restano sul PC, in Phonestra (" + rif("Privacy e sicurezza") + ").",
         "Notifiche private.")

CHAPTER = ("Notifiche", [
    ("La pagina Notifiche", S1),
    ("Gli avvisi a comparsa", S2),
    ("Scegliere gli avvisi", S3),
])
