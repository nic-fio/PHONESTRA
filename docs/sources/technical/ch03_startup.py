from build import arrow, box, c, fig, note, p, path, rif, seq, steps, table, text, ui

AVVIO = seq([("main", "bin/phonestra.rs", "navy"), ("adw::Application", "one per session", "blue"),
             ("Collegamento", "collegamento.rs", "blue"), ("Drawer", "cassetto.rs", "light")], [
    (0, 0, "gst::init()"),
    (0, 1, "application_id io.github.nic_fio.Phonestra"),
    (1, 2, "primo_telefono()"),
    ("sep", "with no phones configured: prepara::apri (“Aggiungi un telefono”) and stop"),
    (1, 2, "esecutore().spawn(mantieni())"),
    (1, 3, "cassetto::apri"),
    (3, 2, "subscribes to stato, guasto, info, notifiche", True),
    ("nota", 1, "last window closed → Collegamento::chiudi, waits for Stato::Chiuso, then exits"),
], "«FIG» — From main to the drawer", width=900)

S1 = p("The program starts from " + c("main") + ", in " + c("src/bin/phonestra.rs") + ": it initializes GStreamer, "
       "creates the GTK application and hands the configured phone over to a " + c("Collegamento") + ", which lives in "
       "a tokio task of its own.", lead=True) + AVVIO + \
    p("A second launch does not open a second connection: " + c("adw::Application") + " is unique per session and "
      + c("connect_activate") + " brings the drawer back to the foreground (reopening it if it was closed). With a "
      "package name as an argument (" + c("phonestra com.android.chrome") + ") that app opens right away too, but only "
      "on the first launch: the instance already running does not receive the argument.") + \
    p("Ctrl+C and SIGTERM close the windows just as the user would, so the phone is restored. When the drawer asks "
      "to switch phones (" + rif("Multiple phones") + "), " + c("main") + " relaunches the program at the end: "
      + c("$APPIMAGE") + " if set, otherwise the current executable.")

STATI = fig(
    box(40, 40, 172, 54, "Cerco", "mDNS, then Adb::wifi", "light")
    + box(365, 40, 172, 54, "Collegato", "apps in use", "blue")
    + box(690, 40, 172, 54, "Bloccato", "apps receive no input", "amber")
    + box(200, 170, 172, 54, "Perso", "retries on its own", "dark")
    + box(690, 170, 172, 54, "Chiuso", "Phonestra ends", "navy")
    + arrow(214, 60, 363, 60) + text(288, 52, "connection open", 11)
    + arrow(539, 58, 688, 58) + text(613, 50, "phone locked", 11)
    + arrow(688, 80, 539, 80) + text(613, 98, "unlocked", 11)
    + arrow(430, 96, 345, 168) + text(398, 140, "drop", 11, "#334155", "400", "start")
    + arrow(110, 96, 235, 168) + text(196, 140, "failed, or 30 s", 11, "#334155", "400", "start")
    + path([(198, 206), (70, 206), (70, 96)], "#475569", True) + text(134, 224, "wait 2 → 10 s", 11)
    + arrow(374, 197, 688, 197, "#475569", True) + text(531, 189, "the user closes Phonestra (from any state)", 11),
    900, 240, "«FIG» — The connection states (Stato) and what changes them")

S2 = p(c("Collegamento::mantieni") + " is the heart of the program. It runs in a tokio task as long as Phonestra "
       "stays open and starts over at every drop.", lead=True) + steps([
    "<b>Finds the phone</b>: first the last address that worked, then the mDNS search for the "
    + c("_adb-tls-connect._tcp") + " service with the saved serial number (" + c("rete::indirizzo_attivo") + "). The "
    "port changes every time Wireless debugging is turned on, so it must always be read again.",
    "<b>Opens the connection</b>: " + c("Adb::wifi") + " (TCP, STLS, TLS with Phonestra's certificate), with a 30 s "
    "timeout.",
    "<b>Reads and saves the user's values</b> (screen timeout and media volume), also in "
    + c("telefoni.toml") + ". They are read only once for all reconnections; at every reconnection Phonestra checks "
    "whether the user has changed the screen timeout in the meantime.",
    "<b>Reads the screen's density and shape</b> (" + c("wm density; wm size") + ", first the “Override” values chosen "
    "by the user, then the “Physical” ones): the density is needed by the virtual displays, the aspect ratio by the "
    "column of portrait-only apps.",
    "<b>Starts the connection guardian</b>: a shell " + c("exec:") + " that raises screen timeout and volume to the "
    "maximum and restores them when the channel closes (" + rif("The two guardians") + ").",
    "<b>Publishes</b> the " + c("Adb") + " (" + c("watch") + ") and the " + c("Collegato") + " state: drawer and "
    "windows restart on their own.",
    "<b>Starts the component</b> in a task (" + c("gira_componente") + "): service on the phone, then " + c("Condiviso")
    + " published for windows and drawer, then audio (after the mirror and 5 s, " + rif("The startup order") + ").",
    "<b>Listens to the clipboard</b> of the phone (" + c("appunti::ascolta") + ").",
    "<b>3-second round</b> as long as the connection holds (next section).",
    "<b>Shutdown</b> when the user closes Phonestra (" + rif("Shutdown") + ").",
]) + p("The state is published as " + c("Stato::{Cerco, Collegato, Bloccato, Perso, Chiuso}") + ". Between one "
       "attempt and the next the wait grows from 2 to 10 s; “Riconnetti ora” (Reconnect now, " + c("riconnetti_ora")
       + ") interrupts it.") + STATI

S3 = p("A single " + c("exec:") + " every 3 s fetches lock state, calls and notifications, and also serves as a check "
       "that the phone is responding: if it does not respond within 5 s, the connection is considered dropped. Battery "
       "and network and, with the phone “in hand”, " + c("lastUserActivityTime") + " are separate commands, with a 5 s "
       "timeout each: if they fail, the connection does not drop.", lead=True) + \
    table(["Command", "When", "Used for"], [
        [c("dumpsys window | grep -m1 -o 'isKeyguardShowing=[a-z]*'"), "every round",
         "knowing whether the phone is locked (" + c("Bloccato") + " state) and whether the user unlocked it by hand"],
        [c("dumpsys telephony.registry | grep -o 'mCallState=[12]'"), "every round",
         "incoming (1) and ongoing (2) calls; with two SIMs there is one line per SIM (" + rif("Calls") + ")"],
        [c("notifiche::COMANDO_NOTIFICHE"), "every round", "the drawer's notifications (" + rif("Notifications and alerts") + ")"],
        [c("notifiche::COMANDO_INFO"), "on the first round and then every 10 (30 s)", "battery and network for the drawn phone"],
        [c("dumpsys power | grep -m1 lastUserActivityTime="), "only with the phone “in hand”",
         "turning the panel off after the screen timeout with no touches (" + rif("The phone in hand") + ")"],
    ], "«TAB» — The commands of the 3-second round") + \
    p("Protected screens do not go through here: the component reports them, session by session ("
      + rif("App events") + ").")

S4 = p("When the user closes Phonestra, " + c("usa") + " leaves the round and restores the phone to how it was, one "
       "step at a time, with a timeout for each.", lead=True) + steps([
    "Music or video that is playing is paused (" + c("cmd media_session dispatch pause") + ", at most 3 s): "
    "otherwise, once the capture is removed, it would resume from the phone's speaker.",
    "The " + c("Adb") + " is withdrawn; the windows close their sessions and remove their apps from recents (waiting "
    "up to 5 s).",
    "The component receives " + c("FINE") + " and shuts down (at most 12 s); its guardian turns the panel back on.",
    "The connection guardian is closed, and it restores volume and screen timeout.",
    "For 3 s Phonestra checks that the screen timeout is no longer Phonestra's; if the guardian has not restored it, "
    "Phonestra restores it directly. If even that fails, it stays recorded in " + c("telefoni.toml") + " and is "
    "restored at the next connection.",
])

S5 = p("Phonestra uses one phone at a time (the option of several phones active together was discarded). The other "
       "configured phones appear in the sidebar as “non attivo” (inactive); to switch to one of them the program "
       "closes and restarts.", lead=True) + steps([
    "Clicking an inactive phone asks for confirmation if apps are open.",
    c("Telefoni::metti_primo") + " moves it to the top of " + c("telefoni.toml") + ": it is the one opened at startup.",
    "The drawer sets " + c("cassetto::RIAVVIA") + " and closes all windows, like a normal shutdown: the previous phone "
    "is restored.",
    c("main") + " relaunches the program (" + c("$APPIMAGE") + " or the current executable), which connects to the new "
    "phone.",
]) + p(ui("Dimentica questo telefono…") + " (Forget this phone…) does the same restart if other phones remain; if "
       "none are left, Phonestra closes, and at the next start “Aggiungi un telefono” (Add a phone) opens.")

S6 = p("Phonestra keeps its data in two folders on the PC: the configuration in " + c("~/.config/Phonestra") + ", "
       "things that can be recreated in " + c("~/.cache/Phonestra") + ". " + c("telefoni.toml") + " and "
       + c("preferenze.toml") + " are read and written in " + c("configurazione.rs") + " (" + c("Telefoni")
       + ", " + c("Preferenze") + ").", lead=True) + \
    table(["File", "Contents"], [
        [c("~/.config/Phonestra/adbkey"), "Phonestra's private RSA key (permissions 600), different from that of "
         + c("adb") + ": the phone authorizes Phonestra as a separate computer"],
        [c("~/.config/Phonestra/telefoni.toml"), "Per phone: " + c("seriale") + ", " + c("nome") + " (the device name, not shown), "
         + c("modello") + ", " + c("nome_scelto") + " (from " + ui("Rinomina…") + "), " + c("modello_commerciale")
         + ", " + c("tablet") + " (" + c("Telefono::nome_mostrato") + " builds the displayed name from them), " + c("android") + ", " + c("ultimo_indirizzo") + ", " + c("spegnimento_originale")
         + ", " + c("volume_originale") + ", " + c("preferiti") + "; the first one is opened at startup"],
        [c("~/.config/Phonestra/preferenze.toml"), c("esc_indietro") + ", " + c("avvisi") + ", " + c("solo_nome_app")
         + ", " + c("app_silenziate") + ", " + c("cartella_file") + ", " + c("cartella_ricevuti")
         + " (if missing: the Scaricati (Downloads) folder); " + rif("Preferences")],
        [c("~/.config/Phonestra/icone/"), "App icons for system alerts"],
        [c("~/.cache/Phonestra/"), "GStreamer plugin registry, image loaders and fallback libraries of the "
         "AppImage, the logo for the “Informazioni” (About) window (" + c("icone/phonestra.png") + "): it can be deleted"],
    ], "«TAB» — Phonestra's data on the PC") + \
    p("Deleting " + c("~/.config/Phonestra") + " brings Phonestra back to its initial state. The configuration "
      "folder follows " + c("$XDG_CONFIG_HOME") + "; screenshots, recordings and received files go to the XDG user "
      "folders (Pictures, Videos, Downloads).") + \
    note("on a new PC the phone must be paired again: the ADB key and the paired phones are not in the "
         "repository.", "A new PC.")

CHAPTER = ("Startup and life cycle", [
    ("Program startup", S1),
    ("Life of a connection", S2),
    ("The 3-second round", S3),
    ("Shutdown", S4),
    ("Multiple phones", S5),
    ("Configuration and data on the PC", S6),
])
