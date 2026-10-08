from build import VERSION, c, p, pill, table

S1 = p("Open as of version " + VERSION + "; the up-to-date status is in " + c("notes/issue-log.md") + ".",
       lead=True) + table(["Issue", "Status"], [
    ["Original volume saved as 15 instead of the user's value: on shutdown the phone may be left at maximum",
     pill("accepted", "off") + " by the user: the volume is turned down by hand (" + c("Collegamento::volume_originale") + ")"],
    ["The audio margin shrinks again only on silence: " + c("Margine::scendi") + " skips Opus packets of a few "
     "bytes; with the AAC of version 1.1.1 it never triggered", pill("to measure", "wait") + " on long YouTube and Facebook sessions"],
    ["With " + c("delayed ack") + " announced, adbd rejects every channel", pill("off", "off") + " to be investigated ("
     + c("notes/adb.md") + ")"],
    ["Service memory about 145 MB (ART startup cost)", pill("to measure", "wait")],
    ["Why the audio startup order matters", pill("rule found", "ok") + " through measurements; the Android "
     "mechanism is still to be understood"],
    ["PC Wi-Fi drop, recording with Opus audio on different phones", pill("to test", "wait")],
    ["Real call after rc.7: panel on for the incoming call and off again after it ends", pill("to test", "wait")],
    ["“Receive files…”: SD card, cancellation, large folders, dark theme", pill("to test", "wait")],
    ["System alerts: on the first read after the drawer opens, the notifications already present on the phone "
     "raise an alert, because the “already seen” set is fixed while the list is still empty (" + c("avvisa_nuove")
     + " in " + c("cassetto.rs") + ")", pill("to verify", "wait") + " probable bug, found by rereading the code"],
    ["Locked phone: on Samsung phones, locking drops Wireless debugging", pill("worked around", "ok") + " the "
     "connection reopens on unlock and the apps go back to where they were"],
], "«TAB» — Known issues")

CHAPTER = ("Appendix C — Known issues", [
    ("Open issues", S1),
])
