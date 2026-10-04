from build import c, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("Phonestra installs on the phone apps downloaded as " + c(".apk") + " files, with no cable and without going "
       "through the phone.", lead=True) + steps([
    "In the drawer press " + ui("Install app…") + " (Installa app…) and choose the file (the " + ui("Choose the app to install") + " (Scegli l'app da installare) window shows only " + ui("Android apps (.apk)") + " (App Android (.apk)) files). Or drag the " + c(".apk")
    + " file onto the phone drawn on the right of the drawer.",
    "Phonestra asks for confirmation: " + ui("Install “<file>”?") + " (Installare «&lt;file&gt;»?). Press " + ui("Install") + " (Installa).",
    "The transfer card, on the drawn phone, shows the progress, then " + ui("installing…") + " (installazione in corso…).",
    "At the end " + ui("“<file>” installed") + " («&lt;file&gt;» installata) appears and the app joins the drawer's list.",
]) + warn("when installing from the PC, Android does not ask the usual confirmation for “unknown sources”. Install only apps "
          "you trust: the confirmation window reminds you of this.", "Trusted apps only.") + \
    table(["Reason in the message", "What to do"], [
        ["“this app is already on the phone, signed by someone else: uninstall it first” (sul telefono c'è già quest'app firmata da qualcun altro: disinstallala prima)", "Uninstall the app already "
         "there, then try again."],
        ["“a newer version is already on the phone” (sul telefono c'è già una versione più recente)", "Nothing: the phone already has a newer release."],
        ["“the app is for a version of Android that is too old, and Android refuses it” (l'app è per una versione di Android troppo vecchia e Android la rifiuta)", "Look for a newer release "
         "of the app. Phonestra does not force the installation."],
        ["“the app needs a newer version of Android than the phone's” (l'app richiede una versione di Android più recente di quella del telefono)", "The app does not run on this phone."],
        ["“there is not enough space on the phone” (sul telefono non c'è abbastanza spazio)", "Free up space on the phone."],
        ["“the phone does not allow installing from the PC (on Xiaomi phones: turn on “Install via USB” in Developer "
         "options)” (il telefono non permette di installare dal PC (sugli Xiaomi: attiva «Installa tramite USB» nelle "
         "Opzioni sviluppatore))", "Follow the hint, then try again."],
        ["“Play Protect blocked the app” (Play Protect ha bloccato l'app)", "Google's security check stopped the app: better not to "
         "install it."],
        ["“the file is not a valid Android app” (il file non è un'app Android valida)", "The file is corrupted or is not an app: download it again."],
        ["“the app is not made for this phone's processor” (l'app non è fatta per il processore di questo telefono)", "Look for the release of the app for this phone."],
    ], "«TAB» — Why an installation can fail") + \
    note("packages split into several files (" + c(".apks") + ", " + c(".xapk") + ", " + c(".apkm") + ") cannot be "
         "installed: Phonestra accepts only " + c(".apk") + " files.", "Only .apk.")

S2 = steps([
    "In the drawer, right-click the app and choose " + ui("Uninstall…") + " (Disinstalla…).",
    "Phonestra asks for confirmation: " + ui("Uninstall <app>?") + " (Disinstallare &lt;app&gt;?) with " + ui("The app and its data will be removed from “<phone>”.") + " (L'app e i suoi dati verranno tolti da «&lt;telefono&gt;».) Press " + ui("Uninstall") + " (Disinstalla).",
    "If the app was open, its window closes. At the end " + ui("<app> uninstalled") + " (&lt;app&gt; disinstallata) appears: the app leaves "
    "the list and the favorites.",
]) + note("system apps (the ones the phone already came with) cannot be uninstalled: the menu item is grayed out, with "
          + ui("System app: cannot be uninstalled") + " (App di sistema: non si può disinstallare).", "System apps.")

S3 = p("Files sent from the PC arrive in a folder on the phone, " + ui("Download") + " unless you chose "
       "another, and show up right away in the phone's Gallery and file app.", lead=True) + steps([
    "In the drawer press " + ui("Send files…") + " (Invia file…) and choose one or more files (" + ui("Choose the files to send") + ", Scegli i file da inviare). "
    "Or drag them from the file manager onto the drawn phone: " + ui("Drop to send to the phone") + " (Rilascia per inviare al telefono) appears.",
    "The transfer card shows the file's name and " + ui("Sending to the phone · <sent> of <total>") + " (Invio al telefono · &lt;inviati&gt; di &lt;totale&gt;).",
    "At the end " + ui("“<file>” is in Download on the phone") + " («&lt;file&gt;» è in Download sul telefono) appears (or in the chosen folder).",
]) + ul([
    "A file with the same name as one already there does not replace it: it arrives with a number, for example "
    + c("foto (1).jpg") + ".",
    "An " + c(".apk") + " file dragged onto the phone is not copied but installed, after confirmation.",
    "The destination folder on the phone is chosen in " + ui("Preferences") + " (Preferenze) › " + ui("Files sent to the phone") + " (File inviati al telefono): " + ui("Download") + ", " + ui("Documents") + " (Documenti), " + ui("Pictures") + " (Immagini), " + ui("Camera") + " (Fotocamera), "
    + ui("Music") + " (Musica) or " + ui("Movies") + " (Video).",
])

POSTI = table(["Place", "What it contains", "Shown as"], [
    [ui("Recent") + " (Recenti)", "Files from the last 7 days in Camera, Screenshots, Download, Documents and WhatsApp, in groups "
     + ui("Today") + ", " + ui("Yesterday") + ", " + ui("This week") + " (Oggi, Ieri, Questa settimana).", "list"],
    [ui("Camera") + " (Fotocamera)", "Photos and videos taken with the phone.", "thumbnails"],
    [ui("Screenshot"), "Screenshots taken on the phone.", "thumbnails"],
    [ui("Download"), "Files downloaded on the phone.", "list"],
    [ui("WhatsApp"), "Photos, videos, documents and audio received with WhatsApp.", "list"],
    [ui("Documents") + " (Documenti)", "The phone's documents folder.", "list"],
    [ui("Phone storage") + " (Memoria del telefono)", "All of the phone's shared storage, folder by folder.", "list"],
    [ui("SD card") + " (Scheda SD)", "The memory card, if the phone has one (with several cards: " + ui("SD card 1") + ", "
     + ui("SD card 2") + "…).", "list"],
], "«TAB» — The places in “Receive files…”. Only those that exist on the phone appear")

S4 = p("Without a cable, the PC's file manager does not see the phone. To copy photos and documents from the phone to the PC, "
       "use " + ui("Receive files…") + " (Ricevi file…).", lead=True) + steps([
    "In the drawer press " + ui("Receive files…") + ": the " + ui("Receive files from the phone") + " (Ricevi file dal telefono) window opens, on the "
    + ui("Recent") + " (Recenti) place.",
    "Choose a place on the left, then open folders with a click. The path at the top and the "
    + ui("Parent folder") + " (Cartella superiore) arrow take you back.",
    "Click the files to receive: they get checked. Checks stay even when you change folder, and at the bottom you see "
    "how many files are selected and how big they are.",
    "Press " + ui("Receive") + " (Ricevi). The window closes and the files arrive in the PC's " + ui("Downloads") + " (Scaricati) folder "
    "(shown at the bottom: " + ui("They arrive in Downloads") + ", Arrivano in Scaricati).",
    "At the end " + ui("N files received in Downloads") + " (N file ricevuti in Scaricati) appears, with the " + ui("Open folder") + " (Apri la cartella) button.",
]) + POSTI + \
    table(["Command", "What it does"], [
        [ui("Select all") + " / " + ui("Deselect all") + " (Scegli tutti / Togli tutti)", "Checks or unchecks all the files shown."],
        [ui("Search in this folder") + " (Cerca in questa cartella)", "Shows only the files whose name matches the search (" + ui("Search by name") + ", Cerca per nome)."],
        [ui("Thumbnails or list") + " (Miniature o elenco)", "Switches between thumbnails and list."],
        [ui("Modified ▾") + " (Modificato ▾)", "Sorts newest first; a click reverses the order (" + ui("Modified ▴") + ", Modificato ▴)."],
        ["Double-click a file", "Receives just that file right away."],
        [ui("Show N more") + " (Mostra altri N)", "Folders are shown 200 files at a time: shows the next ones."],
        [ui("Cancel") + " (Annulla)", "Closes the window without receiving anything."],
    ], "«TAB» — The commands of “Receive files from the phone”") + ul([
        "You see the phone's shared storage, the same you see with a cable: not the apps' private data.",
        "Whole folders cannot be received: open the folder and use " + ui("Select all") + ".",
        "Hidden files (those whose name starts with a dot) do not appear.",
        "Received files keep the phone's date; if a file with the same name already exists, the new one arrives with "
        "a number, for example " + c("foto (1).jpg") + ".",
        "With a single file, " + ui("Open folder") + " (Apri la cartella) opens the folder with the file already highlighted.",
    ]) + tip("the destination folder can be changed once and for all in " + ui("Preferences") + " › " + ui("Files received "
             "from the phone") + " (Preferenze › File ricevuti dal telefono; " + rif("The “Preferences” page") + ").", "Another folder.")

S5 = p("Sends, installations and receptions happen one at a time. During the transfer, at the bottom of the drawer's "
       "drawn phone, there is a card with the file's name, the progress and a " + ui("×") + " to cancel.",
       lead=True) + \
    table(["Text on the card", "Transfer"], [
        [ui("Sending to the phone · <sent> of <total>") + " (Invio al telefono · &lt;inviati&gt; di &lt;totale&gt;)", "A file from the PC to the phone."],
        [ui("Installing · <sent> of <total>") + " (Installazione · &lt;inviati&gt; di &lt;totale&gt;), then " + ui("installing…") + " (installazione in corso…)", "An app being installed."],
        [ui("From the phone · 2 of 5 · <received> of <total>") + " (Dal telefono · 2 di 5 · &lt;ricevuti&gt; di &lt;totale&gt;)", "The second of five files received from the phone."],
    ], "«TAB» — The transfer card") + \
    table(["Message", "Meaning"], [
        [ui("Wait for the current transfer to finish") + " (Aspetta la fine del trasferimento in corso)", "Another transfer is already in progress: wait for it to "
         "finish."],
        [ui("Sending of “<file>” canceled") + " (Invio di «&lt;file&gt;» annullato)", "Sending canceled with the " + ui("×") + "."],
        [ui("Reception canceled") + " (Ricezione annullata), " + ui("Reception canceled: N files in Downloads") + " (Ricezione annullata: N file in Scaricati)", "Reception canceled; the "
         "files that already arrived stay."],
        [ui("“<file>” received in Downloads") + " («&lt;file&gt;» ricevuto in Scaricati)", "One file received."],
        [ui("N files received in Downloads, M failed: …") + " (N file ricevuti in Scaricati, M no: …)", "Some files did not arrive; the reason for the first one is written "
         "after the colon."],
        [ui("No file received: …") + " (Nessun file ricevuto: …)", "No file arrived; the reason follows."],
    ], "«TAB» — Transfer messages")

CHAPTER = ("Apps and files", [
    ("Installing an app", S1),
    ("Uninstalling an app", S2),
    ("Sending files to the phone", S3),
    ("Receiving files from the phone", S4),
    ("The transfer in progress", S5),
])
