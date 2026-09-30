from build import c, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("Phonestra installa sul telefono le app scaricate come file " + c(".apk") + ", senza cavo e senza passare dal "
       "telefono.", lead=True) + steps([
    "Nel drawer premere " + ui("Installa app…") + " e scegliere il file (la finestra " + ui("Scegli l'app da "
    "installare") + " mostra solo i file " + ui("App Android (.apk)") + "). Oppure trascinare il file " + c(".apk")
    + " sul telefono disegnato a destra del drawer.",
    "Phonestra chiede conferma: " + ui("Installare «<file>»?") + ". Premere " + ui("Installa") + ".",
    "La scheda del trasferimento, sul telefono disegnato, mostra l'avanzamento, poi " + ui("installazione in corso…")
    + ".",
    "Alla fine compare " + ui("«<file>» installata") + " e l'app entra nell'elenco del drawer.",
]) + warn("installando dal PC Android non chiede la solita conferma per le «origini sconosciute». Installare solo app "
          "di cui ci si fida: la finestra di conferma lo ricorda.", "Solo app fidate.") + \
    table(["Motivo nel messaggio", "Che cosa fare"], [
        ["sul telefono c'è già quest'app firmata da qualcun altro: disinstallala prima", "Disinstallare l'app già "
         "presente, poi riprovare."],
        ["sul telefono c'è già una versione più recente", "Niente: il telefono ha già una versione più nuova."],
        ["l'app è per una versione di Android troppo vecchia e Android la rifiuta", "Cercare una versione più recente "
         "dell'app. Phonestra non forza l'installazione."],
        ["l'app richiede una versione di Android più recente di quella del telefono", "L'app non va su questo telefono."],
        ["sul telefono non c'è abbastanza spazio", "Liberare spazio sul telefono."],
        ["il telefono non permette di installare dal PC (sugli Xiaomi: attiva «Installa tramite USB» nelle Opzioni "
         "sviluppatore)", "Seguire l'indicazione, poi riprovare."],
        ["Play Protect ha bloccato l'app", "Il controllo di sicurezza di Google ha fermato l'app: meglio non "
         "installarla."],
        ["il file non è un'app Android valida", "Il file è rovinato o non è un'app: scaricarlo di nuovo."],
        ["l'app non è fatta per il processore di questo telefono", "Cercare la versione dell'app per questo telefono."],
    ], "«TAB» — Perché un'installazione può non riuscire") + \
    note("i pacchetti divisi in più file (" + c(".apks") + ", " + c(".xapk") + ", " + c(".apkm") + ") non si "
         "installano: Phonestra accetta solo i file " + c(".apk") + ".", "Solo .apk.")

S2 = steps([
    "Nel drawer, clic destro sull'app e " + ui("Disinstalla…") + ".",
    "Phonestra chiede conferma: " + ui("Disinstallare <app>?") + " con " + ui("L'app e i suoi dati verranno tolti da "
    "«<telefono>».") + " Premere " + ui("Disinstalla") + ".",
    "Se l'app era aperta, la sua finestra si chiude. Alla fine compare " + ui("<app> disinstallata") + ": l'app esce "
    "dall'elenco e dai preferiti.",
]) + note("le app di sistema (quelle che il telefono aveva già) non si disinstallano: la voce è spenta, con "
          + ui("App di sistema: non si può disinstallare") + ".", "App di sistema.")

S3 = p("I file mandati dal PC arrivano in una cartella del telefono, " + ui("Download") + " se non si è scelto "
       "altro, e compaiono subito nella Galleria e nell'app dei file del telefono.", lead=True) + steps([
    "Nel drawer premere " + ui("Invia file…") + " e scegliere uno o più file (" + ui("Scegli i file da inviare") + "). "
    "Oppure trascinarli dal file manager sul telefono disegnato: compare " + ui("Rilascia per inviare al telefono") + ".",
    "La scheda del trasferimento mostra il nome del file e " + ui("Invio al telefono · <inviati> di <totale>") + ".",
    "Alla fine compare " + ui("«<file>» è in Download sul telefono") + " (o nella cartella scelta).",
]) + ul([
    "Un file con lo stesso nome di uno già presente non lo sostituisce: arriva con un numero, per esempio "
    + c("foto (1).jpg") + ".",
    "Un file " + c(".apk") + " trascinato sul telefono non viene copiato ma installato, con conferma.",
    "La cartella di arrivo sul telefono si sceglie in " + ui("Preferenze") + " › " + ui("File inviati al telefono")
    + ": " + ui("Download") + ", " + ui("Documenti") + ", " + ui("Immagini") + ", " + ui("Fotocamera") + ", "
    + ui("Musica") + " o " + ui("Video") + ".",
])

POSTI = table(["Posto", "Che cosa contiene", "Si apre come"], [
    [ui("Recenti"), "I file degli ultimi 7 giorni di Fotocamera, Screenshot, Download, Documenti e WhatsApp, in gruppi "
     + ui("Oggi") + ", " + ui("Ieri") + ", " + ui("Questa settimana") + ".", "elenco"],
    [ui("Fotocamera"), "Le foto e i video scattati col telefono.", "miniature"],
    [ui("Screenshot"), "Gli screenshot fatti sul telefono.", "miniature"],
    [ui("Download"), "I file scaricati sul telefono.", "elenco"],
    [ui("WhatsApp"), "Foto, video, documenti e audio ricevuti con WhatsApp.", "elenco"],
    [ui("Documenti"), "La cartella dei documenti del telefono.", "elenco"],
    [ui("Memoria del telefono"), "Tutta la memoria condivisa del telefono, cartella per cartella.", "elenco"],
    [ui("Scheda SD"), "La scheda di memoria, se il telefono ne ha una (con più schede: " + ui("Scheda SD 1") + ", "
     + ui("Scheda SD 2") + "…).", "elenco"],
], "«TAB» — I posti di «Ricevi file…». Compaiono solo quelli che esistono sul telefono")

S4 = p("Senza cavo, il file manager del PC non vede il telefono. Per copiare foto e documenti dal telefono al PC si "
       "usa " + ui("Ricevi file…") + ".", lead=True) + steps([
    "Nel drawer premere " + ui("Ricevi file…") + ": si apre la finestra " + ui("Ricevi file dal telefono") + ", sul "
    "posto " + ui("Recenti") + ".",
    "Scegliere un posto a sinistra, poi aprire le cartelle con un clic. Il percorso in alto e la freccia "
    + ui("Cartella superiore") + " riportano indietro.",
    "Fare clic sui file da ricevere: si spuntano. Le spunte restano anche cambiando cartella, e in basso compare "
    "quanti file sono scelti e quanto pesano.",
    "Premere " + ui("Ricevi") + ". La finestra si chiude e i file arrivano nella cartella " + ui("Scaricati") + " del PC "
    "(indicata in basso: " + ui("Arrivano in Scaricati") + ").",
    "Alla fine compare " + ui("N file ricevuti in Scaricati") + " con il pulsante " + ui("Apri la cartella") + ".",
]) + POSTI + \
    table(["Comando", "Che cosa fa"], [
        [ui("Scegli tutti") + " / " + ui("Togli tutti"), "Spunta o toglie tutti i file mostrati."],
        [ui("Cerca in questa cartella"), "Mostra solo i file col nome cercato (" + ui("Cerca per nome") + ")."],
        [ui("Miniature o elenco"), "Passa dalle miniature all'elenco e viceversa."],
        [ui("Modificato ▾"), "Ordina dal più recente; un clic inverte l'ordine (" + ui("Modificato ▴") + ")."],
        ["Doppio clic su un file", "Riceve subito solo quel file."],
        [ui("Mostra altri N"), "Le cartelle si mostrano 200 file alla volta: mostra i successivi."],
        [ui("Annulla"), "Chiude la finestra senza ricevere niente."],
    ], "«TAB» — I comandi di «Ricevi file dal telefono»") + ul([
        "Si vede la memoria condivisa del telefono, la stessa che si vede col cavo: non i dati privati delle app.",
        "Le cartelle intere non si ricevono: si entra nella cartella e si usa " + ui("Scegli tutti") + ".",
        "I file nascosti (quelli col nome che comincia con il punto) non compaiono.",
        "I file arrivati conservano la data del telefono; se esiste già un file con lo stesso nome, il nuovo arriva con "
        "un numero, per esempio " + c("foto (1).jpg") + ".",
        "Con un file solo, " + ui("Apri la cartella") + " apre la cartella con il file già evidenziato.",
    ]) + tip("la cartella di arrivo si cambia una volta per tutte in " + ui("Preferenze") + " › " + ui("File ricevuti "
             "dal telefono") + " (" + rif("La pagina Preferenze") + ").", "Un'altra cartella.")

S5 = p("Invii, installazioni e ricezioni si fanno uno alla volta. Durante il trasferimento, in basso sul telefono "
       "disegnato del drawer, c'è una scheda con il nome del file, l'avanzamento e una " + ui("×") + " per annullare.",
       lead=True) + \
    table(["Scritta nella scheda", "Trasferimento"], [
        [ui("Invio al telefono · <inviati> di <totale>"), "Un file dal PC al telefono."],
        [ui("Installazione · <inviati> di <totale>") + ", poi " + ui("installazione in corso…"), "Un'app installata."],
        [ui("Dal telefono · 2 di 5 · <ricevuti> di <totale>"), "Il secondo di cinque file ricevuti dal telefono."],
    ], "«TAB» — La scheda del trasferimento") + \
    table(["Messaggio", "Significato"], [
        [ui("Aspetta la fine del trasferimento in corso"), "Un altro trasferimento è già in corso: aspettare che "
         "finisca."],
        [ui("Invio di «<file>» annullato"), "Invio annullato con la " + ui("×") + "."],
        [ui("Ricezione annullata") + ", " + ui("Ricezione annullata: N file in Scaricati"), "Ricezione annullata; i "
         "file già arrivati restano."],
        [ui("«<file>» ricevuto in Scaricati"), "Un file ricevuto."],
        [ui("N file ricevuti in Scaricati, M no: …"), "Alcuni file non sono arrivati; il motivo del primo è scritto "
         "dopo i due punti."],
        [ui("Nessun file ricevuto: …"), "Nessun file è arrivato; segue il motivo."],
    ], "«TAB» — I messaggi dei trasferimenti")

CHAPTER = ("App e file", [
    ("Installare un'app", S1),
    ("Disinstallare un'app", S2),
    ("Inviare file al telefono", S3),
    ("Ricevere file dal telefono", S4),
    ("Il trasferimento in corso", S5),
])
