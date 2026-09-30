from build import arrow, box, c, fig, note, p, rif, table, text, zone

S1 = p("Phonestra grew out of a few rules, written before the code and followed in every piece. They explain choices "
       "that would otherwise look odd, such as an ADB client written from scratch or a guardian for every changed "
       "setting.", lead=True) + \
    table(["Principle", "What it means"], [
        ["<b>Nothing on the phone</b>", "No app installed. The component is copied to " + c("/data/local/tmp")
         + " at every connection and deleted at the end."],
        ["<b>The phone goes back to how it was</b>", "Every changed setting has a guardian that restores it, even if "
         "the PC disappears or the service dies suddenly (" + rif("The two guardians") + ")."],
        ["<b>Our own ADB</b>", "No " + c("adb") + " to install: TLS, pairing with the code, mDNS and multiple "
         "channels are written here (" + rif("The ADB client") + ")."],
        ["<b>Our own code</b>", "The phone component is written from scratch: no scrcpy, no third-party code "
         "to depend on."],
        ["<b>Measure before writing</b>", "Study, then measurements on the real phone, then code one piece at a time. "
         "The measurements stay in " + c("memoria/") + "."],
        ["<b>One file, no traces</b>", "One AppImage that contains everything; on the PC only " + c("~/.config/Phonestra")
         + " and " + c("~/.cache/Phonestra") + " (" + rif("Configuration and data on the PC") + ")."],
    ], "«TAB» — The principles that shaped Phonestra")

LATI = fig(
    zone(20, 12, 420, 262, "Linux PC (Rust)")
    + zone(460, 12, 420, 262, "Android 14+ phone (Java)")
    + box(36, 44, 170, 50, "Interface", "cassetto · finestra · ricevi", "navy")
    + box(256, 44, 170, 50, "PC-side pieces", "audio, video, input, clipboard", "blue")
    + box(36, 124, 170, 50, "Connection", "collegamento.rs", "blue")
    + box(256, 124, 170, 50, "Component", "componente.rs (Condiviso)", "blue")
    + box(36, 204, 170, 50, "mDNS", "rete.rs", "dark")
    + box(256, 204, 170, 50, "ADB client", "src/adb: TCP + TLS, channels", "dark")
    + box(480, 44, 380, 50, "Audio · Video · Input · Clipboard · Panel", "service classes", "light")
    + box(480, 124, 180, 50, "Service", "app_process, uid 2000", "blue")
    + box(690, 124, 170, 50, "Guardian", "sh with setsid", "amber")
    + box(480, 204, 180, 50, "adbd", "Wireless debugging", "dark")
    + arrow(121, 96, 121, 122) + arrow(208, 69, 254, 69) + arrow(341, 96, 341, 122)
    + arrow(208, 149, 254, 149) + arrow(121, 176, 121, 202) + arrow(341, 176, 341, 202)
    + arrow(428, 229, 478, 229, "#003a90") + text(453, 262, "TLS", 11, "#003a90", "700")
    + arrow(570, 202, 570, 176) + arrow(570, 122, 570, 96)
    + arrow(662, 149, 688, 149, "#475569", True),
    900, 290, "«FIG» — The parts of Phonestra and who calls whom")

S2 = p("Phonestra has two sides: the program on the PC, in Rust, and the component on the phone, in Java. They talk "
       "to each other only through ADB, over a single encrypted Wi-Fi connection.", lead=True) + LATI + \
    p("On the PC each layer calls only the ones below it: the interface does not talk to ADB, and the ADB client knows "
      "nothing about video or audio. On the phone there is a single process per connection, the service, which serves "
      "all windows, the drawer's screen, audio and clipboard. The guardian is a separate shell process that stays "
      "alive even when the service dies; the dashed line is the pipe through which the service passes it the restore "
      "actions.") + \
    table(["Layer", "Responsibility", "Main modules"], [
        ["Interface", "The GTK windows: drawer, app windows, first connection, system alerts",
         c("cassetto.rs") + ", " + c("finestra.rs") + ", " + c("ricevi.rs") + ", " + c("prepara.rs") + ", "
         + c("procedura.rs") + ", " + c("avvisi.rs")],
        ["Connection", "Finding the phone, keeping it connected, the 3-second round, the connection guardian, "
         "shutdown", c("collegamento.rs") + ", " + c("rete.rs") + ", " + c("notifiche.rs") + ", "
         + c("configurazione.rs")],
        ["PC-side pieces", "Audio, video, input and clipboard over the service's channels",
         c("audio_nostro.rs") + ", " + c("video_nostro/") + ", " + c("input_nostro.rs") + ", " + c("appunti.rs")],
        ["Component", "Service startup, command channel, message dispatching", c("componente.rs")],
        ["ADB client", "Messages, TLS, channels, flow control", c("src/adb/")],
        ["Phone", "Service, guardian, classes of the pieces", c("telefono/aiuto/src/phonestra/")],
    ], "«TAB» — The program's layers and their modules") + \
    note("the state shared by the interface and the connection lives in the " + c("Collegamento") + ", in "
         + c("watch") + " channels: " + c("stato()") + ", " + c("guasto()") + ", " + c("info()") + ", " + c("notifiche()")
         + ", " + c("adb()") + ", " + c("componente()") + ". Drawer and windows subscribe and restart on their own "
         "when a value changes (" + rif("Life of a connection") + ").", "Where the state lives.")

S3 = p("Three worlds run together in the Phonestra process: the GTK thread, the tokio runtime and GStreamer. Each has "
       "its own job, and they talk to each other only through channels.", lead=True) + \
    table(["Where", "What runs", "How they talk"], [
        ["GTK main thread", "The whole interface; " + c("glib::spawn_future_local") + " for tasks that "
         "touch widgets", "Reads the " + c("watch") + " channels of the " + c("Collegamento") + ", sends commands over "
         + c("mpsc") + " channels"],
        ["tokio runtime (" + c("phonestra::esecutore()") + ")", "Connection, ADB client, component, audio, video, "
         "input", c("watch") + " for states, " + c("mpsc") + " for messages, " + c("Notify") + " for wake-ups"],
        ["GStreamer", "Video and audio decoding, playback, recording", c("appsrc") + " elements fed by tokio "
         "tasks"],
    ], "«TAB» — Threads and tasks") + \
    note("a " + c("Widget") + " never leaves the GTK thread; an " + c("Adb") + ", a " + c("Condiviso") + " or a "
         + c("Mittente") + " is cloned and goes wherever it is needed.", "Rule of thumb.")

CHAPTER = ("Overall architecture", [
    ("Architectural principles", S1),
    ("The two sides and the layers", S2),
    ("Threads and tasks", S3),
])
