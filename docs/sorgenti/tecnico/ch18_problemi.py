from build import VERSION, c, p, pill, table

S1 = p("Open as of version " + VERSION + "; the up-to-date status is in " + c("memoria/registro-problemi.md") + ".",
       lead=True) + table(["Issue", "Status"], [
    ["Original volume saved as 15 instead of the user's value: on shutdown the phone may be left at maximum",
     pill("accepted", "off") + " by the user: the volume is turned down by hand (" + c("Collegamento::volume_originale") + ")"],
    ["The audio margin grows but does not shrink: " + c("Margine::scendi") + " does not trigger because AAC packets are "
     "of almost constant size", pill("to do", "wait") + " silence marked by the phone"],
    ["With " + c("delayed ack") + " announced, adbd rejects every channel", pill("off", "off") + " to be investigated ("
     + c("memoria/adb.md") + ")"],
    ["Service memory about 145 MB (ART startup cost)", pill("to measure", "wait")],
    ["Why the audio startup order matters", pill("rule found", "ok") + " through measurements; the Android "
     "mechanism is still to be understood"],
    ["PC Wi-Fi drop, recording with AAC audio on different phones", pill("to test", "wait")],
    ["Real call after rc.7: panel on for the incoming call and off again after it ends", pill("to test", "wait")],
    ["“Ricevi file…” (Receive files…): SD card, cancellation, large folders, dark theme", pill("to test", "wait")],
    ["System alerts: on the first read after the drawer opens, the notifications already present on the phone "
     "raise an alert, because the “already seen” set is fixed while the list is still empty (" + c("avvisa_nuove")
     + " in " + c("cassetto.rs") + ")", pill("to verify", "wait") + " probable bug, found by rereading the code"],
    ["Locked phone: on Samsung phones, locking drops Wireless debugging", pill("worked around", "ok") + " the "
     "connection reopens on unlock and the apps go back to where they were"],
], "«TAB» — Known issues")

CHAPTER = ("Appendix C — Known issues", [
    ("Open issues", S1),
])
