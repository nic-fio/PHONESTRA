from build import c, code, key, note, p, rif, steps, table, ui, ul

S1 = p("The interface uses GTK4 and libadwaita, with a style faithful to libadwaita (GNOME title bar, boxed lists, pill buttons). "
       "Phonestra follows the system's light or dark theme: " + c("segui_tema") + " applies to all windows, the "
       + c("SCURO") + " palette of " + c("cassetto.rs") + " to the drawer, “Aggiungi un telefono” (Add a phone) and the wizard (the "
       "app windows have rules of their own); the mockups are "
       "light only. No " + c(".desktop") + " file is installed. The design proposals every screen comes from "
       "are in " + c("mockup/") + "; the rules in " + c("memoria/interfaccia.md") + ".", lead=True) + \
    table(["Window", "File", "What it is", "More detail"], [
        ["Drawer", c("cassetto.rs"), "The main window: apps, notifications, phones, tools, the phone's "
         "screen", rif("The drawer")],
        ["App window", c("finestra.rs"), "One per app, with its own virtual display",
         rif("App windows")],
        ["“Ricevi file…” (Receive files…)", c("ricevi.rs"), "An " + c("adw::Dialog") + " for choosing files on the phone",
         rif("Receiving files from the phone")],
        ["“Aggiungi un telefono” (Add a phone)", c("prepara.rs"), "The first connection without a cable", rif("The first connection")],
        ["Cable wizard", c("procedura.rs"), "The USB cable fallback", rif("The first connection")],
        ["System alerts", c("avvisi.rs"), "The phone's notifications on the desktop, via D-Bus",
         rif("Notifications and alerts")],
    ], "«TAB» — Phonestra's windows and their files")

S1B = p(c("cassetto.rs") + " is the main window, the drawer. It subscribes to four " + c("watch") + " of the "
        + c("Collegamento") + ": " + c("stato()") + ", " + c("guasto()") + ", " + c("info()") + " (battery and network) and "
        + c("notifiche()") + "; it reaches the component through " + c("finestra::vista") + ".", lead=True) + \
    table(["Part", "What it contains"], [
        ["Title bar", "The Phonestra symbol, the name and the phone pill with the connection state. "
         "When the component has failed, the pill turns red: “Phonestra non parte sul telefono” (Phonestra won't start on the phone)."],
        ["Pill menu", "Header with name, “model · Android N”, Wi-Fi and battery; " + ui("Riconnetti")
         + " (Reconnect), " + ui("Rinomina…") + " (Rename…), " + ui("Spegni il Debug wireless alla chiusura")
         + " (Turn off Wireless debugging on close; disabled, “In arrivo”), "
         + ui("Dimentica questo telefono…") + " (Forget this phone…)."],
        ["Sidebar", ui("App") + " (Apps) and " + ui("Notifiche") + " (Notifications, with the counter); section "
         + ui("Telefoni") + " (Phones) with the active phone, the other configured phones (“non attivo”, not active) and "
         + ui("Aggiungi telefono") + " (Add phone); section " + ui("Strumenti") + " (Tools) with " + ui("Installa app…")
         + " (Install app…), " + ui("Invia file…") + " (Send files…), " + ui("Ricevi file…") + " (Receive files…); "
         "at the bottom " + ui("Preferenze") + " (Preferences) and " + ui("Informazioni") + " (About; the libadwaita "
         "“About” window with the logo)."],
        ["Apps page", "Search, Favorites and All apps. Typing in the drawer searches; " + key("Enter")
         + " opens the first app found."],
        ["Notifications page", "The phone's notifications grouped by app, at most 2 per app and then “altre N "
         "notifiche di …” (N more notifications from …); " + ui("Nascondi") + " (Hide) and " + ui("Nascondi tutte")
         + " (Hide all) act only in Phonestra, and a hidden notification reappears if it is updated."],
        ["Drawn phone", "On the right: time, Wi-Fi, battery and the phone's real screen (the mirror, "
         "interactive, with " + c("finestra::vista(…, SCHERMO, …)") + "; it receives keys only after a click). Files "
         "can be dragged onto it (" + rif("Installing and sending") + ")."],
        ["Veil", "When the phone cannot be used, a veil explains why; with the connection lost or the component "
         "failed it offers “Riconnetti ora” (Reconnect now)."],
    ], "«TAB» — The parts of the drawer") + \
    p("The app list comes from the helper (" + c("app::elenco") + ", command " + c("app <lato>") + ") and is valid "
      "only if it ends with the line " + c("fine\\t<n>") + ": a list interrupted by a drop is not used and is reread "
      "on reconnection (" + c("elenco_intero") + "). The app menu (right click) has " + ui("Apri") + " (Open; or "
      + ui("Porta in primo piano") + ", Bring to front, if it is already open), " + ui("Chiudi app") + " (Close app) "
      "if it is open, favorites, " + ui("Informazioni sull'app") + " (App info) and " + ui("Disinstalla…")
      + " (Uninstall…; disabled for system apps).")

S2 = p(c("finestra.rs") + ": one window per app, with " + c("finestra::vista") + " in the center (video, mouse, keyboard) "
       "and around it the bar with " + ui("Indietro") + " (Back), " + ui("Screenshot") + ", " + ui("Registra")
       + " (Record) and the ⋮ menu.",
       lead=True) + \
    table(["Command", "Shortcut", "What it does"], [
        [ui("Screenshot"), "", "Saves to " + c("<Immagini di XDG>/Phonestra/<app> AAAA-MM-GG HH.MM.SS.png")
         + " and copies to the clipboard"],
        [ui("Copia screenshot") + " (Copy screenshot; ⋮ menu)", key("Ctrl", "Shift", "C"), "Copies only"],
        [ui("Registra") + " (Record)", "", "MP4 in " + c("<Video di XDG>/Phonestra") + "; the button becomes " + c("● m:ss")
         + " (" + rif("Recording") + ")"],
        [ui("Ruota") + " (Rotate; ⋮ menu)", key("Ctrl", "R"), "Swaps the window's sides; nothing if it is maximized or recording"],
        [ui("Chiudi app") + " (⋮ menu)", key("Ctrl", "W"), "Closes the window and removes the app from recents"],
    ], "«TAB» — The commands of an app window") + ul([
        "The virtual display follows the window (" + rif("Resizing") + ").",
        "Portrait-only apps (the " + c("orientamento") + " event): fixed-size 9:16 window, no maximizing "
        "and no draggable edges; in full screen the app sits in a column shaped like the phone.",
        "Connection lost or component failed: the last image stays, blurred, with “Riconnetti ora” and “Chiudi”; "
        "the session restarts on its own when the connection returns and the app reappears where it was. With the phone "
        "locked only the subtitle changes.",
        "Protected screen: a message instead of the black image.",
        "When the window is closed, the app is removed from the phone's recents (" + c("ComandiVideo::chiudi(true)")
        + "); if it was the one controlling playback (YouTube, Facebook), it is paused. When the last "
        "session closes, the panel turns back on (" + rif("The phone in hand") + ").",
    ])

S3 = p("The drawer's Preferences page saves the user's choices in " + c("preferenze.toml") + " ("
       + rif("Configuration and data on the PC") + "). Each item has its own value.", lead=True) + \
    table(["Tab", "Item", "Value in " + c("preferenze.toml")], [
    ["App windows", "Esc goes back", c("esc_indietro")],
    ["Notifications", "Pop-up alert", c("avvisi")],
    ["Notifications", "App name only", c("solo_nome_app")],
    ["Notifications", "Apps allowed to alert (one switch per app)", c("app_silenziate")],
    ["Files", "Files sent to the phone: one of the 6 folders of " + c("azioni::CARTELLE")
     + " (Download, Documenti, Immagini, Fotocamera, Musica, Video)", c("cartella_file")],
    ["Files", "Files received from the phone: a folder on the PC; choosing the Scaricati (Downloads) folder saves "
     "“nothing”, so it follows the system folder", c("cartella_ricevuti")],
    ["Phone apps", "App list: " + ui("Aggiorna ora") + " (Refresh now)", "—"],
], "«TAB» — The drawer's Preferences page (" + c("configurazione::Preferenze") + ")")

S5 = p(c("prepara.rs") + " is “Aggiungi un telefono” (Add a phone) without a cable: the list of settings to enable on the phone "
       "(same Wi-Fi network, Developer options, any protections, Wireless debugging with “Pair device with "
       "pairing code”). For each item, the word to search for in Settings and “Chiedi a Google”, which opens "
       "Google's AI Mode with the question already written.", lead=True) + \
    p("Items visible on the network tick themselves: Wireless debugging turned on (" + c("_adb-tls-connect")
      + ") and the pairing code screen open (" + c("_adb-tls-pairing") + "), which enables the 6-digit field. With "
      "the code, Phonestra pairs (" + c("adb::abbina") + "), connects with " + c("adb_client") + ", removes the "
      "expiry of the authorization (" + c("settings put global adb_allowed_connection_time 0") + ") and saves the "
      "phone.") + \
    p(c("procedura.rs") + " is the USB cable fallback, for Android 10 or earlier (it opens from “Android 10 o "
      "precedente? Collega col cavo”). It advances on its own by checking the cable every second (" + c("usb.rs") + " reads "
      + c("/sys/bus/usb/devices") + " without opening the device) and recognizes these cases:") + \
    table(["Cable state (" + c("usb.rs") + ")", "How it is recognized", "Step shown"], [
        [c("SoloRicarica"), "no useful interface, known Android manufacturer (" + c("PRODUTTORI_ANDROID")
         + ")", "choose “Trasferimento file” (File transfer) from the USB notification"],
        [c("DebugSpento"), "MTP interface (" + c("06/01/01") + ") without ADB", "enable Developer options and USB debugging"],
        [c("DebugAttivoSoloRicarica"), "ADB (" + c("ff/42/01") + ") without MTP", "in “Solo ricarica” (Charging only) "
         "the systemd permission (uaccess) is missing and access may be denied: choose “Trasferimento file” (File transfer)"],
        [c("DebugAttivo"), "MTP and ADB", "“Consenti sempre” (Always allow), then the switch to Wi-Fi"],
    ], "«TAB» — The cable states") + \
    p("After the cable, " + c("telefono.rs") + " retries 3 times with a 1 s pause (the error “got AUTH” means "
      "consent is still missing), sets " + c("adb_allowed_connection_time 0") + " and " + c("adb_wifi_enabled 1") + " and "
      "waits up to 30 s for the user's confirmation. After 20 s stuck at the first step, the “Cosa vede il "
      "PC” (What the PC sees) box appears with the list of USB devices, to photograph for whoever is helping remotely.") + \
    p("The per-brand instructions come from " + c("dati/istruzioni.toml") + ", embedded at build time. The "
      "families are chosen by searching for the words in " + c("marche") + " in the manufacturer read from the cable; the last one, "
      "“Altri telefoni” (Other phones), has an empty " + c("marche") + " and acts as the fallback. The paths are the phone's "
      "Settings names as they appear in Italian; in the example: “Impostazioni › Informazioni sul telefono › Informazioni sul "
      "software” (Settings › About phone › Software information), “Numero build” (Build number), “Impostazioni › Opzioni "
      "sviluppatore” (Settings › Developer options) and “Impostazioni › Sicurezza e privacy › Blocco automatico” (Settings › "
      "Security and privacy › Auto Blocker).") + \
    code("""
[[famiglia]]
nome = "Samsung"
marche = ["samsung"]             # words searched in the manufacturer
verificata = true                # paths tested on a real phone
percorso_build = ["Impostazioni", "Informazioni sul telefono", "Informazioni sul software"]
voce_build = "Numero build"
percorso_debug = ["Impostazioni", "Opzioni sviluppatore"]
grigio = "…"                     # what to do if the USB debugging item is grayed out
sicurezza = "…"                  # extra step (Xiaomi: security settings)
[famiglia.prima]                 # a setting to change before anything else
percorso = ["Impostazioni", "Sicurezza e privacy", "Blocco automatico"]
voce = "Blocco automatico"
""", "toml", "dati/istruzioni.toml") + \
    note("with " + c("PHONESTRA_PROVA_PASSO") + " both windows show a specific step without a phone, "
         "for interface tests: " + c("acceso") + ", " + c("codice") + ", " + c("fatto") + " for "
         + c("prepara.rs") + "; " + c("debug") + ", " + c("consenti") + ", " + c("xiaomi") + ", " + c("wifi") + ", "
         + c("fatto") + " for " + c("procedura.rs") + ".", "Tests without a phone.")

S6 = p(c("notifiche.rs") + " reads " + c("dumpsys notification --noredact") + " (title and text of private "
       "notifications too) in the connection's 3-second round, and battery (" + c("dumpsys battery") + ", “charging” with status 2 "
       "or 5) and network (" + c("cmd wifi status") + ") every 30 s. It discards ongoing notifications ("
       + c("ONGOING_EVENT") + ", " + c("FOREGROUND_SERVICE") + "), group summaries (" + c("GROUP_SUMMARY")
       + ") and those with neither title nor text; it sorts them newest first.", lead=True) + \
    p(c("avvisi.rs") + " turns new notifications into system notifications with the D-Bus service "
      + c("org.freedesktop.Notifications") + " (GNOME, KDE, Xfce): GTK notifications on GNOME work only for "
      "programs with a " + c(".desktop") + " file, which Phonestra does not install. “Already seen” notifications are "
      "fixed at the first read; today, however, the first read happens with the drawer just opened, when the list is still "
      "empty, so at the first real read the notifications already on the phone raise an alert ("
      + rif("Appendix C — Known issues") + "). Clicking an alert opens the app. The preferences allow turning them off, "
      "showing only the app name (the text becomes “Nuova notifica”) and muting individual apps. The app "
      "icons for the alerts are written to " + c("~/.config/Phonestra/icone/<pacchetto>.png") + ".")

CHAPTER = ("The user interface", [
    ("Interface architecture", S1),
    ("The drawer", S1B),
    ("App windows", S2),
    ("Preferences", S3),
    ("The first connection", S5),
    ("Notifications and alerts", S6),
])
