# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import c, note, p, rif, table

S1 = p("This appendix collects the central data structures of the PC program, with the file that defines them and "
       "the section that covers them in depth. They are the contracts that run through Phonestra; the Java classes of the component "
       "are listed in " + rif("Appendix B — File map") + ".", lead=True) + \
    table(["Structure", "Defined in", "Role", "Section"], [
        "Connection and data",
        [c("Collegamento"), c("collegamento.rs"), "The active phone: state, " + c("Adb") + ", component, “in hand” "
         "panel, notifications", rif("Life of a connection")],
        [c("Stato"), c("collegamento.rs"), c("Cerco") + ", " + c("Collegato") + ", " + c("Bloccato") + ", "
         + c("Perso") + ", " + c("Chiuso"), rif("Life of a connection")],
        [c("Telefoni") + ", " + c("Telefono"), c("configurazione.rs"), "The configured phones, in "
         + c("telefoni.toml"), rif("Configuration and data on the PC")],
        [c("Preferenze"), c("configurazione.rs"), "The user's choices, in " + c("preferenze.toml"),
         rif("Preferences")],
        [c("Notifica") + ", " + c("Info"), c("notifiche.rs"), "Notifications, battery and network read from " + c("dumpsys"),
         rif("Notifications and alerts")],
        [c("App"), c("app.rs"), "A launcher app: package, activity, name, icon", rif("The drawer")],
        "ADB client",
        [c("Adb"), c("adb/mod.rs"), "The connection: can be cloned, opens channels, runs short commands",
         rif("Channels and flow control")],
        [c("Canale") + ", " + c("Chiusore"), c("adb/mod.rs"), "A channel to a service; closing it from "
         "another task", rif("Channels and flow control")],
        [c("Trasporto"), c("adb/mod.rs"), c("delayed_ack") + ", " + c("max_payload") + ", window",
         rif("Channels and flow control")],
        [c("Messaggio"), c("adb/messaggio.rs"), "An ADB message: 24-byte header and data",
         rif("ADB messages")],
        [c("ShellV2"), c("adb/shell.rs"), "A process on the " + c("shell,v2") + " channel", rif("The shell,v2 service")],
        [c("Voce"), c("adb/sync.rs"), "An entry in a phone folder: name, folder or file, size, "
         "modification date", rif("Copying files: sync:")],
        "Component, PC side",
        [c("Componente"), c("componente.rs"), "The started service: command channel, requests, shutdown",
         rif("PC side: Componente and Condiviso")],
        [c("Condiviso"), c("componente.rs"), "The component held in a task and cloneable: one service, many "
         "windows", rif("PC side: Componente and Condiviso")],
        [c("Mittente") + ", " + c("Apritore"), c("componente.rs"), "Messages without a reply queued on the command "
         "channel; opening of the " + c("audio") + " and " + c("video:<id>") + " channels", rif("PC side: Componente and Condiviso")],
        [c("Pronto") + ", " + c("Ciao"), c("componente.rs"), "The service's ready line; its " + c("CIAO"),
         rif("Starting the service")],
        [c("Messaggio"), c("componente.rs"), "A command-channel message: type, flags, id, content",
         rif("The command channel")],
        [c("SessioneNostra") + ", " + c("ComandiVideo"), c("video_nostro/mod.rs"), "A video session and its "
         "commands", rif("PC side: from session to window")],
        [c("Evento"), c("video_nostro/mod.rs"), "The events of a session: orientation, protected, moved, "
         "removed, end", rif("App events")],
        [c("Opzioni") + ", " + c("Pacchetto"), c("video_nostro/flusso.rs"), "The display options; the packets "
         "of the video channel", rif("The video:<id> channel")],
        [c("InputNostro"), c("input_nostro.rs"), "Taps, scroll wheel, keys, text and clipboard encoded for the "
         + c("Mittente"), rif("Input and clipboard messages")],
        [c("Durate") + ", " + c("Orari") + ", " + c("Margine"), c("audio_nostro.rs"), "Duration, timestamp and playback "
         "time of each audio packet", rif("Playback on the PC")],
        [c("Vista"), c("finestra.rs"), "Video, mouse and keyboard: the heart of an app window, also used for the "
         "screen in the drawer", rif("App windows")],
    ], "«TAB» — The main data structures")

S2 = p("All the command-channel messages in one table, by number. The format of each is described in the "
       "section indicated.", lead=True) + \
    table(["Type", "Name", "Direction", "Section"], [
        "Infrastructure (0x01–0x0f) and tests (0x10–0x1f)",
        [c("0x01"), c("CIAO"), "service → PC", rif("The command channel")],
        [c("0x02"), c("BATTITO"), "both directions", rif("Heartbeat and exit codes")],
        [c("0x03"), c("FINE"), "PC → service", rif("The command channel")],
        [c("0x04"), c("ERRORE"), "service → PC, as a reply", rif("The command channel")],
        [c("0x10"), c("PROVA_CUSTODE"), "PC → service", rif("The command channel")],
        "Video and panel (0x40–0x4f)",
        [c("0x40") + "–" + c("0x45"), c("VIDEO_APRI") + ", " + c("VIDEO_CHIUDI") + ", " + c("VIDEO_AVVIA_APP") + ", "
         + c("VIDEO_RIDIMENSIONA") + ", " + c("VIDEO_CHIAVE") + ", " + c("VIDEO_PANNELLO"), "PC → service",
         rif("Video messages")],
        [c("0x46"), c("VIDEO_EVENTO"), "service → PC, unsolicited", rif("App events")],
        "Input and clipboard (0x50–0x5f)",
        [c("0x50") + "–" + c("0x54"), c("TOCCHI") + ", " + c("ROTELLINA") + ", " + c("TASTO") + ", " + c("TESTO")
         + ", " + c("INDIETRO"), "PC → service, no reply", rif("Input and clipboard messages")],
        [c("0x55") + "–" + c("0x57"), c("APPUNTI_SCRIVI") + ", " + c("APPUNTI_LEGGI") + ", " + c("APPUNTI_ASCOLTA"),
         "PC → service", rif("Clipboard")],
        [c("0x58"), c("APPUNTI_CAMBIATI"), "service → PC, unsolicited", rif("Clipboard")],
        [c("0x5c") + ", " + c("0x5d"), c("CONTEGGI") + ", " + c("PROVA"), "PC → service (diagnostics and tests)",
         rif("Input and clipboard messages")],
    ], "«TAB» — Index of the command-channel messages") + \
    note("the test " + c("nessun_tipo_usato_da_due_moduli") + " checks that every number is unique, in the right "
         "range and the same in Java and in Rust (" + rif("A new message on the command channel") + ").")

S3 = p("Every byte stream in Phonestra starts with a fixed header. Here they are side by side.", lead=True) + \
    table(["Stream", "Header", "Byte order", "Section"], [
        ["ADB message", "24 bytes: command, " + c("arg0") + ", " + c("arg1") + ", length, checksum, command xor "
         + c("0xffffffff"), "little-endian", rif("ADB messages")],
        [c("shell,v2") + " packet", c("id u8 · lunghezza u32"), "little-endian", rif("The shell,v2 service")],
        ["Preamble of a service channel", "secret (16 bytes) · type length (u8) · type (ASCII)", "—",
         rif("Channels and preamble")],
        ["Command channel", "8 bytes: " + c("tipo u8 · bandiere u8 · id u16 · lunghezza u32"), "big-endian",
         rif("The command channel")],
        [c("video:<id>") + " channel", "12 bytes: " + c("pts u64 · lunghezza u32") + ", or " + c("0x80000000")
         + " and the new size", "big-endian", rif("The video:<id> channel")],
        ["Audio channel", "12 bytes: " + c("orario u64 · lunghezza u32"), "big-endian", rif("Audio packets")],
    ], "«TAB» — Stream headers")

CHAPTER = ("Appendix A — Data structures and messages", [
    ("Index of data structures", S1),
    ("Index of messages", S2),
    ("Stream headers", S3),
])
