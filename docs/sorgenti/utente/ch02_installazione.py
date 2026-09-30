from build import c, key, note, p, rif, steps, table, term, tip, ui, ul


S1 = table(["Requisito", "Dettaglio"], [
    ["PC", "Linux a 64 bit per processori Intel o AMD (x86_64), con un desktop grafico: GNOME, KDE, Xfce, Cinnamon o "
     "simili. Distribuzioni dal 2022 in poi (per esempio Ubuntu 22.04, Debian 12, Fedora, Arch)."],
    ["Telefono", "Android <b>14 o successivo</b>, di qualunque marca."],
    ["Rete", "PC e telefono sulla <b>stessa rete Wi-Fi</b>, per esempio quella di casa. Non vanno bene la rete "
     "«ospiti» né i dati mobili del telefono. Il PC può essere collegato al router anche col cavo di rete."],
    ["Audio", "Le casse o le cuffie del PC. L'audio passa dal sistema audio del desktop (PipeWire o PulseAudio)."],
    ["Da installare", "Niente: Phonestra è un file unico, un'AppImage, che contiene tutto quello che serve. Non "
     "servono " + c("adb") + " né altri programmi, né la password di amministratore."],
], "«TAB» — I requisiti") + \
    note("il telefono deve essere <b>sbloccato</b> mentre si usano le app dal PC: è una regola di sicurezza di "
         "Android. Phonestra spegne lo schermo del telefono mentre lo si usa dal PC e lo riaccende alla fine ("
         + rif("Lo schermo del telefono") + ").", "Telefono sbloccato.")

S2 = steps([
    "Aprire nel browser la pagina delle versioni di Phonestra: " + c("https://github.com/nic-fio/PHONESTRA/releases") + ".",
    "Scaricare il file " + c("Phonestra-<versione>-x86_64.AppImage") + " dell'ultima versione.",
    "Spostare il file dove si preferisce, per esempio nella cartella personale. Phonestra funziona da lì: non va "
    "installato.",
    "Rendere il file eseguibile, una volta sola: nel file manager, clic destro sul file, " + "«Proprietà»" + ", "
    "scheda " + "«Permessi»" + ", casella per consentirne l'esecuzione come programma (il nome esatto cambia da un "
    "desktop all'altro). Oppure da un terminale, con il comando qui sotto.",
]) + term("""
$ chmod +x Phonestra-*-x86_64.AppImage
""", "Rendere eseguibile l'AppImage") + \
    tip("la pagina delle versioni riporta anche l'impronta SHA-256 del file. Chi vuole controllare che il file "
        "scaricato sia integro può confrontarla con quella calcolata da " + c("sha256sum") + ".", "Controllo facoltativo.")

S3 = p("Si avvia Phonestra con un doppio clic sul file, oppure da un terminale:", lead=True) + term("""
$ ./Phonestra-*-x86_64.AppImage
""") + table(["Situazione", "Che cosa si apre"], [
    ["Primo avvio: nessun telefono collegato finora", "La finestra " + ui("Aggiungi un telefono") + ", che guida "
     "il primo collegamento (" + rif("Collegare il telefono") + ")."],
    ["Telefono già collegato una volta, acceso e in rete", "Il drawer, la finestra principale di Phonestra: si "
     "collega da solo in pochi secondi (" + rif("Il drawer in breve") + ")."],
    ["Telefono già collegato, ma non raggiungibile", "Il drawer, con " + ui("Collegamento a <nome>…") + " e l'avviso "
     + ui("Il telefono deve essere acceso, sbloccato e sulla stessa rete Wi-Fi.") + " Phonestra continua a cercarlo "
     "in sottofondo e si collega appena lo trova."],
], "«TAB» — Che cosa si apre all'avvio") + ul([
    "Phonestra non aggiunge icone al menu del sistema: per riaprirlo si avvia di nuovo il file.",
    "Se Phonestra è già aperto, un secondo avvio non apre un secondo Phonestra: riporta davanti il drawer (o lo riapre "
    "se era stato chiuso).",
])

S4 = p("Phonestra si chiude quando si chiudono <b>tutte</b> le sue finestre: il drawer e le finestre delle app. "
       "Chiudendo solo il drawer, le app aperte restano aperte.", lead=True) + steps([
    "Chiudere le finestre delle app e il drawer, con la " + ui("×") + " in alto a destra.",
    "Phonestra mette in pausa la musica o i video che il telefono stava suonando, chiude le app aperte nelle "
    "finestre e rimette il telefono com'era: schermo riacceso, tempo di spegnimento e volume di prima.",
    "Dopo qualche secondo il programma esce.",
]) + note("se Phonestra è stato avviato da un terminale, anche " + key("Ctrl", "C") + " nel terminale lo chiude nello "
          "stesso modo, rimettendo a posto il telefono.", "Dal terminale.") + \
    note("se il collegamento cade all'improvviso (PC spento, Wi-Fi perso), il telefono si rimette a posto da solo "
         "pochi secondi dopo; quello che eventualmente resta, Phonestra lo sistema al collegamento successivo ("
         + rif("Cosa cambia sul telefono") + ").", "Chiusure improvvise.")

S5 = steps([
    "Scaricare la nuova versione, come la prima volta, e renderla eseguibile.",
    "Chiudere Phonestra, se è aperto.",
    "Avviare la nuova versione. Il telefono, le preferenze e i preferiti restano: stanno in "
    + c("~/.config/Phonestra") + ", non nel file dell'AppImage.",
    "Cancellare il file della versione vecchia.",
]) + p("La versione in uso si legge in " + ui("Informazioni") + ", in fondo alla barra laterale del drawer.")

CHAPTER = ("Installazione e primo avvio", [
    ("Requisiti", S1),
    ("Scaricare Phonestra", S2),
    ("Avviare Phonestra", S3),
    ("Chiudere Phonestra", S4),
    ("Aggiornare Phonestra", S5),
])
