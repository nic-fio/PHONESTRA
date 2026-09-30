from build import arrow, box, c, diamond, fig, note, p, rif, seq, table, term, text, ul, warn

S1 = p("Everything Phonestra does on the phone goes through a Java service written from scratch, started like the ADB "
       "shell and deleted at the end. This chapter describes its infrastructure; the following chapters describe the pieces: "
       "video, panel, audio, input.", lead=True) + \
    p(c("android/phonestra-helper.jar") + " contains a " + c("classes.dex") + ". The PC embeds it ("
      + c("app::AIUTO") + ", " + c("include_bytes!") + "), copies it to " + c("/data/local/tmp") + " and starts it with "
      + c("app_process") + ": it runs with the shell's uid (2000) and its permissions (capturing the screen and audio, "
      "injecting events, reading the clipboard). It is not an app, it is not installed, it does not appear in the settings.") + \
    p("The same jar has two modes of use, chosen by the first argument of " + c("phonestra.Aiuto") + ": the "
      "<b>service</b>, the long-running process, one per connection, which handles audio, video, input, clipboard and "
      "panel; and the <b>short helper commands</b>, which print and exit. For each short command the PC copies "
      "the jar under a name of its own (" + c("phonestra-aiuto.jar.<8 cifre esadecimali>") + ", because several requests may "
      "arrive together), runs it and deletes it (" + c("app.rs") + ").") + \
    table(["Command", "What it prints", "Who uses it"], [
        [c("app [lato]"), "the list of launcher apps with their PNG icons in base64, closed by a "
         + c("fine\\t<n>") + " line; with no arguments it means " + c("app 96"), "drawer (" + c("app::elenco") + ")"],
        [c("sfondo [larghezza]"), "the phone's wallpaper as PNG (default 540)", "phone drawn in the drawer"],
        [c("miniature <lato> <percorsi in base64>"), "one line " + c("indice\\tJPEG in base64") + " (quality 80) or "
         + c("indice\\t-") + " for each file", "“Ricevi file…” (" + rif("Receiving files from the phone") + ")"],
        [c("pannello [0|1]"), c("schermi=<quanti>") + " after turning the panel on (1, default) or off",
         "the guardian (" + rif("The two guardians") + ")"],
        [c("codificatori"), "the phone's audio and video encoders", c("phonestra-prova codificatori")],
        [c("audio …"), "captured audio, for study (" + c("sorgente=submix|loopback|render") + ", "
         + c("formato") + ", " + c("priorita") + ", " + c("voce") + ")", c("phonestra-prova audio-nostro")],
        [c("video-prova …"), "the video measurement tool of the study", c("phonestra-prova video-prova")],
        [c("servizio"), "the ready line, then stays alive", c("Componente::avvia")],
    ], "«TAB» — The jar's commands. An unknown command exits with code 2") + \
    p("Java only where Android requires it: the APIs needed (" + c("MediaCodec") + ", " + c("VirtualDisplay") + ", "
      + c("AudioPolicy") + ", " + c("InputManager") + ") exist only in Java. Audio and video encoding is done "
      "by the phone's encoders anyway.")

AVVIO = seq([("PC", "Componente::avvia", "navy"), ("adbd", "", "dark"), ("Service", "Java, uid 2000", "blue"),
             ("Guardian", "sh", "amber")], [
    (0, 1, "sync: phonestra-servizio-<random>.jar"),
    (0, 1, "shell,v2,raw: exec app_process … servizio"),
    (1, 2, "start (uid 2000)"),
    (0, 2, "secret (32 hex digits) on the input"),
    (2, 3, "setsid sh -c … (action pipe)"),
    (2, 2, "deletes the jar, context, self-test"),
    (2, 2, "LocalServerSocket phonestra_<32 hex>"),
    (2, 0, "ready line (process output)", True),
    (0, 2, "localabstract: “comandi” preamble"),
    (2, 0, "CIAO (versions, self-test)", True),
    ("sep", "each second, sent both ways"),
    (0, 2, "BATTITO"),
], "«FIG» — From startup to the first heartbeat", width=900)

S2 = p("The PC starts the service on a " + c("shell,v2") + " channel: it copies the jar, launches it with " + c("app_process")
       + ", passes it the secret on the input and waits for the ready line. Then it opens the command channel and receives the "
       + c("CIAO") + ".", lead=True) + AVVIO + \
    p("The ready line is " + c("phonestra-servizio pronto protocollo=1 socket=phonestra_<32 hex> pid=<pid>") + "; if "
      "startup fails, " + c("phonestra-servizio errore <causa>") + " and code 1. The PC waits 20 s for the ready line and 10 s for the "
      + c("CIAO") + "; if the service does not start, the PC deletes the jar itself. " + c("exec") + " makes the service take the "
      "place of " + c("sh") + ", so adbd's SIGHUP reaches it directly; " + c("--nice-name") + " makes it appear "
      "in " + c("ps") + " as " + c("phonestra-servizio") + ".")

S3 = p("Each channel is a " + c("localabstract:phonestra_<32 hex>") + " opened by the PC. The first bytes are the preamble: "
       + c("segreto (16 byte) · lunghezza del tipo u8 · tipo ASCII") + ". The service reads it with a 3 s "
       "timeout and chooses the handler from the part of the type before the colon (" + c("Servizio.TIPI") + ").",
       lead=True) + \
    table(["Type", "Handler", "Content"], [
        [c("comandi"), c("Servizio.comandi"), "Messages in both directions. Only one per service: a second "
         + c("comandi") + " channel is rejected"],
        [c("audio") + ", " + c("audio:aac") + ", " + c("audio:pcm"), c("CanaleAudio.gestisci"), "Audio packets "
         "to the PC (" + rif("The recipe") + "); an unknown format receives the text " + c("errore formato sconosciuto")],
        [c("video:<id>"), c("Video.canale"), "Video packets of a session (" + rif("The video:<id> channel") + ")"],
    ], "«TAB» — Channel types") + \
    p("Wrong secret, uid other than 2000 or unknown type: the socket is closed without a reply.")

S4 = p("The command channel carries all the short messages between PC and service, in both directions: requests with a reply, "
       "spontaneous events, the heartbeat.", lead=True) + \
    p("Messages with an 8-byte big-endian header, " + c("tipo u8 · bandiere u8 · id u16 · lunghezza u32")
       + ", and the content (16 MB at most: beyond that, the stream is corrupted and the channel is closed). " + c("id")
       + " ties the reply to the request (0 = spontaneous message); the " + c("0x01") + " flag marks it as a "
       "reply. The format is the same in " + c("Protocollo.java") + " and " + c("componente.rs") + ", checked by a test with "
       "the same bytes.") + \
    table(["Range", "Who", "Where"], [
        [c("0x01–0x0f"), "Infrastructure", c("componente::tipo") + ", " + c("Protocollo.java")],
        [c("0x10–0x1f"), "Testing and diagnostics", c("PROVA_CUSTODE")],
        [c("0x40–0x4f"), "Video and panel", rif("Video messages")],
        [c("0x50–0x5f"), "Input and clipboard", rif("Input and clipboard messages")],
    ], "«TAB» — Message type ranges") + \
    table(["Type", "Name", "Direction", "Content"], [
        [c("0x01"), c("CIAO"), "service → PC, first message", "Lines " + c("chiave=valore") + ": protocol, "
         "android, sdk, manufacturer, model, pid, " + c("avvio_ms") + ", " + c("autotest_ms") + ", "
         + c("autotest.<voce>=…") + " (" + c("ok") + " or the reason; for the guardian " + c("ok setsid") + ", "
         + c("morto setsid") + ", " + c("ok senza-setsid") + ")"],
        [c("0x02"), c("BATTITO"), "both directions, every second", "empty"],
        [c("0x03"), c("FINE"), "PC → service; same reply", "empty; after the reply the service exits with 0"],
        [c("0x04"), c("ERRORE"), "service → PC, as a reply", "text, for example “tipo sconosciuto 0x2a”"],
        [c("0x10"), c("PROVA_CUSTODE"), "PC → service", "creates a file that the guardian removes at the end (tests only)"],
    ], "«TAB» — Infrastructure messages") + \
    p("A type the service does not know receives " + c("ERRORE") + ": the PC understands that the jar is old. "
      + c("PROTOCOLLO") + " (1 today) changes only if an existing message changes meaning.")

S5 = p("Any message counts as a sign of life. After 5 s of silence each side considers the other gone: "
       "the PC closes the command channel (" + c("ricevi") + " returns " + c("None") + "; it checks every 200 ms), the service "
       "exits. This is needed because adbd on Wireless debugging does not notice by itself that a PC is gone (Wi-Fi lost, PC off).",
       lead=True) + \
    table(["Code", "When"], [
        ["0", c("FINE") + " requested by the PC"],
        ["1", "Startup error or unhandled exception"],
        ["3", "No message from the PC for 5 s"],
        ["4", "Command channel closed by the PC, or write impossible"],
        ["5", "Secret not received or no command channel within 10 s"],
        ["128+n", "Killed by signal n (129 = SIGHUP: startup channel closed; 137 = " + c("kill -9") + ")"],
    ], "«TAB» — Service exit codes (" + c("componente::descrivi_uscita") + ")") + \
    p("The service exits with " + c("System.exit") + " and, after 2 s, " + c("Runtime.halt") + " if something hangs. "
      "Restoring the phone is the guardian's job, which is there both at an orderly end and at a sudden one. The "
      "service restores by itself only what it can restore right away: the audio policy (the "
      + c("audio-fine") + " shutdown hook), the panel when it turns it back on, and " + c("min_refresh_rate") + " as soon as the "
      "panel is off; in those cases it also removes the corresponding action from the guardian.")

S6 = p("Phonestra changes a few things on the phone and must restore them even when something goes wrong. This is done by two "
       "shell processes, each tied to the life of something else:", lead=True) + \
    table(["", "Connection guardian", "Service guardian"], [
        ["Who starts it", "The PC, " + c("collegamento.rs") + ", with " + c("exec:"), "The service, " + c("Custode.java")
         + ", with " + c("setsid sh -c")],
        ["Tied to", "The ADB connection: it ends when adbd closes its input (" + c("cat >/dev/null") + ")",
         "The service process: it reads a pipe that only the service keeps open"],
        ["What it restores", "Screen timeout, media volume", "Physical panel, minimum "
         "display refresh rate, test tasks, jar copies"],
        ["Survives with", c("trap '' HUP TERM PIPE"), c("setsid") + " and " + c("trap '' HUP INT TERM PIPE")],
    ], "«TAB» — The two guardians") + \
    p("The screen timeout is set to the maximum (" + c("SPEGNIMENTO_LUNGO") + ", i.e. never) while Phonestra is open: with the "
      "phone asleep, apps on virtual displays receive no input, and taps from the PC do not count as "
      "phone activity (with 30 minutes it fell asleep during use, prove §55). The volume is set to the maximum "
      "because with the volume at 0 the Facebook app does not start the audio of reels; audio comes out only from the PC anyway.") + \
    p("Both values are also saved in " + c("telefoni.toml") + ": if Phonestra crashes without restoring them, the "
      "next connection restores the saved ones. The screen timeout is read from the phone at startup and "
      "restored at the end only if it is still Phonestra's value: if the user changes it during the connection, "
      "the user's value stays (prove §56). A value under 5 s is considered invalid; 30 minutes ("
      + c("SPEGNIMENTO_LUNGO_VECCHIO") + ", the value of releases up to rc.3) is treated as a leftover of a "
      "crashed Phonestra.") + \
    p("The service guardian receives the whole list of actions at every change, between a " + c("#inizio")
      + " line and a " + c("#fine") + " line; a half list does not replace the previous one. When the service dies it runs them "
      "in ascending order (for equal order, the last added first), one per line with " + c("sh -c")
      + " (one that fails does not stop the others), then deletes the service's jar and forgotten copies older "
      "than one minute, and terminates.") + \
    table(["Order", "Action", "Who sets it"], [
        ["400", "Turn the panel back on: " + c("app_process … phonestra.Aiuto pannello 1") + " with a copy of the jar "
         "kept for the guardian (" + c("phonestra-custode-<pid>.jar") + "); if the copy is missing, the old fallback "
         + c("KEYCODE_SLEEP") + "/" + c("WAKEUP"), c("Pannello.java")],
        ["410", "Restore " + c("min_refresh_rate") + " to what it was (only while waiting, 1 s at most, before the "
         "panel is turned off)", c("Pannello.java")],
        ["500", "Remove the tasks started by the input tests", c("InputProva.java")],
        ["900", "Remove the " + c("PROVA_CUSTODE") + " file", c("Servizio.java")],
    ], "«TAB» — The service guardian's actions") + \
    note("at first the guardian turned the screen back on by putting the phone to sleep and waking it up (" + c("KEYCODE_SLEEP")
         + "/" + c("WAKEUP") + "). On Samsung phones this locked the phone and made Wireless debugging drop at every "
         "Phonestra restart. Now the guardian calls " + c("setDisplayPowerMode") + " like the service (misure §51).",
         "Why the panel is turned back on in Java.") + \
    p("Virtual displays need no action: Android closes them when the process dies. Apps are not "
      "removed from recents on a drop, on purpose: when the connection comes back they return to their window with their "
      "state. Android removes the audio policy (" + rif("Who removes the capture") + ").")

S7 = p("Many of the APIs needed are hidden (" + c("IDisplayManager") + ", " + c("IWindowManager") + ", "
       + c("InputManagerGlobal") + ", " + c("IClipboard") + ", " + c("AudioPolicy") + "). In " + c("app_process")
       + " the hidden API policy is off: they are called via reflection.", lead=True) + \
    table(["Part", "How"], [
        ["Hidden", "Services from " + c("ServiceManager") + " + " + c("Stub.asInterface") + ", methods looked up by name and "
         "number of parameters among the known variants. Rule: look at what is there, not at the version."],
        ["Context", c("app_process") + " has no Android context. One step at a time, it sets up the main "
         "Looper, a system " + c("ActivityThread") + ", its " + c("ConfigurationController") + " (without it, "
         "on Samsung phones " + c("DisplayManagerGlobal") + " fails) and the context of the "
         + c("com.android.shell") + " package: since Android 16 the shell's permissions apply only with the right package."],
        ["Self-test", "At startup it checks that every hidden API is there and with which signature, without using it. The result goes to the PC "
         "in the " + c("CIAO") + " (" + c("Ciao::mancanti()") + ")."],
        ["System", c("Sistema.java") + " gathers the common pieces: the display manager for each virtual display, "
         "app launching (" + c("startActivityAsUser") + " with 11 parameters, with " + c("am start")
         + " as fallback), " + c("togliTask") + " when a window is closed, the internal shell commands."],
    ], "«TAB» — Context and hidden APIs") + \
    table(["Item", "What it checks"], [
        [c("contesto"), "Shell context ready, " + c("com.android.shell") + " package"],
        [c("permessi"), "The shell permissions the pieces need"],
        [c("display_manager"), c("createVirtualDisplay") + " and " + c("DisplayManagerGlobal")],
        [c("capture_display"), c("IWindowManager.captureDisplay") + " and the class of its arguments"],
        [c("task_stack_listener"), c("TaskStackListener") + " and its registration"],
        [c("inject_input_event"), c("injectInputEvent") + " (2 or 3 parameters), " + c("InputEvent.setDisplayId")],
        [c("audio_policy"), c("AudioPolicy") + ", " + c("AudioMix") + ", " + c("createAudioRecordSink") + ", registration"],
        [c("appunti"), "Methods of " + c("IClipboard") + "; presence of " + c("semclipboard")],
        [c("custode"), "Guardian alive, with or without " + c("setsid")],
    ], "«TAB» — Self-test items")

S8 = p("The service runs with the shell's permissions: nobody else must be able to control it, and at the end nothing must "
       "be left on the phone.", lead=True) + ul([
    "Abstract socket with a random 128-bit name; every channel must come from uid 2000 (" + c("getPeerCredentials")
    + ") and start with the secret (constant-time comparison). No TCP port open on the phone.",
    "The secret is passed on the process input, not on the command line: " + c("/proc/<pid>/cmdline") + " is "
    "readable by other processes with the same uid.",
    "No generic command: the PC cannot make the service run arbitrary shell commands. The guardian's actions "
    "are decided by the service; test commands accept only packages and actions made of letters, digits, dots and "
    "underscores.",
    "Nothing is left: jar deleted at startup and by the guardian, service that exits after 5 s without a PC, guardian that terminates "
    "after restoring.",
])

SMISTA = fig(
    box(20, 55, 150, 50, "Message", "from the service", "navy") + arrow(172, 80, 218, 80)
    + diamond(290, 80, 140, 64, "RISPOSTA?")
    + arrow(290, 113, 290, 158) + text(302, 140, "yes", 11, "#334155", "700", "start")
    + box(200, 160, 180, 50, "To the requester", "same id; ERRORE → error", "blue")
    + arrow(361, 80, 428, 80) + text(394, 72, "no", 11, "#334155", "700")
    + diamond(510, 80, 160, 64, "VIDEO_EVENTO?")
    + arrow(510, 113, 510, 158) + text(522, 140, "yes", 11, "#334155", "700", "start")
    + box(420, 160, 180, 50, "To session id=…", "evento=fine closes it", "blue")
    + arrow(591, 80, 668, 80) + text(629, 72, "no", 11, "#334155", "700")
    + box(670, 55, 210, 50, "To the type's subscribers", "for example APPUNTI_CAMBIATI", "blue"),
    900, 230, "«FIG» — Dispatching the command channel's messages")

S9 = p(c("Componente") + " (in " + c("componente.rs") + ") is a started service: " + c("avvia") + ", "
       + c("apri_canale") + ", " + c("manda") + ", " + c("richiesta") + ", " + c("ricevi") + ", " + c("chiudi")
       + ". In Phonestra it is always used through " + c("Condiviso") + ", which keeps it in a task (" + c("smista")
       + ") and can be cloned: one service per connection, many windows. When the last copy of "
       + c("Condiviso") + " goes away, the service is closed.", lead=True) + SMISTA + \
    table([c("Condiviso") + " method", "What it is for"], [
        [c("domanda(tipo, dati)"), "A request with a reply (5 s timeout; expired requests are checked "
         "every 500 ms)"],
        [c("manda(tipo, dati)"), "A message without a reply"],
        [c("apri_sessione"), c("VIDEO_APRI") + ": reply and events of the session; events that arrive before the "
         "reply are kept for 2 s"],
        [c("dimentica(sessione)"), "Stop dispatching the events of a closed session"],
        [c("iscrivi(tipo)"), "Receive the spontaneous messages of a type"],
        [c("mittente()"), "A cloneable " + c("Mittente") + ": taps and keys go straight into the command channel's "
         "queue, without going through the task"],
        [c("apritore()"), "An " + c("Apritore") + " to open the " + c("audio") + " and " + c("video:<id>") + " channels"],
        [c("nome_dispositivo()"), "The model from the " + c("CIAO") + " (“telefono”, phone, if missing)"],
        [c("finito()") + ", " + c("vivo()"), "Find out whether the service is still alive"],
        [c("chiudi()"), c("FINE") + ", waiting for the exit, closing the channels"],
    ], "«TAB» — The methods of " + c("Condiviso")) + \
    p(c("Collegamento::gira_componente") + " restarts the service if it dies while the phone is still connected, after 2 s. "
      "After " + c("CADUTE_MASSIME") + " (3) failed starts or drops it stops trying and publishes the reason ("
      + c("Collegamento::guasto") + "): drawer and windows show “Phonestra non parte sul telefono” (Phonestra won't start on the phone) with “Riconnetti ora” (Reconnect now).")

CHAPTER = ("The on-phone component", [
    ("What the component is", S1),
    ("Starting the service", S2),
    ("Channels and preamble", S3),
    ("The command channel", S4),
    ("Heartbeat and exit codes", S5),
    ("The two guardians", S6),
    ("Context, hidden APIs and self-test", S7),
    ("Service security", S8),
    ("PC side: Componente and Condiviso", S9),
])
