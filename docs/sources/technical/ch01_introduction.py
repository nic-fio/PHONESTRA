from build import VERSION, arrow, box, c, conta, dl, fig, note, numeri, p, righe, rif, table, tree, zone

INSIEME = fig(
    zone(20, 12, 300, 214, "Linux PC (Rust, GTK4)")
    + box(40, 42, 260, 50, "Drawer", "apps, notifications, phone screen", "navy")
    + box(40, 102, 260, 50, "App window", "video, mouse, keyboard", "blue")
    + box(40, 162, 260, 50, "App window", "one for each open app", "blue")
    + box(362, 102, 176, 50, "ADB over Wi-Fi", "one encrypted connection", "dark")
    + arrow(322, 127, 360, 127) + arrow(540, 127, 578, 127)
    + zone(580, 12, 300, 214, "Android 14+ phone (Java)")
    + box(600, 42, 260, 50, "Mirror", "the main screen", "light")
    + box(600, 102, 260, 50, "Virtual display", "the window's app", "light")
    + box(600, 162, 260, 50, "Virtual display", "another app", "light"),
    900, 240, "«FIG» — Phonestra: each PC window shows one phone screen, all over a single connection")

S1 = p("<b>Phonestra</b> brings the apps of an Android phone to a Linux PC, each in its own window, over Wi-Fi and "
       "without installing anything on the phone. " + c("SPECIFICATION.md") + " says what it does; this manual explains how.",
       lead=True) + \
    p("Each app window is a virtual display on the phone, as large as the window; the drawer, the main window, also "
      "shows the phone's real screen. Video, audio, touches, keys and clipboard go through a component written from "
      "scratch, copied to the phone at every connection and deleted at the end.") + INSIEME + \
    p("After reading this manual you should know where to work in every file, how to build the program and the "
      "phone component, build the AppImage, run the tests, and add a message or a channel without breaking the "
      "rest.") + \
    p("The manual assumes you know Rust and a little Java. Android and ADB concepts are explained where they are "
      "needed; the " + rif("Glossary") + " sums them up. The reason behind every choice is in the " + c("notes/")
      + " folder: here you will find the right reference when it matters (for example, “prove §49” is section 49 of "
      + c("notes/connection-tests.md") + ").") + \
    table(["Item", "Value"], [
        ["Version", c(VERSION)],
        ["Languages", "Rust 2024 on the PC, Java on the phone"],
        ["Size", f"{righe(conta('.rs'))} lines of Rust, {righe(conta('.java'))} of Java"],
        ["Target", "Linux x86_64 (AppImage) · Android 14 and later"],
    ], "«TAB» — Phonestra at a glance")

S2 = p("A few concepts recur throughout the manual. This table introduces each of them in one line, with the chapter "
       "that covers it in depth.", lead=True) + \
    table(["Concept", "What it is", "Where", "More"], [
        ["<b>Connection</b>", "The active phone and its ADB connection, kept up as long as Phonestra is open; "
         "it starts over at every drop", c("collegamento.rs"), rif("Life of a connection")],
        ["<b>ADB client</b>", "The ADB protocol written by us: all channels over a single encrypted TCP connection",
         c("src/adb/"), rif("The ADB client")],
        ["<b>Component</b>", "The Java jar copied to the phone; the <i>service</i> is its long-running process, "
         "one per connection", c("android/helper/") + ", " + c("componente.rs"), rif("The on-phone component")],
        ["<b>Video session</b>", "A virtual display for an app window, or the mirror of the main screen for the "
         "drawer, with its " + c("video:<id>") + " channel", c("video_nostro/"),
         rif("Sessions: virtual display and mirror")],
        ["<b>Guardian</b>", "A shell process that restores the phone when whatever it is tied to ends",
         c("collegamento.rs") + ", " + c("Custode.java"), rif("The two guardians")],
        ["<b>Panel “in hand”</b>", "The physical screen turns off while the phone is used from the PC and turns back "
         "on when the user picks the phone up", c("Pannello.java") + ", " + c("collegamento.rs"),
         rif("The phone in hand")],
        ["<b>Drawer</b>", "The main window: apps, notifications, phones, tools and the drawn phone "
         "screen", c("cassetto.rs"), rif("The drawer")],
        ["<b>Helper</b>", "The same jar, used for short commands that print and exit (app list, wallpaper, "
         "thumbnails)", c("app.rs") + ", " + c("Aiuto.java"), rif("What the component is")],
    ], "«TAB» — Basic concepts") + \
    note("Phonestra uses one phone at a time; the other configured phones wait in the drawer's sidebar "
         "(" + rif("Multiple phones") + ").", "One phone at a time.")

S3 = p("The code is split into cohesive parts. This is the map that the manual explores chapter by chapter; the "
       "line counts of every file are in " + rif("Appendix B — File map") + ".", lead=True) + dl([
    (c("src/bin/phonestra.rs") + ", " + c("src/collegamento.rs"), c("main") + ", the life of the connection, the "
     "3-second round, shutdown, data on the PC (" + rif("Startup and life cycle") + ")."),
    (c("src/adb/") + ", " + c("src/rete.rs"), "The ADB client: messages, TLS, channels, " + c("shell,v2") + ", "
     + c("sync:") + ", pairing with the code, mDNS discovery (" + rif("The ADB client") + ")."),
    (c("android/helper/src/phonestra/") + ", " + c("src/componente.rs"), "The component: service, channels, command "
     "channel, heartbeat, guardians, self-test, and its PC side (" + rif("The on-phone component") + ")."),
    (c("src/video_nostro/") + ", " + c("SessioneVideo.java") + ", " + c("Codifica.java"), "Virtual displays, mirror, "
     "H.264 encoder, resizing, app events (" + rif("Video") + ")."),
    (c("Pannello.java"), "The physical screen turned off during use from the PC, calls, drops ("
     + rif("The phone's panel") + ")."),
    (c("src/audio_nostro.rs") + ", " + c("CanaleAudio.java"), "Loopback capture, AAC, playback and recording ("
     + rif("Audio") + ")."),
    (c("src/input_nostro.rs") + ", " + c("src/appunti.rs") + ", " + c("Input.java") + ", " + c("Appunti.java"),
     "Touches, keys, text and clipboard in both directions (" + rif("Input and clipboard") + ")."),
    (c("src/cassetto.rs") + ", " + c("src/finestra.rs") + ", " + c("src/prepara.rs") + ", " + c("src/procedura.rs")
     + ", " + c("src/avvisi.rs"), "Drawer, app windows, preferences, first connection, notifications ("
     + rif("The user interface") + ")."),
    (c("src/azioni.rs") + ", " + c("src/ricevi.rs"), "Installing apps, sending and receiving files (" + rif("Apps and files")
     + ")."),
    (c("src/bin/prova.rs") + ", " + c("tests/"), "Tests on the PC and on the phone (" + rif("Testing and diagnostics") + ")."),
    (c("packaging/") + ", " + c("android/helper/build.sh"), "The compiled component, the AppImage, releases ("
     + rif("Build and release") + ")."),
])

S4 = p("The " + c("nic-fio/PHONESTRA") + " repository is the only complete copy of the project: code, compiled "
       "component, documents, mockups and measurements.", lead=True) + tree([
    "PHONESTRA/",
    "├── Cargo.toml  # the phonestra package: one library and two executables",
    "├── Cargo.lock  # the exact versions of the Rust crates",
    "├── Makefile  # make, make test, make clippy, make docs, make docs-check, make dist, make helper, make clean",
    "├── src/  # the PC program",
    "│   ├── adb/  # the ADB client",
    "│   ├── video_nostro/  # PC side of the component's video",
    "│   └── bin/  # phonestra.rs (the program) and prova.rs (phonestra-prova)",
    "├── android/",
    "│   ├── helper/src/phonestra/  # the phone component, in Java",
    "│   ├── helper/stub/  # fake Android classes, for the compiler only",
    "│   ├── helper/build.sh  # rebuilds the jar (make helper)",
    "│   ├── phonestra-helper.jar  # the compiled component, embedded in the executable",
    "│   └── README.md  # what runs on the phone and how it is built",
    "├── data/instructions.toml  # per-brand instructions for the cable procedure",
    "├── data/en/  # English translations of the interface, one table per module",
    "├── vendor/rusb/  # rusb without the built-in libusb (dynamic linking, vendor/README.md)",
    "├── packaging/  # container, scripts and AppRun of the AppImage, third-party licenses",
    "├── docs/  # the two manuals and their sources (docs/sources/), index.html, README.md",
    "├── notes/  # decisions, studies, measurements, issue log",
    "├── tests/  # integration tests (tests/manual.rs)",
    "├── tools/  # setup-dev.sh (what a fresh clone needs), backup.sh (the whole project as a git bundle)",
    "├── logos/  # the Phonestra logo: icons/, icons-with-text/",
    "├── mockup/  # design proposals, icons and the sources of the interface canvas",
    "├── site/  # the website phonestra.nicfio.it: landing page, publish.sh",
    "├── experiments/  # small test scripts outside the program",
    "├── .github/workflows/ci.yml  # CI at every push: make all, make test, make docs-check",
    "├── README.md  # the project in brief",
    "├── CLAUDE.md  # working rules for Claude Code",
    "├── SPECIFICATION.md  # what Phonestra does; the code's “§” references point here",
    "├── NOTICE.md  # copyright and third-party components with their licenses",
    "├── LICENSE.md  # Phonestra Freeware Licence, from the version after 1.0.0-rc.8",
    "└── LICENSE-1.0.0-rc.8-and-earlier.md  # free personal use, up to 1.0.0-rc.8",
], "«FIG» — The repository's folders") + \
    table(["Path", "Contents"], [
        [c("src/"), "The PC program: interface, connection, ADB client (" + c("src/adb/")
         + "), PC side of the component (" + c("componente.rs") + ", " + c("audio_nostro.rs") + ", "
         + c("video_nostro/") + ", " + c("input_nostro.rs") + ")."],
        [c("android/helper/"), "The phone component: Java sources in " + c("src/phonestra/")
         + ", fake Android classes for the compiler in " + c("stub/") + ", " + c("build.sh") + "."],
        [c("android/phonestra-helper.jar"), "The compiled component (dex inside a jar). It is in the repository and is "
         "embedded in the executable."],
        [c("data/instructions.toml"), "Instructions for each brand family for the cable procedure, embedded at "
         "build time (format in " + rif("The first connection") + ")."],
        [c("docs/"), c("Technical Manual.html") + " and " + c("User Manual.html") + " (generated, "
         "never by hand) and their sources in " + c("docs/sources/") + ": " + c("build.py") + ", " + c("style.css") + ", " + c("manual.js")
         + ", one file per chapter in " + c("technical/") + " and " + c("user/") + "."],
        [c("packaging/"), c("Containerfile") + ", " + c("build.sh") + ", " + c("collect.sh") + ", "
         + c("rust-licenses.py") + ", " + c("test-distributions.sh") + " and " + c("AppRun") + ": the AppImage "
         "and the third-party licenses it carries (" + rif("Build and release") + ")."],
        [c("notes/"), "Decisions, studies, measurements, issue log: the reasons behind the project."],
        [c("tests/"), "Integration tests; " + c("tests/manual.rs") + " checks the manuals."],
        [c("tools/"), c("setup-dev.sh") + " says what a fresh clone lacks (packages, Rust, git identity) and, with "
         + c("--install") + ", installs the packages; "
         + c("backup.sh") + " saves the repository as a git bundle."],
        [c("logos/"), "The Phonestra logo in several versions, with the icons (" + c("icons/") + ") and the icons "
         "with the name (" + c("icons-with-text/") + ")."],
        [c("mockup/"), "Interface proposals, icons and the sources of the design canvas."],
        [c("Makefile"), "The usual commands: " + c("make") + ", " + c("make test") + ", " + c("make clippy") + ", "
         + c("make docs") + ", " + c("make docs-check") + ", " + c("make dist") + " (AppImage), " + c("make helper")
         + " (jar), " + c("make clean") + "."],
        [c(".github/workflows/ci.yml"), "The CI on GitHub: at every push and pull request it builds, runs "
         + c("make test") + " and " + c("make docs-check") + ", without a phone."],
        [c("SPECIFICATION.md"), "What Phonestra does, section by section; the code's “§” numbers point here."],
        [c("NOTICE.md"), "Copyright, the third-party components (Rust crates, the AppImage's libraries) and their "
         "licenses, where their texts are in the AppImage and where to get their sources."],
        [c("LICENSE.md"), "The Phonestra Freeware Licence, from the first version after 1.0.0-rc.8: free use, also at "
         "work, and free redistribution of the unchanged AppImage; no sale, no inclusion in commercial products, no "
         "modification. The licence of the versions up to 1.0.0-rc.8 (free personal use) stays in "
         + c("LICENSE-1.0.0-rc.8-and-earlier.md") + "."],
        [c("site/"), "The website, " + c("https://phonestra.nicfio.it") + ": the landing page (" + c("landing/")
         + "), " + c("publish.sh") + " that builds it with the manuals, the licence page and the download, and "
         "copies it to the server."],
    ], "«TAB» — What is in the repository")

PARTI = [
    ("Tests and measurements (PC)", c("src/bin/prova*") + ", " + c("prova_input.rs") + ", " + c("misura_audio.rs") + ", "
     + c("video_nostro/prova.rs") + ", " + c("adb/misura.rs") + ", " + c("adb/prove.rs") + ", " + c("tests/"),
     "the " + c("phonestra-prova") + " tool, the study's measurements, the integration tests",
     lambda f: f.startswith(("src/bin/prova", "tests/")) or f in (
         "src/prova_input.rs", "src/misura_audio.rs", "src/video_nostro/prova.rs", "src/adb/misura.rs", "src/adb/prove.rs")),
    ("Tests and measurements (phone)", c("VideoProva.java") + ", " + c("InputProva.java") + ", " + c("Codificatori.java"),
     "measurement tools and test commands of the component",
     lambda f: f.endswith(("/VideoProva.java", "/InputProva.java", "/Codificatori.java"))),
    ("Interface", c("cassetto.rs") + ", " + c("finestra.rs") + ", " + c("ricevi.rs") + ", " + c("prepara.rs") + ", "
     + c("procedura.rs") + ", " + c("avvisi.rs") + ", " + c("foto.rs") + ", " + c("bin/phonestra.rs"),
     "drawer, app windows, receiving files, first connection, alerts",
     lambda f: f in ("src/cassetto.rs", "src/finestra.rs", "src/ricevi.rs", "src/prepara.rs", "src/procedura.rs",
                     "src/avvisi.rs", "src/foto.rs", "src/bin/phonestra.rs")),
    ("ADB client", c("src/adb/"), "messages, TLS, channels, " + c("shell,v2") + ", " + c("sync:") + ", pairing",
     lambda f: f.startswith("src/adb/")),
    ("Component, PC side", c("componente.rs") + ", " + c("audio_nostro.rs") + ", " + c("video_nostro/") + ", "
     + c("input_nostro.rs") + ", " + c("appunti.rs"), "service, dispatching, audio, video, input, clipboard",
     lambda f: f in ("src/componente.rs", "src/audio_nostro.rs", "src/input_nostro.rs", "src/appunti.rs")
     or f.startswith("src/video_nostro/")),
    ("Connection and data", c("collegamento.rs") + ", " + c("rete.rs") + ", " + c("configurazione.rs") + ", "
     + c("telefono.rs") + ", " + c("usb.rs") + ", " + c("notifiche.rs") + ", " + c("azioni.rs") + ", " + c("app.rs")
     + ", " + c("lib.rs"), "connection, panel, mDNS, configuration, cable, notifications, actions, app list",
     lambda f: f.startswith("src/")),
    ("On-phone component", c("android/helper/src/phonestra/"),
     "service, guardian, audio, video, input, clipboard, panel, thumbnails",
     lambda f: f.startswith("android/helper/src/")),
    ("Build and tools", c("packaging/") + ", " + c("android/helper/build.sh") + ", " + c("docs/sources/"),
     "container, AppImage, jar, manual generator", lambda f: True),
]

S5 = p("How big Phonestra is, part by part. The lines are counted from the sources every time the manual is generated.",
       lead=True) + numeri(PARTI, "«TAB» — The project's parts and their line counts") + \
    p("The numbers in this table and in " + rif("Appendix B — File map") + " are not written by hand, and "
      + c("cargo test") + " fails if the published manual no longer matches the sources ("
      + rif("The manuals and their checks") + ").")

CHAPTER = ("Introduction and concepts", [
    ("What Phonestra is", S1),
    ("Basic concepts", S2),
    ("Subsystems at a glance", S3),
    ("The repository", S4),
    ("The project in numbers", S5),
])
