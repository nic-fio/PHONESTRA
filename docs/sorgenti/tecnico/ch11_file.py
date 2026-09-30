from build import c, note, p, rif, table, ui, ul

S1 = p(c("azioni.rs") + " usa solo la shell di Android: niente app sul telefono, niente permessi in più.", lead=True) + \
    table(["Azione", "Da dove", "Come"], [
        ["Installare un'app", ui("Installa app…") + ", o un " + c(".apk") + " trascinato sul telefono disegnato "
         "(con conferma)", "Copia con " + c("sync:") + " in " + c("/data/local/tmp") + ", " + c("pm install -r")
         + "; i motivi di un rifiuto sono tradotti in parole semplici (" + c("azioni::spiega_installazione") + ")"],
        ["Disinstallare", ui("Disinstalla…") + " nel menu dell'app (solo le app dell'utente, "
         + c("pm list packages -3") + "), con conferma", c("pm uninstall") + "; l'app esce dai preferiti e la sua "
         "finestra si chiude"],
        ["Inviare file", ui("Invia file…") + " (anche più file), o file trascinati sul telefono disegnato",
         "Copia in " + c("/sdcard/<cartella>") + " (la cartella delle preferenze, Download se non scelta) con un nome "
         "libero, senza sovrascrivere, e annuncio al media scanner: il file compare in Galleria e File"],
    ], "«TAB» — Installare, disinstallare, inviare") + ul([
        "Un trasferimento alla volta: un secondo viene rifiutato con l'avviso «Aspetta la fine del trasferimento in corso». La scheda del trasferimento mostra "
        "l'avanzamento e si annulla con ×.",
        "Dopo un'installazione o una disinstallazione l'elenco delle app si rilegge.",
    ])

S2 = p(c("ricevi.rs") + " è la finestra «Ricevi file…» (" + c("adw::Dialog") + "): si naviga nel telefono e si "
       "scelgono i file da copiare sul PC (SPECIFICHE §11.1, mockup " + c("ricevi-file.html") + ").", lead=True) + \
    table(["Posto", "Cartella del telefono", "Si apre a"], [
        ["Recenti", "gli ultimi 7 giorni di Fotocamera, Screenshot, Download, Documenti e WhatsApp (immagini, video, "
         "documenti, audio), a gruppi Oggi, Ieri, Questa settimana", "elenco"],
        ["Fotocamera", c("/sdcard/DCIM/Camera"), "miniature"],
        ["Screenshot", c("/sdcard/DCIM/Screenshots") + " o " + c("/sdcard/Pictures/Screenshots"), "miniature"],
        ["Download", c("/sdcard/Download"), "elenco"],
        ["WhatsApp", c("/sdcard/Android/media/com.whatsapp/WhatsApp/Media") + " (Android 11+) o "
         + c("/sdcard/WhatsApp/Media"), "elenco"],
        ["Documenti", c("/sdcard/Documents"), "elenco"],
        ["Memoria del telefono", c("/sdcard"), "elenco"],
        ["Scheda SD", "le schede trovate in " + c("/storage") + " (più d'una: «Scheda SD 1», «Scheda SD 2»…)", "elenco"],
    ], "«TAB» — I posti di «Ricevi file…». Recenti e Memoria del telefono ci sono sempre; gli altri posti "
       "compaiono solo se esistono (" + c("STA2") + ", vale la prima cartella candidata che c'è); le schede SD si "
       "trovano elencando " + c("/storage") + " (tolti " + c("emulated") + " e " + c("self") + ")") + ul([
        "Percorso cliccabile e pulsante «su», fino a Memoria del telefono o Scheda SD; ricerca per nome; ordine per "
        "data, dal più nuovo o dal più vecchio; miniature o elenco.",
        "Le cartelle si leggono con " + c("sync.rs") + " (" + rif("Copiare file: sync:") + "): si leggono intere e se ne "
        "mostrano 200 per volta («Mostra altri N»); i file nascosti (che iniziano col punto) non compaiono.",
        "Le spunte restano cambiando cartella; «Scegli tutti» / «Togli tutti»; il doppio clic riceve subito il file.",
        "Le miniature le fa l'aiutante (" + c("miniature <lato> <percorsi in base64>") + ", " + c("Miniature.java")
        + ") a gruppi di 40, di 192 pixel: ogni avvio costa circa un secondo. Le foto sono ridotte da "
        + c("BitmapFactory") + " e raddrizzate con l'EXIF; i video danno un fotogramma con "
        + c("MediaMetadataRetriever") + " attraverso una " + c("MediaDataSource") + ".",
    ]) + \
    p("La copia la fa il drawer, con la stessa scheda di trasferimento dell'invio: ogni file arriva con "
      + c("sync::ricevi") + " nella cartella di " + c("Preferenze::cartella_ricevuti") + " (Scaricati se non scelta), "
      "con un nome libero e la data di modifica del telefono. Alla fine un avviso dice quanti file sono arrivati, con "
      + ui("Apri la cartella") + " (con un file solo, la cartella si apre col file già evidenziato).") + \
    note(c("PHONESTRA_PROVA_RICEVI=<posto>") + " apre da sola «Ricevi file…» al collegamento, sul posto indicato "
         "(per esempio " + c("Fotocamera") + "), per le prove dell'interfaccia; " + c("phonestra-prova file")
         + " prova elenco, ricezione e miniature da riga di comando.", "Per le prove.")

CHAPTER = ("App e file", [
    ("Installare e inviare", S1),
    ("Ricevere file dal telefono", S2),
])
