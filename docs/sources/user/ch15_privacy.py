from build import c, key, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("To work, Phonestra changes a few things on the phone, only while it is connected. When it closes it puts "
       "them back as they were, even if the connection drops suddenly.", lead=True) + \
    table(["What", "During the connection", "At the end"], [
        ["Phone screen", "Off, with the phone awake and unlocked (" + rif("The phone's screen") + ")",
         "Turned back on"],
        ["Screen timeout", "At the maximum, so that the phone does not go to sleep", "The previous value; "
         "if you changed it on the phone in the meantime, the new one stays"],
        ["Media volume", "At the maximum (the phone stays silent: the audio comes out of the PC)", "The previous value"],
        ["Phone audio", "Comes out of the PC", "Goes back to the phone's loudspeaker; music that is playing is paused"],
        ["Apps open in windows", "Run on extra screens, invisible on the phone", "Closed and removed from the "
         "recent apps"],
        ["Phonestra's part on the phone", "A small program copied into a temporary folder on the phone "
         "(" + c("/data/local/tmp") + ") and started at every connection", "Deleted"],
    ], "«TAB» — What Phonestra changes on the phone and how it puts it back") + ul([
        "No app is installed on the phone, and nothing is left behind after closing.",
        "If Phonestra does not close properly (PC switched off abruptly, Wi-Fi lost), the phone puts itself back in "
        "order a few seconds later. If something stays changed, Phonestra restores it at the next connection: it "
        "remembers the previous values on the PC.",
        "After closing, the phone stays unlocked with the screen on: it turns off and locks by itself, after its "
        "screen timeout, as always.",
    ]) + warn("in some cases the media volume can stay at the maximum after closing (" + rif("Volume")
              + "): just lower it with the phone's buttons.", "Volume.")

S2 = p("Some things switched on for the first connection stay on even after Phonestra closes, so that next time "
       "there is nothing to do again.", lead=True) + \
    table(["What", "Why it stays", "How to turn it off"], [
        [ui("Developer options"), "Needed for Wireless debugging.", "In the phone's "
         "Settings, the switch at the top of " + ui("Developer options") + "."],
        [ui("Wireless debugging"), "Phonestra needs it to find and reach the phone. Phonestra "
         "does not turn it off when it closes: the " + ui("Turn off Wireless debugging on exit") + " item of the phone menu is " + ui("Coming soon") + ".", "In "
         + ui("Developer options") + ". To use Phonestra again it will have to be turned back on, without "
         "entering the code again."],
        ["PC authorization", "The phone remembers this PC among the " + ui("Paired devices") + ". Phonestra removes the automatic expiry that Android applies to authorizations not used for a "
         "few days.",
         ui("Wireless debugging") + " › " + ui("Paired devices") + ": remove the PC."],
    ], "«TAB» — What stays switched on on the phone") + \
    note("with Wireless debugging on, the phone announces itself on the Wi-Fi network. Only PCs paired with the code "
         "can connect. On networks other than your home one (hotel, office, public places) it is advisable to turn "
         "it off when you are not using Phonestra.", "Public networks.")

S3 = ul([
    "<b>No internet.</b> Phonestra only talks to the phone, on the home network. It does not send data to external "
    "servers, and has no advertising or usage statistics. The only exception is " + ui("Ask Google ↗") + " in " + ui("Add a phone") + ", which opens the browser on a search, only if you press it.",
    "<b>Encrypted connection.</b> Everything between PC and phone travels encrypted, with the same protection as "
    "Android's Wireless debugging.",
    "<b>Phonestra's key.</b> The PC identifies itself to the phone with a secret key, in the file "
    + c("~/.config/Phonestra/adbkey") + ". Anyone who has that file can connect to the phone as Phonestra: do not "
    "copy or share it.",
    "<b>Notifications.</b> Phonestra reads the title and text of the phone's notifications to show them on the PC. "
    "It does not keep them: they only remain while Phonestra is open. With " + ui("App name only") + " the desktop alerts do not show sender and text.",
    "<b>Clipboard.</b> Passwords copied on the phone do not pass to the PC, and those copied on the PC from a "
    "password manager do not pass to the phone (" + rif("The clipboard") + ").",
    "<b>PC screen.</b> While Phonestra is open, anyone looking at the PC sees the phone's apps and notifications.",
    "<b>Protected screens.</b> Screens that apps protect (banking, passwords) never reach the PC.",
]) + tip("to leave the PC for a while, just close Phonestra: the phone goes back to how it was and nothing is left "
         "open on the PC.", "Leaving the PC.")

S4 = table(["Folder", "Contents", "Can it be deleted?"], [
    [c("~/.config/Phonestra/adbkey"), "Phonestra's secret key", "Yes, but then every phone has to be paired "
     "again"],
    [c("~/.config/Phonestra/telefoni.toml"), "The connected phones: name, model, favorites, the screen timeout and "
     "volume values to restore", "Yes: Phonestra starts again from " + ui("Add a phone") + ""],
    [c("~/.config/Phonestra/preferenze.toml"), "The preferences", "Yes: they return to the defaults"],
    [c("~/.config/Phonestra/icone/"), "The app icons, for the desktop alerts", "Yes"],
    [c("~/.cache/Phonestra/"), "Phonestra's working files", "Yes, without losing anything"],
    [c("Pictures/Phonestra"), "The screenshots", "They are your own files"],
    [c("Videos/Phonestra"), "The recordings", "They are your own files"],
    [c("Downloads"), "The files received from the phone (unless you chose another folder)", "They are your own files"],
], "«TAB» — Phonestra's folders on the PC") + \
    p("Phonestra does not write anywhere else: no menu icons, no services that start with the PC, no changes to the "
      "system. Deleting " + c("~/.config/Phonestra") + " brings Phonestra back to how it was at the first start.") + \
    note(c("~") + " is the home folder. The names " + c("Pictures") + ", " + c("Videos") + " and " + c("Downloads")
         + " are those of an English-language desktop: Phonestra uses the folders the desktop has chosen for "
         "pictures, videos and downloads, which may have other names in other languages.",
         "Folder names.")

S5 = steps([
    "Close Phonestra, so that the phone goes back to how it was.",
    "Delete the file " + c("Phonestra-<version>-x86_64.AppImage") + ".",
    "Delete the folders " + c("~/.config/Phonestra") + " and " + c("~/.cache/Phonestra") + " (in the file manager "
    "they are hidden folders: they appear with " + "“Show Hidden Files”" + " or " + key("Ctrl", "H") + ").",
    "If you no longer need them, delete " + c("Pictures/Phonestra") + " and " + c("Videos/Phonestra") + " too.",
    "On the phone: " + ui("Wireless debugging") + " › " + ui("Paired devices") + ", remove the PC; then turn off "
    + ui("Wireless debugging") + " and, if you like, " + ui("Developer options") + ".",
]) + p("After these steps nothing of Phonestra is left, neither on the PC nor on the phone.")

CHAPTER = ("Phone, PC and privacy", [
    ("What changes on the phone", S1),
    ("What stays switched on on the phone", S2),
    ("Privacy and security", S3),
    ("The folders on the PC", S4),
    ("Removing Phonestra", S5),
])
