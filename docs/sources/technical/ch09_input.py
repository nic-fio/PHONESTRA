from build import c, flow, key, note, p, rif, table

S1 = p("The PC's mouse, touchpad and keyboard become fingers and keys on the app's screen; the clipboard travels in "
       "both directions, text only, never passwords.", lead=True) + \
    p("Range " + c("0x50–0x5f") + ", big-endian. Events have " + c("id") + " 0 and no response: the PC does not "
      "wait for the phone. " + c("larghezza") + "/" + c("altezza") + " are the size of the image on which the PC "
      "computed the coordinates.") + \
    table(["Type", "Name", "Content"], [
        [c("0x50"), c("TOCCHI"), c("display i32 · larghezza u16 · altezza u16 · n u8 · n × (dito i64 · azione u8 · x i32 · y i32 · pressione f32)")],
        [c("0x51"), c("ROTELLINA"), c("display i32 · x i32 · y i32 · larghezza u16 · altezza u16 · orizzontale f32 · verticale f32")],
        [c("0x52"), c("TASTO"), c("display i32 · azione u8 · codice u32 · ripetizione u32 · meta u32")],
        [c("0x53"), c("TESTO"), c("display i32 · testo UTF-8")],
        [c("0x54"), c("INDIETRO"), c("display i32 · azione u8")],
        [c("0x55"), c("APPUNTI_SCRIVI"), c("display i32 · incolla u8 · testo UTF-8") + "; responds if " + c("id")
         + " is not 0"],
        [c("0x56"), c("APPUNTI_LEGGI"), "empty request; response " + c("stato u8 · testo")],
        [c("0x57"), c("APPUNTI_ASCOLTA"), c("attivo u8") + "; responds if " + c("id") + " is not 0"],
        [c("0x58"), c("APPUNTI_CAMBIATI"), "service → PC, unsolicited: " + c("stato u8 · testo")],
        [c("0x5c"), c("CONTEGGI"), "diagnostics: injected, failed, discarded, clipboard notices, "
         + c("ascolto_appunti") + ", last error"],
        [c("0x5d"), c("PROVA"), "test commands (" + c("InputProva.java") + "), on their own thread ("
         + c("input-prova") + "): " + c("apri <l> <a> <dpi>") + ", " + c("avvia <display> <pacchetto>") + ", "
         + c("azione <display> <azione> [pacchetto]") + ", " + c("firma <display>") + " (32×56 luminance), "
         + c("chiudi") + ", " + c("salva-appunti") + ", " + c("ripristina-appunti") + ", " + c("esterno <0|1> <testo>")],
    ], "«TAB» — Input and clipboard messages") + \
    p("A Rust test in " + c("input_nostro.rs") + " reads " + c("Input.java") + " and " + c("Video.java")
      + " and checks that no message number is used twice.")

S2 = p("On the phone, input becomes Android events injected into the right display, in order, by a single "
       "thread.", lead=True) + flow([
    ("App window", "GTK events", "navy"), ("Input encoder", "encoding, sender", "blue"),
    ("Command channel", "no response", "dark"), ("Service", "input receiver", "blue"),
    ("“input” thread", "injectInputEvent", "light")],
    "«FIG» — The path of a touch, from the PC window to the virtual display") + table(["Step", "How"], [
    ["Queue", "The thread that reads commands does not inject; it queues the messages for the “input” thread, a "
     "single one, so order is preserved."],
    ["Event", c("InputEvent.setDisplayId") + " always, then " + c("injectInputEvent") + " asynchronously. A "
     "rejection is counted (" + c("falliti") + "); the log records the first error and then one every 100."],
    ["Fingers", "Each PC identifier (−1 mouse, −2 generic finger, 10 and 11 for pinch) becomes a finger with a "
     "local number 0–9, at most 10. Clicks and drags are fingers (" + c("SOURCE_TOUCHSCREEN") + "): dragging "
     "scrolls, a long press opens menus. Every event carries all the fingers that are down."],
    ["Scaling", "Coordinates × (display size / PC size). An event computed on a size different from the one "
     "declared by the video is discarded: during a resize a click would land in the wrong place. For this "
     "reason the PC rounds sizes to multiples of 8, like the phone."],
    ["Wheel", c("ACTION_SCROLL") + ", " + c("SOURCE_MOUSE") + ", fractional values, within ±16."],
    ["Keys", c("KeyEvent") + " from a virtual keyboard. Text: the virtual key map, one character at a time, in "
     "practice ASCII only; the PC sends the rest (accented letters, symbols) with “paste”."],
    ["Back", c("KEYCODE_BACK") + "; on the main display when it is off, " + c("POWER") + " to turn it back on."],
    ["Paste", "Phone clipboard + " + c("KEYCODE_PASTE") + "."],
], "«TAB» — How the service injects input")

S3 = p(c("finestra.rs") + " translates GTK events (" + c("finestra::tastiera") + " for keys). Modifiers "
       "follow the values of " + c("KeyEvent.META_*") + " (" + c("META_SHIFT") + ", " + c("META_CTRL") + ").",
       lead=True) + \
    table(["On the PC", "On the phone"], [
        ["Click, drag", "Finger"],
        ["Right click", "Long press (selects and opens the Android menu)"],
        ["Wheel, two fingers on the touchpad", "Scrolling at the pointer position"],
        [key("Ctrl") + " + wheel, " + key("Ctrl", "+") + " / " + key("Ctrl", "−") + ", pinch on the touchpad",
         "Two-finger pinch (zoom; with the keys, at the center of the window)"],
        [key("Esc") + ", mouse “back” button, Back button", "Back (" + key("Esc")
         + " can be turned off in the preferences)"],
        [key("↑") + " / " + key("↓"), "One scroll step; real keys while typing (after a letter, "
         + key("Backspace") + " or " + key("Delete") + ")"],
        [key("Page Up") + " / " + key("Page Down"), "Scrolling by 80% of the window height"],
        [key("Enter") + ", " + key("Backspace") + ", " + key("Delete") + ", " + key("Tab") + ", " + key("←") + " "
         + key("→") + ", " + key("Home") + ", " + key("End"), "The corresponding Android keys (also with "
         + key("Alt") + ": " + key("Alt", "←") + " is an arrow, not Back)"],
        ["ASCII letters", c("TESTO") + "; the rest via paste"],
        [key("Ctrl") + " + letter", "The Android shortcut (" + key("Ctrl", "C") + ", " + key("Ctrl", "A")
         + "…), pressed and released"],
        [key("Ctrl", "V") + ", " + key("Shift", "Insert"), "The PC clipboard goes into the app"],
        [key("Ctrl", "R") + ", " + key("Ctrl", "W") + ", " + key("Ctrl", "Shift", "C"), "Stay with the PC: Rotate, "
         "Close app, Copy screenshot (" + rif("App windows") + ")"],
        [key("Alt") + " + other", "Stays with the desktop (" + key("Alt", "F4") + "…)"],
    ], "«TAB» — Keyboard and mouse")

S4 = p("The service talks directly to " + c("IClipboard") + ", as the package " + c("com.android.shell")
       + ": no " + c("ClipboardManager") + ", hence no Looper to run and no detour through the Samsung "
       "service " + c("semclipboard") + ", which rejects the write with the wrong context. Signatures change between "
       "versions: the longest variant is chosen whose parameters, after the fixed ones, are only strings and integers.",
       lead=True) + \
    table(["Direction", "How", "What does not pass"], [
        ["Phone → PC", c("APPUNTI_ASCOLTA 1") + " at startup; on every copy " + c("APPUNTI_CAMBIATI") + " ("
         + c("appunti::ascolta") + ") → " + c("Collegamento::appunti") + " → GTK clipboard", "Copies marked as "
         "sensitive (state 2) or of unknown sensitivity (3); texts over 200,000 bytes; echoes"],
        ["PC → phone", "Only with " + key("Ctrl", "V") + " in a window: " + c("APPUNTI_SCRIVI") + " with "
         "incolla=1", "Texts from password managers (" + c("x-kde-passwordManagerHint") + "): alert “Password not sent to the phone”; texts that are too long: “Text too long: use file transfer”"],
    ], "«TAB» — The clipboard in both directions") + \
    p("<b>Echoes.</b> What Phonestra puts into the phone's clipboard must not come back to the PC. The service "
      "ignores its own writes (even an identical text within 3 s, because the notice may arrive later) and Samsung sends "
      "every notice twice (discarded within 0.5 s); the PC remembers the texts it sent (" + c("Collegamento::e_un_rimbalzo")
      + "). The service rereads the clipboard only if the current clip is its own: reading another app's clip would "
      "make Android's “pasted from your clipboard” notice appear.") + \
    note("on GNOME the PC clipboard can only be changed while a window of the program is active. " + c("main")
         + " sets the copy immediately and, to be safe, sets it again the next time a Phonestra window is "
         "activated; if something else is copied on the PC in the meantime, the pending one is forgotten.", "Wayland.")

CHAPTER = ("Input and clipboard", [
    ("Input and clipboard messages", S1),
    ("Injection", S2),
    ("Keyboard and mouse on the PC", S3),
    ("Clipboard", S4),
])
