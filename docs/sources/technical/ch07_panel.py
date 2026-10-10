# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import arrow, box, c, fig, note, p, path, rif, steps, table, term, text, warn

S1 = p("While apps are used from the PC the phone's screen turns off, but the phone stays awake and unlocked: "
       "that way the apps on the virtual displays keep running and receiving touches. When the user picks the phone "
       "back up, the panel turns back on; when they leave it there, it turns off again.", lead=True) + \
    p(c("Pannello.java") + " calls " + c("SurfaceControl.setDisplayPowerMode(token, 0|2)") + " on every physical "
      "display: it acts only on the compositor, Android believes the screen is on and any change of state (power "
      "button) turns it back on. Since Android 14 the tokens live in " + c("DisplayControl") + ", inside "
      + c("services.jar") + ", loaded with a class loader on the " + c("SYSTEMSERVERCLASSPATH") + " and the library "
      + c("android_servers") + ".") + \
    p("The PC controls it with " + c("VIDEO_PANNELLO") + " (" + rif("Video messages") + "): "
      + c("video_nostro::pannello") + " sends it without waiting for the reply, " + c("ComandiVideo::pannello")
      + " sends it from a session. There is one panel per phone, not one per window: the one that decides whether to "
      "turn it off is the " + c("Collegamento") + ", with the phone “in hand” rule described below. When the service "
      "dies, the guardian's action 400 turns it back on (" + rif("The two guardians") + ").")

S2 = p("Samsung phones change the display refresh rate by themselves (10–120 Hz): after a moment of calm it is at 24 Hz. When "
       "the panel is turned off SurfaceFlinger switches to 60 Hz, but confirms the change only with the panel's vsyncs, "
       "which no longer arrive: its model stays at 24 Hz, takes the frames of the virtual displays at that pace, and apps that "
       "draw faster remain stuck waiting for it (Facebook, with AV1 reels decoded in software, starves "
       "the audio). Prove §59.", lead=True) + steps([
    "Before turning off, " + c("Pannello.java") + " reads " + c("mDisplayModePtr") + " and "
    + c("mPeriodConfirmationInProgress") + " from " + c("dumpsys SurfaceFlinger") + ".",
    "If the model is not confirmed at 60 Hz or more, it sets " + c("min_refresh_rate=60") + " and registers with the "
    "guardian action 410, which restores the previous value.",
    "It waits for the confirmation, at most 1 s (" + c("ATTESA_60_MS") + "), then turns off.",
    "It immediately restores " + c("min_refresh_rate") + " as it was and removes the guardian's action.",
])

STATI = fig(
    box(60, 90, 270, 70, "Panel off", "the phone is used from the PC", "navy")
    + box(570, 90, 270, 70, "Panel on, “in hand”", "set by the connection", "amber")
    + path([(330, 108), (570, 108)], "#d97706") + text(450, 40, "manual unlock · incoming call", 11, "#9a3412", "600")
    + text(450, 58, "end of call · last session closed", 11, "#9a3412", "600")
    + text(450, 76, "reconnection after a lock", 11, "#9a3412", "600")
    + path([(570, 142), (330, 142)], "#0050C0") + text(450, 184, "a touch, a key or a click from the PC", 11,
                                                          "#003a90", "600")
    + text(450, 202, "idle for the user's screen timeout, with no calls", 11, "#003a90", "600")
    + text(195, 240, "sessions that open turn the panel off", 11, "#475569", "400")
    + text(705, 240, "sessions that open leave it on", 11, "#475569", "400"),
    900, 260, "«FIG» — The two states of the panel and what changes them")

S3 = p("The state is a single value in the " + c("Collegamento") + ": " + c("a_mano") + ", read with "
       + c("Collegamento::pannello_a_mano") + ". With " + c("a_mano") + " false, sessions that open turn the "
       "panel off; with " + c("a_mano") + " true they leave it on. It lasts as long as the process, across "
       "reconnections: a drop does not reset it.", lead=True) + STATI + \
    table(["Event", "Where", "Effect"], [
        ["The phone, locked during use, is unlocked", c("Collegamento::sbloccato"), "“in hand”: the panel "
         "stays on"],
        ["First check after a connection, phone unlocked", c("Collegamento::sbloccato"), "“in hand” only if "
         "there had been a lock before, or a drop with the phone asleep (" + rif("Drops and reconnections") + ")"],
        ["Incoming call", "3-second round", "panel on, “in hand” (" + rif("Calls") + ")"],
        ["End of a call with the panel off", "3-second round", "panel on, “in hand”"],
        ["Closing of the last session", c("finestra.rs") + ", " + c("pannello_acceso_senza_finestre"),
         "panel on, “in hand”"],
        ["Touch, key, scroll, text, paste, zoom, long press or Back from a window",
         c("Collegamento::usa_dal_pc"), "if it was “in hand”: panel off, no longer “in hand”"],
        ["Phone “in hand” idle for the user's screen timeout, with no calls during that time",
         "3-second round", "panel off, no longer “in hand”"],
    ], "«TAB» — Who turns the panel on and who turns it off") + \
    p("Turning off without touches exists because the phone's screen timeout, during the connection, is at the "
      "maximum (" + rif("The two guardians") + "): a phone unlocked by hand and then left on the table would never "
      "turn off. The 3-second round, as long as the phone is “in hand”, asks " + c("dumpsys power | grep -m1 "
      "lastUserActivityTime=") + " and reads “(N ms ago)” (" + c("fermo_da") + "): once the time chosen by the user "
      "(the one saved at the start of the connection) has passed, it turns the panel off. This also applies with no windows open: the phone "
      "stays awake and unlocked (prove §58 and §60).") + \
    warn("the session counter (" + c("Collegamento::sessioni") + ") also counts the drawer's mirror. "
         "“Last session” therefore means the last window <i>and</i> the drawer: with the drawer open, closing "
         "an app's last window does not turn the panel back on. Opening the mirror also turns the panel off, "
         "just like opening a window.", "The mirror counts as a session.")

S4 = p("The 3-second round reads the call state with " + c("dumpsys telephony.registry | grep -o "
       "'mCallState=[12]'") + ": 1 means it is ringing, 2 that it is in progress. With two SIMs there is one line per SIM, and "
       "one line is enough to count.", lead=True) + \
    table(["Moment", "Rule", "Why"], [
        ["It starts ringing", "If the phone is not already “in hand”, panel on and “in hand”", "To answer "
         "with the phone in hand (prove §58)."],
        ["During the call", "The time without touches is counted from the last round with a call "
         "(" + c("chiamata_alle") + "): the panel does not turn off", "With the phone at your ear there are no touches."],
        ["It ends, and the phone was not “in hand”", "Panel on and “in hand”; then turning off without "
         "touches applies", "Answered with a click from the PC: during the call Android turns the panel back on by itself (proximity "
         "sensor), and Phonestra would believe it off; it stayed on forever (prove §60)."],
    ], "«TAB» — Calls and the panel") + \
    warn("the call variables (" + c("squillava") + ", " + c("in_chiamata") + ", " + c("chiamata_alle")
         + ") start from zero at every connection. Before restarting Phonestra for a test, look at all the "
         + c("mCallState") + " lines: a restart during a call leaves the mirror black (prove §61).",
         "No restarts during a call.")

S5 = p("On Samsung phones locking the phone drops Wireless debugging: the connection reopens on unlock. But "
       "the connection also drops because of the network, with the phone still unlocked on the table. In both cases the panel "
       "is on upon return (the guardian or the user turned it back on), and " + c("Collegamento::sbloccato") + " must figure out "
       "who did it.", lead=True) + steps([
    "On the drop, " + c("mantieni") + " notes the time (" + c("caduto") + "), only the first time and only if the "
    "connection was really open.",
    "At the first lock check after the reconnection, if the phone is unlocked: if before the drop the round "
    "had seen the lock (" + c("bloccato_durante_uso") + "), the user unlocked it: “in hand”.",
    "Otherwise " + c("dumpsys power | grep -m1 mLastSleepTime=") + " tells how long ago the phone fell asleep "
    "(" + c("dormito_da") + "). If that happened no longer ago than the time elapsed since the drop plus "
    + c("MARGINE_CADUTA") + " (15 s, generous compared with the 8 s within which the PC notices the drop: 5 s of "
    "waiting for the reply plus 3 s between checks), the phone has "
    "slept and the user unlocked it: “in hand”.",
    "If it slept earlier, the drop was a network one: the panel turns off again as the sessions restart. If the line "
    "is missing or the command does not reply, when in doubt “in hand”.",
]) + p("“Unlocked by hand” is decided from the first lock check after the connection, not from the "
       "reconnection alone (prove §55 and §57).")

S6 = p("Every panel change leaves a line in Phonestra's log, which ends up in the PC's journal:",
       lead=True) + term("""
$ journalctl --user --since today | grep -i phonestra
""") + table(["Line", "When"], [
    [c("[collegamento] sbloccato a mano: il pannello resta acceso"), "manual unlock, even after a drop (“unlocked by hand: the panel stays on”)"],
    [c("[collegamento] caduta senza blocco: il pannello si rispegne"), "reconnection after a network drop (“drop without lock: the panel turns off again”)"],
    [c("[collegamento] chiamata in arrivo: pannello acceso"), "it starts ringing (“incoming call: panel on”)"],
    [c("[collegamento] fine della chiamata: pannello acceso, si rispegne senza tocchi"), "end of a call with the "
     "panel off (“end of the call: panel on, turns off again without touches”)"],
    [c("[collegamento] telefono in mano non toccato da N s: pannello spento"), "turning off without touches (“phone in hand not touched for N s: panel off”)"],
    [c("[finestra] usato dal PC: pannello spento"), "first touch from the PC with the phone “in hand” (“used from the PC: panel off”)"],
    [c("[finestra] ultima finestra chiusa: pannello acceso"), "closing of the last session (“last window closed: panel on”)"],
], "«TAB» — The lines of the panel change log") + \
    note("turning off when a session opens does not leave a line: it happens all the time. The "
         "service's lines arrive in the same log prefixed by " + c("[servizio]") + ": when the "
         "display needs to be brought to 60 Hz (" + rif("Display refresh rate when the panel is off") + "), "
         + c("phonestra-servizio: video: frequenza del display a 60 Hz prima dello spegnimento") + " appears.")

CHAPTER = ("The phone's panel", [
    ("Turning the panel off", S1),
    ("Display refresh rate when the panel is off", S2),
    ("The phone in hand", S3),
    ("Calls", S4),
    ("Drops and reconnections", S5),
    ("The change log", S6),
])
