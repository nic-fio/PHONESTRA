from build import arrow, box, c, fig, note, p, rif, seq, steps, table, zone

S1 = p("Phonestra speaks the ADB protocol on its own, in " + c("src/adb/") + ": no " + c("adb") + " server, no "
       "programs to install. All channels (commands, audio, the video of every window, shell) travel over a single "
       "encrypted TCP connection.", lead=True) + \
    p("The " + c("adb_client") + " library reads all channels from the same connection without dispatching the messages: "
      "it handles one command at a time, not video, audio and commands together. The client in " + c("src/adb/") + " has a "
      "reader task that dispatches each message to the right channel by local ID, and every channel honors "
      "ADB's flow control.") + \
    p(c("adb_client") + " remains for short commands outside the actual connection: the USB cable ("
      + c("telefono.rs") + ") in the fallback procedure and in " + c("phonestra-prova prepara") + ", and the first "
      "Wi-Fi connection of “Add a phone” (Aggiungi un telefono), which after pairing reads model and version and removes the "
      "expiry from the authorization (" + c("telefono::Collegamento::wifi") + ", " + rif("The first connection") + ").")

S2 = p("ADB is made of a few messages, all with the same shape: a 24-byte little-endian header "
       "(command, " + c("arg0") + ", " + c("arg1") + ", data length, byte sum, command xor "
       + c("0xffffffff") + ") followed by the data (" + c("messaggio.rs") + ").", lead=True) + table(["Command", "Meaning", "Use in Phonestra"], [
    [c("CNXN"), "Handshake: version, " + c("max_payload") + ", features",
     "Ours announces " + c("host::features=shell_v2,cmd,stat_v2") + " (and " + c("delayed_ack") + " if enabled)"],
    [c("STLS"), "Switch to TLS", "Always, on Wireless debugging"],
    [c("OPEN"), "Opens a channel to a service (" + c("shell,v2,raw:…") + ", " + c("sync:") + ", "
     + c("localabstract:…") + ")", c("Adb::apri") + ", with a 10 s timeout for the reply"],
    [c("OKAY"), "Channel accepted, or data acknowledged", "Flow control"],
    [c("WRTE"), "Data on a channel", c("Canale::scrivi") + " / " + c("leggi")],
    [c("CLSE"), "Closing a channel", c("Canale::chiudi") + ", " + c("Chiusore")],
], "«TAB» — ADB messages")

TLS = seq([("Phonestra", "Adb::wifi", "navy"), ("adbd", "Wireless debugging", "dark")], [
    (0, 1, "TCP (5 s, TCP_NODELAY)"),
    (0, 1, "CNXN in clear: features and max_payload"),
    (1, 0, "STLS", True),
    (0, 1, "STLS, then TLS with Phonestra's certificate"),
    (1, 0, "CNXN after TLS: common max_payload, phone's features", True),
], "«FIG» — Opening the Wi-Fi connection", width=760)

S3 = p("On Wireless debugging the connection starts in clear and switches to TLS right away. " + c("Adb::wifi") + " opens it "
       "in four steps; the phone recognizes the PC by its authorized public key.", lead=True) + TLS + steps([
    "TCP to the address found via mDNS (5 s timeout, " + c("TCP_NODELAY") + ").",
    c("CNXN") + " in clear: the phone reads our features and the " + c("max_payload") + " from here, not from the "
    + c("CNXN") + " after TLS.",
    "The phone replies " + c("STLS") + "; we reply " + c("STLS") + " and TLS starts (" + c("tls.rs") + "). "
    "The client certificate is self-signed with Phonestra's RSA key (" + c("~/.config/Phonestra/adbkey")
    + "): the phone recognizes the PC by the authorized public key. The phone's certificate is not checked "
    "against an authority (it is self-signed): pairing guarantees the identity.",
    "After TLS comes the phone's " + c("CNXN") + ": its " + c("arg1") + " is the minimum of the two "
    + c("max_payload") + " values, and its text contains the phone's features.",
])

MUX = fig(
    box(20, 34, 190, 50, "Command channel", "the component", "blue")
    + box(245, 34, 190, 50, "Audio channel", "one at a time", "blue")
    + box(470, 34, 190, 50, "video:<id> channels", "one per session", "blue")
    + box(690, 34, 190, 50, "exec: and shell,v2", "short commands, service", "blue")
    + "".join(arrow(x - 14, 86, x - 14, 122) + arrow(x + 14, 122, x + 14, 88, "#475569", True)
              for x in (115, 340, 565, 785))
    + zone(20, 124, 860, 94, "Adb — cloned and passed around everywhere")
    + box(50, 152, 250, 52, "Adb::invia", "writes, one at a time (Mutex)", "navy")
    + box(328, 152, 250, 52, "leggi_sempre", "reads and dispatches by ID", "navy")
    + box(606, 152, 250, 52, "Posta", "the OKAYs that cannot wait", "navy")
    + arrow(452, 220, 452, 250)
    + box(262, 252, 380, 46, "One TCP + TLS connection", "", "dark")
    + arrow(644, 275, 698, 275)
    + box(700, 252, 180, 46, "adbd", "Wireless debugging", "dark"),
    900, 310, "«FIG» — Many channels over one connection: solid arrows are writes, dashed arrows the dispatched data")

S4 = p("All channels of a connection share a single underlying connection. Each channel has its own local "
       "ID; the reader dispatches, writers wait their turn.", lead=True) + MUX + \
    p("An " + c("Adb") + " is cloned and passed around everywhere. One task reads from the socket and dispatches ("
       + c("leggi_sempre") + "); writes go directly to the socket, one at a time under a " + c("Mutex")
       + " (" + c("Adb::invia") + "). A second task, the “posta” (mailbox), sends only the acknowledgments that cannot wait "
       "for the writers (the " + c("OKAY") + " messages acknowledging reads).") + \
    p(c("Adb::apri(servizio)") + " returns a " + c("Canale") + " with " + c("scrivi") + ", " + c("leggi")
      + " (cancelable), " + c("leggi_esatti") + ", " + c("leggi_tutto") + " and " + c("chiudi") + "; "
      + c("Canale::chiusore()") + " returns an object that closes the channel from another task. " + c("Adb::esegui")
      + " is the shortcut for a short command (the " + c("exec:") + " service) that returns the output.") + \
    p("Without " + c("delayed ack") + " each channel has only one " + c("WRTE") + " in flight: the next one leaves after "
      "the " + c("OKAY") + ". The transport parameters are in " + c("Trasporto") + ":") + \
    table(["Value", "Default", "Test variable", "Why"], [
        [c("delayed_ack"), "off", c("PHONESTRA_ADB_DELAYED_ACK=1"), "On 28 Sep 2026 adbd rejected every "
         + c("OPEN") + " when it was announced (" + rif("Appendix C — Known issues") + ")."],
        [c("max_payload"), "64 KiB", c("PHONESTRA_ADB_PAYLOAD=1m"), "A single connection for all channels: a 1 MiB "
         + c("WRTE") + " holds the wire for ~200 ms and audio waits behind video; a 64 KiB one for ~13 ms. "
         "With 64 KiB audio stays in sync (misure §50)."],
        ["window", "256 KiB", c("PHONESTRA_ADB_FINESTRA=512k"), "Matters only with " + c("delayed ack") + ": bytes "
         "in flight per channel."],
    ], "«TAB» — Transport parameters") + \
    p("The details of " + c("delayed ack") + " (balance, 4-byte " + c("OKAY") + ", adbd's behavior) are in "
      + c("notes/adb.md") + " and in the pure parts of " + c("flusso.rs") + ", tested by " + c("adb/prove.rs")
      + " against a fake in-memory adbd.")

S5 = p(c("shell.rs") + " starts a process without a terminal, with input, output, errors and exit code "
       "kept separate. Packets " + c("id u8 · lunghezza u32 LE · dati") + " travel on the channel.", lead=True) + \
    table(["id", "Direction", "Content"], [
        ["0", "PC → phone", "Process input"],
        ["1", "phone → PC", "Output"],
        ["2", "phone → PC", "Error output"],
        ["3", "phone → PC", "Exit code (1 byte)"],
        ["4", "PC → phone", "Closing of the input"],
    ], "«TAB» — The packets of " + c("shell,v2")) + \
    p("This is how the Phonestra service starts: the output carries the ready line without mixing with the logs, "
      "the input carries the secret, the exit code tells why the service ended, and if the channel drops adbd "
      "sends SIGHUP to the process. For short commands " + c("Adb::esegui") + " is enough (the " + c("exec:") + " service).")

S6 = p(c("sync.rs") + " implements the " + c("sync:") + " protocol in both directions.", lead=True) + \
    table(["Function", "Requests", "Use"], [
        [c("invia") + ", " + c("invia_a_blocchi"), c("SEND") + " with path and permissions, " + c("DATA")
         + " blocks, " + c("DONE"), "copying the component to " + c("/data/local/tmp") + ", the files the user sends to the "
         "phone and the " + c(".apk") + " files to install (with progress and cancellation)"],
        [c("elenca"), c("STA2") + ", then " + c("LIS2"), "listing a phone folder for “Receive files…” (Ricevi file…): 72-byte "
         + c("DNT2") + " entries after the ID, 64-bit sizes"],
        [c("e_cartella"), c("STA2"), "finding out whether a path exists and is a folder (the places of “Receive files…”, Ricevi file…)"],
        [c("ricevi"), c("RECV"), "copying a file to the PC by writing the " + c("DATA") + " blocks straight into the "
         "file, without holding it in memory"],
    ], "«TAB» — The functions of " + c("sync.rs")) + \
    note("in shared storage (FUSE) the listing does not contain “.” and “..”: an empty folder and one that does not exist "
         "would give the same answer, which is why " + c("STA2") + " is asked first.")

S7 = p(c("abbina.rs") + " reimplements Android 11+'s " + c("adb pair") + " from the Android and BoringSSL sources: the "
       "phone's “Pair device with pairing code” (Associa dispositivo con codice di associazione) screen shows 6 digits, and the user types them into "
       "Phonestra.", lead=True) + steps([
    "TLS directly on the pairing port, with Phonestra's certificate.",
    "Password = the 6 digits + 64 bytes exported from TLS (label " + c("adb-label\\0") + "): nobody can get in the middle.",
    "SPAKE2 over Ed25519 (" + c("curve25519-dalek") + "), with Phonestra in the “alice” role.",
    "From the shared key, with HKDF-SHA256, an AES-128-GCM key (" + c("ring") + ").",
    "Encrypted exchange of the " + c("PeerInfo") + ": we send Phonestra's ADB public key, the phone sends its ID.",
]) + p("From then on the phone accepts Phonestra's key in Wi-Fi connections, as after an “Always allow” "
       "(Consenti sempre) over the cable.")

S8 = \
    p(c("rete.rs") + " builds and parses DNS packets by hand: a PTR query for " + c("_adb-tls-connect._tcp.local")
      + " (connection) or " + c("_adb-tls-pairing._tcp.local") + " (the code screen) with the QU bit, i.e. reply "
      "sent directly to our socket. It is the method that found the phone when the discovery of " + c("adb") + " "
      "saw nothing.", lead=True) + \
    table(["Rule", "Why"], [
        ["The instance is named " + c("adb-<seriale>-<suffisso>"), "The name identifies the saved phone."],
        ["The port comes from the SRV record", "It changes every time Wireless debugging restarts."],
        ["The query is repeated every second", "Some phones only answer the second one."],
        ["An SRV with TTL 0 removes the found phone", "It is the “goodbye” of a service that is shutting down."],
        [c("indirizzo_attivo") + ": last good address (800 ms), then up to three 3 s searches",
         "The common case is fast; an address is accepted only if the port really answers."],
    ], "«TAB» — How the phone is found on the network")

CHAPTER = ("The ADB client", [
    ("Why our own client", S1),
    ("ADB messages", S2),
    ("Wi-Fi connection and TLS", S3),
    ("Channels and flow control", S4),
    ("The shell,v2 service", S5),
    ("Copying files: sync:", S6),
    ("Pairing with the code", S7),
    ("Network discovery: mDNS", S8),
])
