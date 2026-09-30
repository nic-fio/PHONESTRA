import re

from build import HERE, c, file_map, p

J = "telefono/aiuto/src/phonestra/"


def capitoli():
    """Una riga per ogni capitolo dei due manuali, col suo titolo."""
    righe = []
    for cartella, nome in (("tecnico", "Technical Manual"), ("utente", "User Manual")):
        for f in sorted((HERE / cartella).glob("ch[0-9][0-9]_*.py")):
            titolo = re.search(r'^CHAPTER = \("([^"]+)"', f.read_text(), re.M).group(1)
            righe.append((f"docs/sorgenti/{cartella}/{f.name}", f"{nome}, chapter {int(f.name[2:4])}: {titolo}"))
    return righe


GRUPPI = [
    ("User interface", [
        ("src/avvisi.rs", "Phone notifications as system notifications (D-Bus), app icons"),
        ("src/bin/phonestra.rs", c("main") + ": one process for all windows, connection, drawer, PC "
         "clipboard, Ctrl+C and SIGTERM, shutdown and restart to switch phones"),
        ("src/cassetto.rs", "The drawer: phone pill, App, Notifiche and Preferenze pages, phones, tools, "
         "phone drawn with the mirror, transfers"),
        ("src/finestra.rs", "An app's window: view (video, mouse, keyboard, zoom), sessions, panel, "
         "recording, screenshot"),
        ("src/foto.rs", c("PHONESTRA_FOTO") + ": window images for user interface tests"),
        ("src/prepara.rs", "“Aggiungi un telefono” without a cable: settings list, checkmarks from mDNS, 6-digit code"),
        ("src/procedura.rs", "The fallback cable procedure, by brand family"),
        ("src/ricevi.rs", "“Ricevi file…”: places, Recents, listing and thumbnails of the phone's folders, file selection"),
    ]),
    ("Connection and data", [
        ("src/app.rs", "The embedded jar (" + c("AIUTO") + "), the helper, app list with the fine row, "
         "wallpaper, thumbnails, encoders"),
        ("src/azioni.rs", "Installing, uninstalling, sending files"),
        ("src/collegamento.rs", "The active phone: discovery, connection, connection guardian, component, 3-second "
         "round, panel “in hand”, calls, shutdown"),
        ("src/configurazione.rs", "XDG folders, ADB key, " + c("telefoni.toml") + ", " + c("preferenze.toml")),
        ("src/lib.rs", "The library modules, " + c("esecutore()") + ", diagnostics"),
        ("src/notifiche.rs", "Reading notifications, battery and network from " + c("dumpsys")),
        ("src/rete.rs", "Hand-written mDNS: phones with Wireless debugging, the code screen"),
        ("src/telefono.rs", "Cable or Wi-Fi connection with " + c("adb_client") + ", for the fallback procedure and "
         "the first connection"),
        ("src/usb.rs", "Cable-connected phones read from sysfs, without permissions"),
    ]),
    ("ADB client", [
        ("src/adb/abbina.rs", "Pairing with the code: TLS, SPAKE2, HKDF, AES-GCM"),
        ("src/adb/flusso.rs", "Pure parts of flow control: banner, OPEN, OKAY, delayed ack balance"),
        ("src/adb/messaggio.rs", "ADB messages: 24-byte header, reading and writing"),
        ("src/adb/mod.rs", c("Adb") + ", " + c("Canale") + ", " + c("Trasporto") + ": Wi-Fi connection, "
         "dispatching, flow control"),
        ("src/adb/shell.rs", "The " + c("shell,v2") + " service: input, output, errors, exit code"),
        ("src/adb/sync.rs", "The " + c("sync:") + " protocol: copying files to the phone, listing its folders, "
         "receiving files"),
        ("src/adb/tls.rs", "Wireless debugging TLS: client certificate from Phonestra's key"),
    ]),
    ("Component, PC side", [
        ("src/appunti.rs", "Copies made on the phone → PC clipboard, with limits and bounce-backs"),
        ("src/audio_nostro.rs", "Audio channel, packets, timestamps, margin, GStreamer playback, copies for recording"),
        ("src/componente.rs", c("Componente") + " and " + c("Condiviso") + ": startup, preamble, " + c("CIAO")
         + ", heartbeat, dispatching, leftovers"),
        ("src/input_nostro.rs", c("InputNostro") + ": encoding of touches, scroll wheel, keys, text, clipboard"),
        ("src/video_nostro/flusso.rs", "Screen options and reading of video packets"),
        ("src/video_nostro/mod.rs", c("SessioneNostra") + ", " + c("ComandiVideo") + ", events, panel"),
    ]),
    ("Component on the phone", [
        (J + "Aiuto.java", "Jar entry point: short helper commands and the service"),
        (J + "Appunti.java", "Direct " + c("IClipboard") + ": reading, writing, sensitive items, listener"),
        (J + "Audio.java", "Audio capture, reading and encoding; measurement tool"),
        (J + "Autotest.java", "Startup check of the hidden APIs, without using them"),
        (J + "CanaleAudio.java", "The audio channel: loopback capture, AAC, four threads"),
        (J + "Codifica.java", "Hardware " + c("MediaCodec") + " from a " + c("Surface") + ", keyframe on demand"),
        (J + "Contesto.java", "Android context for " + c("app_process") + ", package " + c("com.android.shell")),
        (J + "Custode.java", "The service guardian: " + c("sh") + " script, list of actions"),
        (J + "EventiApp.java", c("TaskStackListener") + ": orientation, protected screen, tasks moved or closed"),
        (J + "Input.java", "Input messages, queue, injection, fingers, scaling"),
        (J + "Miniature.java", "Photo and video thumbnails for “Ricevi file…” (downsampled " + c("BitmapFactory") + ", EXIF, "
         "video frame)"),
        (J + "Nascoste.java", "Adapters for the hidden APIs, via reflection"),
        (J + "Pannello.java", "Physical panel on or off, refresh rate at 60 Hz before turning off, restore via the guardian"),
        (J + "Protetta.java", "Protected screen with " + c("captureDisplay") + " and " + c("containsSecureLayers")),
        (J + "Protocollo.java", "Message format of the command channel and the preamble"),
        (J + "Pulizia.java", "Teardown in reverse order at the end of a test"),
        (J + "Servizio.java", "The service: secret, socket, channels, heartbeat, exit codes, " + c("TIPI")),
        (J + "SessioneVideo.java", "Virtual display or mirror, encoder, resizing, forced redraw"),
        (J + "Sistema.java", "Shared pieces: display manager for virtual displays, app launching, " + c("togliTask")
         + ", internal commands, images"),
        (J + "Video.java", "Video messages, session registry, " + c("video:<id>") + " channel"),
    ]),
    ("Tests and measurements (PC)", [
        ("src/adb/misura.rs", c("phonestra-prova throughput") + ": transport speed and latency"),
        ("src/adb/prove.rs", "The ADB client against a fake in-memory adbd"),
        ("src/bin/prova/audio_componente.rs", c("phonestra-prova audio-componente")),
        ("src/bin/prova.rs", c("phonestra-prova") + ": the command-line test commands"),
        ("src/misura_audio.rs", "Analysis of the studio audio: measurement lines, levels, WAV"),
        ("src/prova_input.rs", c("phonestra-prova input-componente") + ": clipboard, touches, text"),
        ("src/video_nostro/prova.rs", c("phonestra-prova video-componente")),
        ("tests/manuale.rs", "This manual kept in step with the sources (runs " + c("build.py --controlla") + ")"),
    ]),
    ("Tests and measurements (phone)", [
        (J + "Codificatori.java", "List of the phone's audio and video encoders"),
        (J + "InputProva.java", "Input test commands: test screen, signature, clipboard"),
        (J + "VideoProva.java", "Measurement tool for the studio video"),
    ]),
    ("Build and tools", [
        ("costruzione/AppRun", "AppImage launcher: GTK and GStreamer variables, fallback libraries"),
        ("costruzione/Containerfile", "Ubuntu 22.04 container with GTK 4.14, libadwaita 1.5, " + c("gst-plugin-gtk4")),
        ("costruzione/compila.sh", "Builds a library from a tarball with meson, inside the container"),
        ("costruzione/prova-distribuzioni.sh", "The AppImage on Ubuntu, Debian, Fedora and Arch in containers"),
        ("costruzione/raccogli.sh", "Gathers the executable and libraries into the AppDir and creates the AppImage"),
        ("telefono/aiuto/costruisci.sh", "Builds the component: " + c("javac") + ", D8, " + c("phonestra-aiuto.jar")),
        ("docs/sorgenti/build.py", "Generates the two manuals: functions for text, tables and SVG figures, numbering, checks"),
    ] + capitoli()),
]

S1 = p("Every source file with its line count, recounted each time the manual is generated. If a file is missing from "
       "the map, or the map lists a file that no longer exists, generation stops and " + c("cargo test") + " fails.",
       lead=True) + file_map(GRUPPI, "«TAB» — The project's files")

CHAPTER = ("Appendix B — File map", [
    ("The project's files", S1),
])
