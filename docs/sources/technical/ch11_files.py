from build import c, note, p, rif, table, ui, ul

S1 = p(c("azioni.rs") + " uses only the Android shell: no app on the phone, no extra permissions.", lead=True) + \
    table(["Action", "From where", "How"], [
        ["Installing an app", ui("Install app…") + ", or an " + c(".apk") + " dragged onto the drawn phone "
         "(with confirmation)", "Copy with " + c("sync:") + " to " + c("/data/local/tmp") + ", " + c("pm install -r")
         + "; the reasons for a rejection are translated into plain words (" + c("azioni::spiega_installazione") + ")"],
        ["Uninstalling", ui("Uninstall…") + " in the app menu (user apps only, "
         + c("pm list packages -3") + "), with confirmation", c("pm uninstall") + "; the app leaves the favorites and its "
         "window closes"],
        ["Sending files", ui("Send files…") + " (several files too), or files dragged onto the drawn phone",
         "Copy to " + c("/sdcard/<cartella>") + " (the folder from the preferences, Download if not chosen) with a "
         "free name, without overwriting, and notification to the media scanner: the file appears in Gallery and Files"],
    ], "«TAB» — Installing, uninstalling, sending") + ul([
        "One transfer at a time: a second one is rejected with the alert “Wait for the current transfer to finish”. The transfer card shows "
        "the progress and is canceled with ×.",
        "After an installation or uninstallation, the app list is reread.",
    ])

S2 = p(c("ricevi.rs") + " is the “Receive files…” window (" + c("adw::Dialog") + "): you browse the phone and "
       "choose the files to copy to the PC (SPECIFICATION §11.1, mockup " + c("receive-files.html") + ").", lead=True) + \
    table(["Place", "Phone folder", "Opens as"], [
        ["Recent", "the last 7 days of Camera, Screenshot, Download, Documents and WhatsApp (images, videos, "
         "documents, audio), in groups Today, Yesterday, This week", "list"],
        ["Camera", c("/sdcard/DCIM/Camera"), "thumbnails"],
        ["Screenshot", c("/sdcard/DCIM/Screenshots") + " or " + c("/sdcard/Pictures/Screenshots"), "thumbnails"],
        ["Download", c("/sdcard/Download"), "list"],
        ["WhatsApp", c("/sdcard/Android/media/com.whatsapp/WhatsApp/Media") + " (Android 11+) or "
         + c("/sdcard/WhatsApp/Media"), "list"],
        ["Documents", c("/sdcard/Documents"), "list"],
        ["Phone storage", c("/sdcard"), "list"],
        ["SD card", "the cards found in " + c("/storage") + " (more than one: “SD card 1”, “SD card 2”…)", "list"],
    ], "«TAB» — The places of “Receive files…”. Recent and Phone storage are always there; the "
       "other places appear only if they exist (" + c("STA2") + ", the first candidate folder that exists wins); SD cards are "
       "found by listing " + c("/storage") + " (minus " + c("emulated") + " and " + c("self") + ")") + ul([
        "Clickable path and “up” button, up to Phone storage or SD card; search by name; sort by "
        "date, newest or oldest first; thumbnails or list.",
        "Folders are read with " + c("sync.rs") + " (" + rif("Copying files: sync:") + "): they are read in full and "
        "shown 200 at a time (“Show N more”, Mostra altri N); hidden files (starting with a dot) do not appear.",
        "Checkmarks persist when changing folder; “Select all” / “Deselect all”; a double click receives the file immediately.",
        "The thumbnails are made by the helper (" + c("miniature <lato> <percorsi in base64>") + ", " + c("Miniature.java")
        + ") in groups of 40, at 192 pixels: each launch costs about one second. Photos are downscaled by "
        + c("BitmapFactory") + " and straightened using EXIF; videos give a frame with "
        + c("MediaMetadataRetriever") + " through a " + c("MediaDataSource") + ".",
    ]) + \
    p("The copy is done by the drawer, with the same transfer card as sending: each file arrives with "
      + c("sync::ricevi") + " in the folder of " + c("Preferenze::cartella_ricevuti") + " (the Downloads folder "
      "if not chosen), with a free name and the phone's modification date. At the end an alert says how many files arrived, with "
      + ui("Open folder") + " (with a single file, the folder opens with the file already highlighted).") + \
    note(c("PHONESTRA_PROVA_RICEVI=<posto>") + " opens “Receive files…” by itself on connection, at the given place "
         "(for example " + c("Fotocamera") + "), for interface tests; " + c("phonestra-prova file")
         + " tests listing, receiving and thumbnails from the command line.", "For tests.")

CHAPTER = ("Apps and files", [
    ("Installing and sending", S1),
    ("Receiving files from the phone", S2),
])
