from build import note, p, rif, steps, tip, ui, ul, warn


S1 = p("Phonestra può ricordare più telefoni, ma ne usa <b>uno alla volta</b>. Il telefono in uso è quello "
       "«attivo»; gli altri compaiono nella barra laterale del drawer, sotto " + ui("TELEFONI") + ", con la scritta "
       + ui("non attivo") + ".", lead=True) + steps([
    "Nel drawer premere " + ui("Aggiungi telefono") + ", nella barra laterale.",
    "Si apre " + ui("Aggiungi un telefono") + ": seguire i passi come per il primo telefono ("
    + rif("Collegare il telefono") + ").",
    "Alla fine il drawer dice " + ui("<nome> aggiunto: lo trovi tra i telefoni") + ". Il telefono attivo non cambia.",
]) + note("Phonestra mostra solo i telefoni collegati con " + ui("Aggiungi telefono") + ", mai quelli di altre persone "
          "sulla stessa rete.", "Solo i propri telefoni.")

S2 = steps([
    "Nella barra laterale fare clic sul telefono " + ui("non attivo") + " (il suggerimento dice "
    + ui("Passa a <nome>") + ").",
    "Se ci sono app aperte, Phonestra chiede conferma: " + ui("Chiudere N app di «<attivo>» e passare a «<altro>»?")
    + " con " + ui("Un solo telefono alla volta: le finestre delle app si chiudono.") + " Premere " + ui("Passa") + ".",
    "Phonestra chiude le finestre, rimette a posto il telefono di prima e si riavvia collegato all'altro telefono.",
]) + tip("il telefono scelto diventa quello che Phonestra apre ai prossimi avvii.", "Il prossimo avvio.")

S3 = steps([
    "Fare clic sulla pillola del telefono, in alto nel drawer, e scegliere " + ui("Rinomina…") + ".",
    "Nella finestra " + ui("Rinomina il telefono") + " scrivere il nuovo nome e premere " + ui("Rinomina") + ".",
]) + p("Il nome cambia solo in Phonestra (" + ui("Il nome si vede solo in Phonestra.") + "): il nome del telefono "
       "nelle sue impostazioni resta quello di prima. Il nome iniziale è quello che il telefono dà a se stesso.")

S4 = p("«Dimenticare» un telefono lo toglie da Phonestra. Serve, per esempio, quando si cambia telefono o lo si "
       "regala.", lead=True) + steps([
    "Fare clic sulla pillola del telefono e scegliere " + ui("Dimentica questo telefono…") + ".",
    "Leggere la finestra " + ui("Dimenticare «<nome>»?") + " e premere " + ui("Dimentica") + ".",
    "Phonestra si chiude, rimettendo il telefono com'era. Se restano altri telefoni, si riavvia collegato al primo; "
    "altrimenti, al prossimo avvio si apre " + ui("Aggiungi un telefono") + ".",
]) + warn("dimenticare il telefono non lo scollega dal lato del telefono: il PC resta associato. Per toglierlo, sul "
          "telefono: " + ui("Debug wireless") + " › " + ui("Dispositivi associati") + " (o, se era stato collegato col "
          "cavo, " + ui("Opzioni sviluppatore") + " › " + ui("Revoca autorizzazioni debug USB") + ").",
          "Anche sul telefono.") + \
    p("Per collegarlo di nuovo basta " + ui("Aggiungi telefono") + ".")

CHAPTER = ("Più telefoni", [
    ("Aggiungere un altro telefono", S1),
    ("Passare a un altro telefono", S2),
    ("Rinominare un telefono", S3),
    ("Dimenticare un telefono", S4),
])
