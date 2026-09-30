from build import c, note, p, rif, table, term, ul

S1 = p("Two levels: the tests on the PC, which need no phone and run at every commit, and the tests on the real "
       "phone with " + c("phonestra-prova") + ".", lead=True) + \
    p(c("cargo test") + " tests the pure parts: the formats of the chunked messages (commands, audio, video, input), the preamble, "
      "the ready line, " + c("CIAO") + ", the heartbeat, " + c("shell,v2") + " packets, dispatching without a network, audio timing and "
      "margin, real AAC decoded and written to MP4, flow control against a fake adbd, mDNS, "
      "parsing of " + c("dumpsys") + ", message numbers matching between Java and Rust, and these manuals ("
      + c("tests/manuale.rs") + ", " + rif("The manuals and their checks") + ").")

S2 = p("A separate executable, without a user interface, that uses the same code as the program. The component tests are "
       "run with the phone unlocked and Phonestra closed; each one cleans up after itself and checks that nothing is "
       "left behind on the phone.", lead=True) + \
    table(["Command", "What it does"], [
        "Connection",
        [c("cerca") + ", " + c("collega") + ", " + c("banner"), "mDNS discovery, connection to the saved phone, "
         "features announced by adbd"],
        [c("abbina <codice> [ip:porta]"), "Pairing with the 6-digit code; without an address it finds the "
         "pairing-code screen by itself with mDNS"],
        [c("usb") + ", " + c("prepara") + ", " + c("shell-usb <comando>") + ", " + c("procedura"), "The cable: status, "
         "Wi-Fi preparation, shell, the fallback procedure"],
        [c("shell <comando>"), "A shell command over Wi-Fi (instead of " + c("adb shell") + ")"],
        [c("throughput [MB] [--senza-delayed-ack] [--payload N] [--finestra N] [--latenza] [--exec] [--alla-lettura]"),
         "Throughput and latency of the ADB transport"],
        [c("canali"), "Several commands at once on the same connection and a file copy with " + c("sync:") + " (old test)"],
        "Helper and drawer",
        [c("app") + ", " + c("sfondo") + ", " + c("notifiche") + ", " + c("codificatori"), "Helper commands "
         "and drawer reads"],
        [c("file <cartella>") + ", " + c("file ricevi <percorso> <destinazione>") + ", " + c("file miniature <percorsi>"),
         "“Ricevi file…” from the command line: listing, copy to the PC, thumbnails"],
        "Component",
        [c("servizio [secondi] [--sparisci]"), "The skeleton of the component: startup, " + c("CIAO") + ", heartbeat, "
         "guardian, exit; " + c("--sparisci") + " simulates a PC that disappears"],
        [c("custode [abbandona]"), "A test guardian like the connection's one (screen timeout at 1,234 s, "
         "without the volume): it closes the channel and checks that the value is restored; with " + c("abbandona")
         + " it exits without closing it, and the restore is checked by hand (" + c("shell settings get system screen_off_timeout") + ")"],
        [c("audio-componente <secondi> [aac|pcm] [--ascolta] [--uccidi]"), "Audio from the component into a file ("
         + c("phonestra-prova.aac") + "/" + c(".wav") + "), check of timing and policies"],
        [c("video-componente app|schermo [--app P] [--secondi N] [--codec C] [--senza-pannello]"), "Virtual display or "
         "mirror, keyframes, resizing, panel, events; saves the stream for " + c("ffprobe")],
        [c("input-componente appunti|tocchi|testo|tutte [--misura LxA] [--dpi D] [--app P] [--azione A[:P]] [--tocca X,Y]"),
         "Clipboard, taps, scroll wheel, pinch, text, verified with the “signature” of the test screen"],
        "Study",
        [c("video-prova schermo|chiave|istanze|protetto|task|permessi|codificatori"), "The video measurement tool "
         "of the study"],
        [c("audio-nostro <secondi> [submix|loopback|render] [pcm|aac] [senza-priorita] [voce]"), "The audio "
         "measurements of the study"],
    ], "«TAB» — The commands of " + c("phonestra-prova")) + \
    term("""
$ ./target/debug/phonestra-prova servizio 10
$ ./target/debug/phonestra-prova video-componente app --secondi 15
$ PHONESTRA_DEBUG=1 ./target/debug/phonestra-prova input-componente tutte
""", "Examples")

S3 = p("Tests on the real phone follow a few rules: the phone must go back to how it was, and every measurement must "
       "stay on record.", lead=True) + ul([
    c("phonestra-prova") + " first; the system " + c("adb") + " only for diagnostics the tool cannot do "
    "(it uses a different key: the phone asks for a new authorization).",
    "At the end of a test: shut down any " + c("adb") + " server that was started and any open sessions; do not leave "
    "phone settings changed.",
    "Before restarting Phonestra for a test, check all the " + c("mCallState") + " lines (one per SIM): never "
    "during a call (" + rif("Calls") + ").",
    "Measurements and results go in " + c("memoria/prove-collegamento.md") + ", with the section number (§).",
])

S4 = p("The " + c("PHONESTRA_*") + " variables turn on diagnostics or change a parameter, for tests and measurements.",
       lead=True) + \
    table(["Variable", "Effect"], [
    [c("PHONESTRA_DEBUG=1"), "Diagnostics on the terminal (works with any value): audio measurements, frames "
     "drawn and dropped, clipboard, window scaling"],
    [c("PHONESTRA_FOTO=<cartella>"), "Every window saves itself as PNG 6 s after opening and then every 6 s ("
     + c("foto.rs") + ")"],
    [c("PHONESTRA_PROVA_PASSO=…"), "Shows a step of the first connection without a phone (" + rif("The first connection") + ")"],
    [c("PHONESTRA_PROVA_RICEVI=<posto>"), "Opens “Ricevi file…” by itself on connection, at the given place"],
    [c("PHONESTRA_AUDIO_CODEC=pcm"), "PCM audio instead of AAC (also " + c("raw") + ")"],
    [c("PHONESTRA_VIDEO_FPS") + ", " + c("PHONESTRA_VIDEO_PRIORITA") + ", " + c("PHONESTRA_VIDEO_PROTETTA=0"),
     "Test switches for the encoder and for the protected-screen check (they become " + c("max_fps=")
     + ", " + c("priorita=") + ", " + c("protetta=") + " of " + c("VIDEO_APRI") + ")"],
    [c("PHONESTRA_ADB_DELAYED_ACK") + ", " + c("PHONESTRA_ADB_PAYLOAD") + ", " + c("PHONESTRA_ADB_FINESTRA"),
     "ADB transport parameters (" + rif("Channels and flow control") + ")"],
], "«TAB» — The environment variables") + \
    p("The log of Phonestra started from the AppImage is in the PC's journal: " + c("journalctl --user --since today | "
      "grep -i phonestra") + " (" + rif("The change log") + ").")

S5 = p("Both manuals are generated: the sources are in " + c("docs/sorgenti/") + ", one per chapter in "
       + c("tecnico/") + " and " + c("utente/") + ", and " + c("build.py") + " assembles them into "
       + c("docs/Technical Manual.html") + " and " + c("docs/User Manual.html") + ", two self-contained "
       "files without scripts, readable even when downloaded on their own. Names, style and structure are those of the "
       "IR_Service manuals, as in AMS; the cover shows the official logo ("
       + c("grafica/phonestra-logo-orizzontale-scuro.png") + ", embedded in the page).", lead=True) + \
    p(c("tests/manuale.rs") + " runs " + c("python3 docs/sorgenti/build.py --controlla") + ", which fails if:") + ul([
        "the published file does not match the sources (the manuals are not edited by hand);",
        "a source file is missing from " + rif("Appendix B — File map") + " or the map lists a file that no longer exists;",
        "a cited Rust symbol (such as " + c("Collegamento::mantieni") + ") does not exist in the sources, or a "
        "cited Java method (such as " + c("Servizio.comandi") + ") is not in the class's file;",
        "a cited file does not exist in the repository;",
        "a " + c("PHONESTRA_*") + " variable in the code is not documented, or the manuals cite one that does not exist;",
        "a command of " + c("phonestra-prova") + " or of the helper is not documented;",
        "an internal link leads to a section that does not exist, or the version from " + c("Cargo.toml") + " does not appear; "
        "or the running text of either manual contains Italian sentences (outside code, interface labels and program output).",
    ]) + note("the checks find names that have disappeared, not behavior that has changed: when you change the way something "
              "works, look in the manuals for the section that describes it and update it in the same commit.",
              "What the checks do not see.")

CHAPTER = ("Testing and diagnostics", [
    ("Tests on the PC", S1),
    ("The phonestra-prova tool", S2),
    ("Rules for tests on the phone", S3),
    ("Diagnostic variables", S4),
    ("The manuals and their checks", S5),
])
