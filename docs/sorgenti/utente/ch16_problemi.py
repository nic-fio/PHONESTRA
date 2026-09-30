from build import c, note, p, rif, table, term, tip, ui, ul


S1 = table(["Symptom", "Likely cause", "Remedy"], [
    "Connection",
    ["The drawer stays on " + ui("Collegamento a <nome>…") + " (connecting to &lt;name&gt;…)", "The phone is off, "
     "locked, on another Wi-Fi network, or " + ui("Debug wireless") + " (Wireless debugging) has turned off (this "
     "happens when changing network)", "Unlock the phone, check the "
     "network, turn " + ui("Debug wireless") + " back on in " + ui("Opzioni sviluppatore") + " (Developer options). "
     "Phonestra connects by itself as soon as it finds it (" + rif("Later connections") + ")."],
    ["Orange pill, " + ui("bloccato: sbloccalo") + " (locked: unlock it)", "The phone has locked", "Unlock it: "
     "Phonestra reconnects by itself (" + rif("Locked phone") + ")."],
    ["Orange pill, " + ui("riconnessione…") + " (reconnecting…)", "The connection dropped (weak Wi-Fi, phone locked)",
     "Wait a few seconds, or use " + ui("Riconnetti ora") + " (Reconnect now) or " + ui("Riconnetti") + "."],
    ["Red pill, " + ui("Phonestra non parte sul telefono") + " (Phonestra does not start on the phone)", "The part "
     "of Phonestra that shows the apps does not start on the phone; the reason is written in brackets", "Press "
     + ui("Riconnetti ora") + "; if that is not enough, restart the phone and wait for Phonestra to reconnect."],
    ["In " + ui("Aggiungi un telefono") + " the code field stays grey", "Phonestra cannot see the code screen: "
     "the phone is on another network, or a firewall on the PC blocks network discovery", "Check the network; "
     "close and reopen " + ui("Associa dispositivo con codice di associazione") + " (Pair device with pairing code). "
     "If the PC has a firewall, allow device discovery on the local network (mDNS, UDP port 5353)."],
    [ui("Codice non accettato (…)") + " (code not accepted)", "Wrong or expired code", "Reopen the code screen on the "
     "phone and type the new code (" + rif("The 6-digit code") + ")."],
    [ui("Debug wireless") + " grey or turning itself off again", "A phone protection prevents it ("
     + ui("Blocco automatico") + ", " + ui("Protezione avanzata") + ": Auto Blocker, Advanced Protection), or the "
     "network is not a Wi-Fi network",
     "Turn off the protection (item 3 of " + ui("Aggiungi un telefono") + "); use a Wi-Fi network, not mobile data."],
    "Apps and windows",
    ["The apps do not appear: " + ui("App non lette") + " (apps not read)", "The connection dropped while the list "
     "was being read", "Wait for the reconnection, or use " + ui("Riconnetti") + "."],
    ["An app just installed on the phone is missing", "The list is read when connecting", ui("Preferenze") + " › "
     + ui("Aggiorna ora") + " (Preferences › Update now)."],
    ["Black window with " + ui("Schermata protetta") + " (protected screen)", "The app does not allow that screen to "
     "be shown outside the phone", "Use the phone in hand for that screen (" + rif("Protected screens") + ")."],
    ["The drawer grids are faded and do not respond", "The phone cannot be used at that moment",
     "Look at the status in the pill and at the veil over the phone screen."],
    ["On Xiaomi, Redmi and Poco phones the apps are visible but do not respond to mouse and keyboard", "A Xiaomi "
     "security setting is missing", "In " + ui("Opzioni sviluppatore") + " turn on " + ui("Debug USB (impostazioni di "
     "sicurezza)") + " (USB debugging (Security settings))."],
    ["Keys do not reach the phone screen in the drawer", "The keys go to the app search", "Click "
     "on the phone screen first."],
    ["An app window does not widen or maximize", "The app only accepts portrait orientation", "This is intended ("
     + rif("Size, rotation and portrait-only apps") + ")."],
    "Audio, clipboard, notifications",
    ["No audio right after connecting", "Audio starts a few seconds after the picture", "Wait a few "
     "seconds."],
    ["An app stays silent", "The app forbids capturing its audio, or it is a call", "Calls are heard "
     "on the phone (" + rif("Calls") + ")."],
    ["No audio from any app", "PC volume low, or wrong audio output", "Check the volume and "
     "the output in the desktop's mixer."],
    ["Phone volume left at the maximum after closing", "Known issue", "Lower it with the phone's buttons."],
    ["Text copied on the phone does not paste on the PC", "On GNOME the clipboard only changes while a Phonestra "
     "window is active", "Click a Phonestra window and paste again."],
    [ui("Password non inviata al telefono") + " (password not sent to the phone)", "The text comes from a password "
     "manager", "This is intended: type it by hand "
     "or copy it on the phone."],
    ["On opening, alerts arrive for old notifications", "Known issue", "Only at startup: later alerts "
     "concern new notifications."],
    ["No pop-up alerts", ui("Avviso a comparsa") + " (pop-up alert) off, app muted, desktop “Do not disturb”",
     "Check " + ui("Preferenze") + " › " + ui("NOTIFICHE") + " and the desktop."],
    "Files",
    ["Installation refused", "The phone says why in the message", "See " + rif("Installing an app") + "."],
    [ui("Cartella non leggibile") + " (folder not readable) in " + ui("Ricevi file…"), ui("Il telefono non la mostra al PC.")
     + " (the phone does not show it to the PC)",
     "It is a private folder of an app or of the system: it cannot be received."],
    [ui("Aspetta la fine del trasferimento in corso") + " (wait for the current transfer to finish)", "A transfer is "
     "already in progress", "Wait for it to finish, "
     "or cancel it with the " + ui("×") + "."],
    "Startup",
    ["The AppImage file does not start", "It is not executable", "Make it executable (" + rif("Downloading Phonestra") + ")."],
    ["Starting Phonestra again brings up the drawer that is already open", "Phonestra was already open", "This is "
     "intended: only one Phonestra at a time."],
], "«TAB» — Common problems")

S2 = table(["Message", "Where", "Meaning"], [
    [ui("Collegamento a <nome>…"), ui("App") + " page", "Phonestra is looking for the phone."],
    [ui("Lettura delle app…"), ui("App") + " page", "Phonestra is reading the list of apps from the phone."],
    [ui("Nessuna app trovata"), ui("App") + " page", "No app with the name searched for."],
    [ui("Telefono bloccato"), "Phone screen in the drawer", "The phone needs to be unlocked."],
    [ui("Collegamento perso"), "Phone screen in the drawer", "Phonestra retries by itself."],
    [ui("Riconnessione…"), "An app window", "Phonestra retries by itself; the app returns to where it was."],
    [ui("scollegato"), "Subtitle of a window", "The connection has dropped."],
    [ui("Phonestra non parte sul telefono"), "Pill, screen in the drawer, windows", "See " + rif("Common problems")
     + "."],
    [ui("Il telefono non è collegato"), "Short message in the drawer", "The action requires the connection."],
    [ui("Registrazione non avviata: …"), "An app window", "The recording did not start; the reason follows."],
    [ui("Screenshot non salvato: …"), "An app window", "The file could not be written; the reason follows."],
    [ui("Nome non salvato: …") + ", " + ui("Preferenza non salvata: …"), "Drawer", "Phonestra cannot "
     "write to its folder " + c("~/.config/Phonestra") + "."],
], "«TAB» — Phonestra's messages")

S3 = p("Phonestra writes a log of what it does: connections, drops, screen on and off, errors. It helps whoever is "
       "helping you understand a problem.", lead=True) + ul([
    "If you start Phonestra from a terminal, the log lines appear in the terminal.",
    "If you start it with a double-click, they usually end up in the system log (the journal) and can be read with "
    "the command below.",
]) + term("""
$ journalctl --user --since today | grep -i phonestra
""", "Today's lines") + \
    note("the log may contain the phone's name and the names of apps. Before sending it to someone, read it "
         "through and remove whatever you do not want to share.", "Before sharing it.") + \
    tip("along with the log, a picture of the window with the message is useful, and the Phonestra version ("
        + ui("Informazioni") + ", About).", "What to send.")

S4 = ul([
    "Only one phone active at a time (" + rif("Multiple phones") + ").",
    "The phone must stay on, unlocked and on the same Wi-Fi network as the PC.",
    "Calls, including those made from apps, are heard and made on the phone. Text messages are read and written "
    "with the messaging app, in a window, like the other apps.",
    "The PC's microphone and webcam do not reach the phone; the phone does not act as a webcam for the PC.",
    "Protected screens stay black; apps that forbid capturing their audio stay silent.",
    "Notifications hidden in Phonestra stay on the phone: Phonestra does not delete the phone's notifications.",
    ui("Opzioni sviluppatore") + " and " + ui("Debug wireless") + " can only be turned on by hand, on the phone: "
    "Android does not allow a program to do it.",
    "Only single " + c(".apk") + " files can be installed.",
    "Phonestra is made for Linux on 64-bit Intel or AMD processors.",
])

CHAPTER = ("Troubleshooting", [
    ("Common problems", S1),
    ("Phonestra's messages", S2),
    ("Phonestra's log", S3),
    ("Limitations to be aware of", S4),
])
