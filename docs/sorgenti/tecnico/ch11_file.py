from build import c, note, p, rif, table, ui, ul

S1 = p(c("azioni.rs") + " uses only the Android shell: no app on the phone, no extra permissions.", lead=True) + \
    table(["Action", "From where", "How"], [
        ["Installing an app", ui("Installa app…") + " (Install app…), or an " + c(".apk") + " dragged onto the drawn phone "
         "(with confirmation)", "Copy with " + c("sync:") + " to " + c("/data/local/tmp") + ", " + c("pm install -r")
         + "; the reasons for a rejection are translated into plain words (" + c("azioni::spiega_installazione") + ")"],
        ["Uninstalling", ui("Disinstalla…") + " (Uninstall…) in the app menu (user apps only, "
         + c("pm list packages -3") + "), with confirmation", c("pm uninstall") + "; the app leaves the favorites and its "
         "window closes"],
        ["Sending files", ui("Invia file…") + " (Send files…; several files too), or files dragged onto the drawn phone",
         "Copy to " + c("/sdcard/<cartella>") + " (the folder from the preferences, Download if not chosen) with a "
         "free name, without overwriting, and notification to the media scanner: the file appears in Gallery and Files"],
    ], "«TAB» — Installing, uninstalling, sending") + ul([
        "One transfer at a time: a second one is rejected with the alert “Aspetta la fine del trasferimento in corso”. The transfer card shows "
        "the progress and is canceled with ×.",
        "After an installation or uninstallation, the app list is reread.",
    ])

S2 = p(c("ricevi.rs") + " is the “Ricevi file…” (Receive files…) window (" + c("adw::Dialog") + "): you browse the phone and "
       "choose the files to copy to the PC (SPECIFICHE §11.1, mockup " + c("ricevi-file.html") + ").", lead=True) + \
    table(["Place", "Phone folder", "Opens as"], [
        ["Recenti", "the last 7 days of Fotocamera, Screenshot, Download, Documenti and WhatsApp (images, videos, "
         "documents, audio), in groups Oggi, Ieri, Questa settimana (Today, Yesterday, This week)", "list"],
        ["Fotocamera", c("/sdcard/DCIM/Camera"), "thumbnails"],
        ["Screenshot", c("/sdcard/DCIM/Screenshots") + " or " + c("/sdcard/Pictures/Screenshots"), "thumbnails"],
        ["Download", c("/sdcard/Download"), "list"],
        ["WhatsApp", c("/sdcard/Android/media/com.whatsapp/WhatsApp/Media") + " (Android 11+) or "
         + c("/sdcard/WhatsApp/Media"), "list"],
        ["Documenti", c("/sdcard/Documents"), "list"],
        ["Memoria del telefono", c("/sdcard"), "list"],
        ["Scheda SD", "the cards found in " + c("/storage") + " (more than one: “Scheda SD 1”, “Scheda SD 2”…)", "list"],
    ], "«TAB» — The places of “Ricevi file…”. Recenti (Recent) and Memoria del telefono (Phone storage) are always there; the "
       "other places appear only if they exist (" + c("STA2") + ", the first candidate folder that exists wins); SD cards are "
       "found by listing " + c("/storage") + " (minus " + c("emulated") + " and " + c("self") + ")") + ul([
        "Clickable path and “up” button, up to Memoria del telefono or Scheda SD (SD card); search by name; sort by "
        "date, newest or oldest first; thumbnails or list.",
        "Folders are read with " + c("sync.rs") + " (" + rif("Copying files: sync:") + "): they are read in full and "
        "shown 200 at a time (“Mostra altri N”); hidden files (starting with a dot) do not appear.",
        "Checkmarks persist when changing folder; “Scegli tutti” / “Togli tutti” (Select all / Deselect all); a double click receives the file immediately.",
        "The thumbnails are made by the helper (" + c("miniature <lato> <percorsi in base64>") + ", " + c("Miniature.java")
        + ") in groups of 40, at 192 pixels: each launch costs about one second. Photos are downscaled by "
        + c("BitmapFactory") + " and straightened using EXIF; videos give a frame with "
        + c("MediaMetadataRetriever") + " through a " + c("MediaDataSource") + ".",
    ]) + \
    p("The copy is done by the drawer, with the same transfer card as sending: each file arrives with "
      + c("sync::ricevi") + " in the folder of " + c("Preferenze::cartella_ricevuti") + " (the Scaricati (Downloads) folder "
      "if not chosen), with a free name and the phone's modification date. At the end an alert says how many files arrived, with "
      + ui("Apri la cartella") + " (Open folder; with a single file, the folder opens with the file already highlighted).") + \
    note(c("PHONESTRA_PROVA_RICEVI=<posto>") + " opens “Ricevi file…” by itself on connection, at the given place "
         "(for example " + c("Fotocamera") + "), for interface tests; " + c("phonestra-prova file")
         + " tests listing, receiving and thumbnails from the command line.", "For tests.")

CHAPTER = ("Apps and files", [
    ("Installing and sending", S1),
    ("Receiving files from the phone", S2),
])
