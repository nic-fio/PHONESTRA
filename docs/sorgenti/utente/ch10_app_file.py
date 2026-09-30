from build import c, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("Phonestra installs on the phone apps downloaded as " + c(".apk") + " files, with no cable and without going "
       "through the phone.", lead=True) + steps([
    "In the drawer press " + ui("Installa app…") + " (Install app…) and choose the file (the " + ui("Scegli l'app da "
    "installare") + " window shows only " + ui("App Android (.apk)") + " files). Or drag the " + c(".apk")
    + " file onto the phone drawn on the right of the drawer.",
    "Phonestra asks for confirmation: " + ui("Installare «<file>»?") + " (Install “&lt;file&gt;”?). Press " + ui("Installa") + " (Install).",
    "The transfer card, on the drawn phone, shows the progress, then " + ui("installazione in corso…")
    + " (installing…).",
    "At the end " + ui("«<file>» installata") + " (“&lt;file&gt;” installed) appears and the app joins the drawer's list.",
]) + warn("when installing from the PC, Android does not ask the usual confirmation for “unknown sources”. Install only apps "
          "you trust: the confirmation window reminds you of this.", "Trusted apps only.") + \
    table(["Reason in the message", "What to do"], [
        ["“sul telefono c'è già quest'app firmata da qualcun altro: disinstallala prima”", "Uninstall the app already "
         "there, then try again."],
        ["“sul telefono c'è già una versione più recente”", "Nothing: the phone already has a newer release."],
        ["“l'app è per una versione di Android troppo vecchia e Android la rifiuta”", "Look for a newer release "
         "of the app. Phonestra does not force the installation."],
        ["“l'app richiede una versione di Android più recente di quella del telefono”", "The app does not run on this phone."],
        ["“sul telefono non c'è abbastanza spazio”", "Free up space on the phone."],
        ["“il telefono non permette di installare dal PC (sugli Xiaomi: attiva «Installa tramite USB» nelle Opzioni "
         "sviluppatore)”", "Follow the hint, then try again."],
        ["“Play Protect ha bloccato l'app”", "Google's security check stopped the app: better not to "
         "install it."],
        ["“il file non è un'app Android valida”", "The file is corrupted or is not an app: download it again."],
        ["“l'app non è fatta per il processore di questo telefono”", "Look for the release of the app for this phone."],
    ], "«TAB» — Why an installation can fail") + \
    note("packages split into several files (" + c(".apks") + ", " + c(".xapk") + ", " + c(".apkm") + ") cannot be "
         "installed: Phonestra accepts only " + c(".apk") + " files.", "Only .apk.")

S2 = steps([
    "In the drawer, right-click the app and choose " + ui("Disinstalla…") + " (Uninstall…).",
    "Phonestra asks for confirmation: " + ui("Disinstallare <app>?") + " with " + ui("L'app e i suoi dati verranno tolti da "
    "«<telefono>».") + " (The app and its data will be removed from “&lt;phone&gt;”.) Press " + ui("Disinstalla") + ".",
    "If the app was open, its window closes. At the end " + ui("<app> disinstallata") + " (&lt;app&gt; uninstalled) appears: the app leaves "
    "the list and the favorites.",
]) + note("system apps (the ones the phone already came with) cannot be uninstalled: the menu item is grayed out, with "
          + ui("App di sistema: non si può disinstallare") + " (System app: cannot be uninstalled).", "System apps.")

S3 = p("Files sent from the PC arrive in a folder on the phone, " + ui("Download") + " unless you chose "
       "another, and show up right away in the phone's Gallery and file app.", lead=True) + steps([
    "In the drawer press " + ui("Invia file…") + " (Send files…) and choose one or more files (" + ui("Scegli i file da inviare") + "). "
    "Or drag them from the file manager onto the drawn phone: " + ui("Rilascia per inviare al telefono") + " (Drop to send to the phone) appears.",
    "The transfer card shows the file's name and " + ui("Invio al telefono · <inviati> di <totale>") + " (Sending to the phone · &lt;sent&gt; of &lt;total&gt;).",
    "At the end " + ui("«<file>» è in Download sul telefono") + " (“&lt;file&gt;” is in Download on the phone) appears (or in the chosen folder).",
]) + ul([
    "A file with the same name as one already there does not replace it: it arrives with a number, for example "
    + c("foto (1).jpg") + ".",
    "An " + c(".apk") + " file dragged onto the phone is not copied but installed, after confirmation.",
    "The destination folder on the phone is chosen in " + ui("Preferenze") + " › " + ui("File inviati al telefono")
    + " (Files sent to the phone): " + ui("Download") + ", " + ui("Documenti") + ", " + ui("Immagini") + ", " + ui("Fotocamera") + ", "
    + ui("Musica") + " or " + ui("Video") + ".",
])

POSTI = table(["Place", "What it contains", "Shown as"], [
    [ui("Recenti"), "Files from the last 7 days in Camera, Screenshots, Download, Documents and WhatsApp, in groups "
     + ui("Oggi") + ", " + ui("Ieri") + ", " + ui("Questa settimana") + " (Today, Yesterday, This week).", "list"],
    [ui("Fotocamera"), "Photos and videos taken with the phone.", "thumbnails"],
    [ui("Screenshot"), "Screenshots taken on the phone.", "thumbnails"],
    [ui("Download"), "Files downloaded on the phone.", "list"],
    [ui("WhatsApp"), "Photos, videos, documents and audio received with WhatsApp.", "list"],
    [ui("Documenti"), "The phone's documents folder.", "list"],
    [ui("Memoria del telefono"), "All of the phone's shared storage, folder by folder.", "list"],
    [ui("Scheda SD"), "The memory card, if the phone has one (with several cards: " + ui("Scheda SD 1") + ", "
     + ui("Scheda SD 2") + "…).", "list"],
], "«TAB» — The places in “Ricevi file…”. Only those that exist on the phone appear")

S4 = p("Without a cable, the PC's file manager does not see the phone. To copy photos and documents from the phone to the PC, "
       "use " + ui("Ricevi file…") + " (Receive files…).", lead=True) + steps([
    "In the drawer press " + ui("Ricevi file…") + ": the " + ui("Ricevi file dal telefono") + " (Receive files from the phone) window opens, on the "
    + ui("Recenti") + " (Recent) place.",
    "Choose a place on the left, then open folders with a click. The path at the top and the "
    + ui("Cartella superiore") + " (Parent folder) arrow take you back.",
    "Click the files to receive: they get checked. Checks stay even when you change folder, and at the bottom you see "
    "how many files are selected and how big they are.",
    "Press " + ui("Ricevi") + " (Receive). The window closes and the files arrive in the PC's " + ui("Scaricati") + " (Downloads) folder "
    "(shown at the bottom: " + ui("Arrivano in Scaricati") + ").",
    "At the end " + ui("N file ricevuti in Scaricati") + " (N files received in Scaricati) appears, with the " + ui("Apri la cartella") + " (Open folder) button.",
]) + POSTI + \
    table(["Command", "What it does"], [
        [ui("Scegli tutti") + " / " + ui("Togli tutti"), "Checks or unchecks all the files shown."],
        [ui("Cerca in questa cartella"), "Shows only the files whose name matches the search (" + ui("Cerca per nome") + ")."],
        [ui("Miniature o elenco"), "Switches between thumbnails and list."],
        [ui("Modificato ▾"), "Sorts newest first; a click reverses the order (" + ui("Modificato ▴") + ")."],
        ["Double-click a file", "Receives just that file right away."],
        [ui("Mostra altri N"), "Folders are shown 200 files at a time: shows the next ones."],
        [ui("Annulla"), "Closes the window without receiving anything."],
    ], "«TAB» — The commands of “Ricevi file dal telefono”") + ul([
        "You see the phone's shared storage, the same you see with a cable: not the apps' private data.",
        "Whole folders cannot be received: open the folder and use " + ui("Scegli tutti") + " (Select all).",
        "Hidden files (those whose name starts with a dot) do not appear.",
        "Received files keep the phone's date; if a file with the same name already exists, the new one arrives with "
        "a number, for example " + c("foto (1).jpg") + ".",
        "With a single file, " + ui("Apri la cartella") + " opens the folder with the file already highlighted.",
    ]) + tip("the destination folder can be changed once and for all in " + ui("Preferenze") + " › " + ui("File ricevuti "
             "dal telefono") + " (" + rif("The “Preferenze” page") + ").", "Another folder.")

S5 = p("Sends, installations and receptions happen one at a time. During the transfer, at the bottom of the drawer's "
       "drawn phone, there is a card with the file's name, the progress and a " + ui("×") + " to cancel.",
       lead=True) + \
    table(["Text on the card", "Transfer"], [
        [ui("Invio al telefono · <inviati> di <totale>"), "A file from the PC to the phone."],
        [ui("Installazione · <inviati> di <totale>") + ", then " + ui("installazione in corso…"), "An app being installed."],
        [ui("Dal telefono · 2 di 5 · <ricevuti> di <totale>"), "The second of five files received from the phone."],
    ], "«TAB» — The transfer card") + \
    table(["Message", "Meaning"], [
        [ui("Aspetta la fine del trasferimento in corso"), "Another transfer is already in progress: wait for it to "
         "finish."],
        [ui("Invio di «<file>» annullato"), "Sending canceled with the " + ui("×") + "."],
        [ui("Ricezione annullata") + ", " + ui("Ricezione annullata: N file in Scaricati"), "Reception canceled; the "
         "files that already arrived stay."],
        [ui("«<file>» ricevuto in Scaricati"), "One file received."],
        [ui("N file ricevuti in Scaricati, M no: …"), "Some files did not arrive; the reason for the first one is written "
         "after the colon."],
        [ui("Nessun file ricevuto: …"), "No file arrived; the reason follows."],
    ], "«TAB» — Transfer messages")

CHAPTER = ("Apps and files", [
    ("Installing an app", S1),
    ("Uninstalling an app", S2),
    ("Sending files to the phone", S3),
    ("Receiving files from the phone", S4),
    ("The transfer in progress", S5),
])
