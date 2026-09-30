from build import VERSION, arrow, box, c, fig, note, p, rif, table, text, tip, ui, ul, zone


SCHEMA = fig(
    zone(20, 14, 360, 196, "Linux PC")
    + box(40, 46, 320, 44, "Phonestra drawer", "apps, notifications, the phone screen", "navy")
    + box(40, 100, 155, 44, "An app's window", "one for each app", "blue")
    + box(205, 100, 155, 44, "An app's window", "used with the mouse", "blue")
    + box(40, 154, 320, 40, "PC speakers, mouse, keyboard and clipboard", "", "soft", 12)
    + zone(520, 14, 360, 196, "Android 14 or later phone")
    + box(540, 46, 320, 44, "The phone's apps", "run on the phone, as always", "blue")
    + box(540, 100, 320, 44, "Wireless debugging", "an Android setting, turned on once", "dark")
    + box(540, 154, 320, 40, "No app to install", "", "soft", 12)
    + arrow(516, 70, 384, 70, "#0050C0") + text(450, 58, "picture and sound", 11, "#003a90", "700")
    + arrow(384, 124, 516, 124, "#16a34a") + text(450, 112, "clicks, keys, files", 11, "#166534", "700")
    + text(450, 176, "same Wi-Fi network", 11.5, "#334155", "700")
    + text(450, 192, "encrypted connection", 11, "#475569"),
    900, 224, "«FIG» — Phonestra shows on the PC the apps running on the phone")

S1 = p("Phonestra is a Linux program that brings the apps of your Android phone to your PC. Each app opens in its own "
       "window, like a PC program, and is used with mouse and keyboard. The phone stays on the desk: it only needs to be "
       "on, unlocked and on the same Wi-Fi network as the PC.", lead=True) + SCHEMA + \
    p("The apps keep running on the phone, with their data and their accounts: Phonestra shows their picture on the PC "
      "and sends clicks and keystrokes to the phone. With Phonestra you can:") + ul([
        "open the phone's apps in PC windows, even several apps at once;",
        "type with the PC keyboard and copy and paste text in both directions;",
        "hear the phone's audio through the PC speakers;",
        "see the phone's notifications on the PC;",
        "install apps, send files to the phone and receive files from it;",
        "take screenshots and record an app's screen.",
    ]) + \
    note("no app is installed on the phone. Only a few Android settings are turned on, once "
         "(" + rif("Connecting the phone") + "). Whatever Phonestra changes on the phone while in use, it puts back "
         "the way it was at the end (" + rif("What changes on the phone") + ").", "Nothing on the phone.")

S2 = table(["Who", "What they do with Phonestra", "Chapters"], [
    ["Phonestra users", "Download the program, connect the phone the first time and use the apps from the PC.", "2 - 14"],
    ["Anyone who wants to know what happens to the phone", "Read what changes on the phone, what stays switched on and "
     "where Phonestra keeps its data.", "15"],
    ["Anyone with a problem", "Look up the symptom or the message and the fix.", "16"],
], "«TAB» — The readers of this manual") + \
    p("First-time users can follow " + rif("First steps") + ": it goes from the downloaded file to the "
      "first app open on the PC in a few steps. The " + rif("Glossary") + " explains the less common words.")

S3 = table(["Convention", "Meaning"], [
    [ui("Aggiungi telefono") + " (Add phone)", "Text that appears in the interface: buttons, menu items, labels, messages."],
    [c("~/.config/Phonestra"), "File and folder names, commands, values to type exactly as shown."],
    ["Blue <b>Note</b> box", "Useful information for understanding."],
    ["Green <b>Tip</b> box", "A simpler or safer way to do something."],
    ["Yellow <b>Warning</b> box", "A behavior that may be surprising or may cause data loss."],
], "«TAB» — The manual's conventions") + \
    p("In this manual, “phone” always means the Android phone connected to Phonestra; “PC” means the computer running "
      "Linux. Phonestra's interface is in Italian: the manual quotes its labels exactly as they appear, with the English "
      "translation in parentheses where needed. The names of Android settings (for example " + ui("Debug wireless")
      + " for Wireless debugging) may vary slightly from one brand to another: the manual uses the most common ones.") + \
    note("this manual describes Phonestra " + VERSION + ". The internal workings are described in the Technical "
         "Manual (" + c("docs/Technical Manual.html") + "). Some interface items are marked "
         + ui("In arrivo") + " (Coming soon): they do nothing yet, and the manual mentions them only to say so.", "Version.") + \
    tip("Phonestra follows the desktop's light or dark theme. The figures in this manual are diagrams, not photographs: "
        "colors and proportions on your own screen may differ.", "Appearance.")

CHAPTER = ("Welcome", [
    ("What Phonestra is", S1),
    ("Who this manual is for", S2),
    ("Conventions", S3),
])
