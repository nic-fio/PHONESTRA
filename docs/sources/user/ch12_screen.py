from build import box, fig, note, p, path, rif, steps, table, text, tip, ui, ul, warn


STATI = fig(
    box(40, 80, 280, 70, "Screen off", "you use the phone from the PC", "navy")
    + box(580, 80, 280, 70, "Screen on: phone “in hand”", "you use the phone with your hands", "amber")
    + path([(320, 98), (580, 98)], "#d97706")
    + text(450, 34, "manual unlock after a lock", 11.5, "#9a3412", "600")
    + text(450, 52, "incoming call", 11.5, "#9a3412", "600")
    + text(450, 70, "end of a call", 11.5, "#9a3412", "600")
    + path([(580, 132), (320, 132)], "#0050C0")
    + text(450, 172, "a click, a key or the scroll wheel in a Phonestra window", 11.5, "#003a90", "600")
    + text(450, 190, "or phone untouched for its screen timeout", 11.5, "#003a90", "600"),
    900, 206, "«FIG» — When the phone's screen turns on and when it turns off again")

S1 = p("As soon as it connects, Phonestra turns off the phone's screen. The phone stays on and unlocked: the apps "
       "keep running and responding to the PC's mouse.", lead=True) + ul([
    "With the screen off you save battery and keep what you do from the PC private.",
    "Android allows apps to be shown on the PC only while the phone is unlocked: this is why Phonestra does not let it "
    "fall asleep. During the connection it sets the screen timeout to the maximum and at the end it "
    "restores it.",
    "Incoming notifications do not turn the screen back on: the phone only vibrates.",
    "The phone screen in the drawer keeps showing, live, what the phone would have on its screen.",
]) + warn("with the screen off the phone is not asleep: the glass can still receive touches. Do not put it in "
          "your pocket or bag while Phonestra is connected.", "Touches with the screen off.")

S2 = p("When you pick the phone up again, the screen turns back on and stays on; when you put it down again, "
       "it turns off again.", lead=True) + STATI + \
    table(["What happens", "The phone's screen"], [
        ["The phone, locked during use, is unlocked by hand", "Stays on: the phone is “in hand”."],
        ["A call comes in", "Turns on, so you can answer from the phone."],
        ["A call ends", "Stays on, then the touch rule applies."],
        ["Phonestra is closed", "Turns back on, and the phone goes back to how it was (" + rif("What changes on the phone") + ")."],
        ["With the phone “in hand”, a click, a key or the scroll wheel in a Phonestra window", "Turns off again: you "
         "go back to using the phone from the PC."],
        ["With the phone “in hand”, no touch for the screen timeout set on the phone", "Turns off again, as "
         "the phone would on its own."],
    ], "«TAB» — What turns the phone's screen on and off") + \
    note("while the drawer is open, closing the last app does not turn the phone's screen back on: the drawer still "
         "shows the phone's screen and keeps it off.", "With the drawer open.")

S3 = p("If the phone locks (for example with the power button, or with a double tap on the screen while it is off, a "
       "gesture on some brands), apps can no longer be shown: it is an Android security rule.",
       lead=True) + steps([
    "The drawer shows " + ui("Telefono bloccato") + " (Phone locked) with " + ui("Sbloccalo per continuare: mi ricollego da solo.")
    + " (Unlock it to continue: I will reconnect on my own.); the app windows say " + ui("Telefono bloccato: sbloccalo per continuare") + " (Phone locked: unlock it to continue).",
    "Unlock the phone with your PIN, fingerprint or face.",
    "Phonestra reconnects on its own and the apps come back in their windows, where they were. The phone's screen stays "
    "on, because the phone is “in hand”; it turns off again at the first click from the PC.",
]) + note("on some phones (for example Samsung ones) locking drops the connection: for a few seconds "
          + ui("riconnessione…") + " (reconnecting…) appears. This is normal.", "A few seconds of waiting.")

S4 = p("Calls are made and received with the phone in hand (" + rif("Calls") + ").", lead=True) + ul([
    "When the phone rings, Phonestra turns the screen back on: you answer from the phone as always.",
    "During the call the screen does not turn off on its own, even if the phone is not touched (you do not touch it "
    "while it is at your ear).",
    "After the call the screen stays on; it turns off again at the first click from the PC or after the phone's "
    "screen timeout.",
]) + tip("a call of an app open in a window (for example WhatsApp) can also be accepted with a click on the "
         "PC; the voice, however, always goes through the phone.", "Answering from the PC.")

CHAPTER = ("The phone's screen", [
    ("Why the screen turns off", S1),
    ("The phone in hand", S2),
    ("Locked phone", S3),
    ("Incoming calls", S4),
])
