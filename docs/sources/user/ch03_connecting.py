# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import c, flow, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("The first connection is done only once for each phone, without a cable, in a few minutes. Phonestra "
       "asks you to turn on four things on the phone; how to do it on your model you can find out with the search "
       "in the phone's Settings.", lead=True) + ul([
    "The phone: Android 14 or later, charged, unlocked, within reach.",
    "The PC and the phone on the same Wi-Fi network.",
    "Phonestra running: at first start " + ui("Add a phone") + " opens by itself; for an "
    "additional phone, open it from the drawer with " + ui("Add phone") + " ("
    + rif("Adding another phone") + ").",
])

VOCI = table(["No.", "Item in the window", "What to do on the phone", "Word to search for in Settings"], [
    ["1", ui("Phone and PC on the same Wi-Fi network"), "Check that the phone "
     "is connected to the same Wi-Fi network as the PC (not the “guest” one, not mobile data).", "—"],
    ["2", ui("Unlock Developer options"), "Open the phone information and tap "
     + ui("Build number") + " 7 times (some brands call it something else); the phone asks for the PIN "
     "and unlocks the " + ui("Developer options") + ".", c("build number")],
    ["3", ui("Turn off the protections that block the connection"), "Only if present: " + ui("Auto Blocker")
     + " on Samsung, " + ui("Advanced Protection") + " on Google Pixel. If you "
     "can't find them, the phone doesn't have them.",
     c("auto blocker")],
    ["4", ui("Wireless debugging: turn it on and pair this PC"), "Turn "
     "on the " + ui("Wireless debugging") + " switch and tap " + ui("Allow") + " for the network; then tap the "
     + ui("Wireless debugging") + " label (not the switch), then " + ui("Pair device with pairing code")
     + ".", c("wireless debugging")],
], "«TAB» — The four items of “Add a phone”")

S2 = p("The window is called " + ui("Add a phone") + ". On the left, under the heading " + ui("Prepare the phone") + ", is the list of the four things to do; on the right, the " + ui("WHAT I SEE ON THE NETWORK") + " and " + ui("STUCK?") + " boxes and the button for the cable "
       "route.", lead=True) + VOCI + \
    p("Each item opens with a click and contains:") + ul([
        "why it is needed, in plain words;",
        "the " + ui("WHERE TO FIND IT") + " box, with the word to type into the search in the phone's "
        "Settings;",
        ui("Done ›") + ", to press when the item is done (the first three);",
        ui("Ask Google ↗") + ", which opens a Google search in the browser with the question already "
        "typed: it explains how to do it on your model, often with a video.",
    ]) + \
    table(["Label to the right of the item", "Meaning"], [
        [ui("to do on the phone"), "Item still to be done."],
        [ui("✓ done"), "Marked as done with " + ui("Done ›") + "."],
        [ui("✓ seen by Phonestra"), "Phonestra sees Wireless debugging turned on: the items before "
         "it are necessarily done."],
        [ui("I'll notice by myself"), "Phonestra is waiting to see Wireless debugging turned on "
         "on the network."],
        [ui("✓ on: now the code"), "Wireless debugging is on: the pairing code is still needed."],
        [ui("type the code"), "The phone shows the code: type it into the field of item 4."],
        [ui("pairing…"), "Phonestra is pairing with the phone."],
        [ui("✓ connected"), "Done: the phone is connected."],
    ], "«TAB» — The status of each item") + \
    note("the items that Phonestra can see on the network get checked by themselves. The " + ui("WHAT I SEE ON THE NETWORK")
         + " box says what it sees at that moment: " + ui("○ No phone with Wireless debugging on") + ", " + ui("● A phone with Wireless debugging on") + ", " + ui("● Code screen open") + ".",
         "Automatic checkmarks.") + \
    tip("if you get stuck, the " + ui("STUCK?") + " box suggests taking a photo of the window and "
        "sending it to whoever recommended Phonestra: the window shows how far you have got.", "Getting help.")

FLUSSO = flow([
    ("Same network", "home Wi-Fi", "soft"),
    ("Developer", "options: 7 taps", "blue"),
    ("Protections", "only if present", "blue"),
    ("Wireless debugging", "turned on", "blue"),
    ("Code", "6 digits", "navy"),
    ("Connected", "once only", "green"),
], "«FIG» — The first connection, from the network to the connected phone", width=960)

S3 = FLUSSO + steps([
    "Check that the phone is on the same Wi-Fi network as the PC; press " + ui("Done ›") + ".",
    "Unlock the " + ui("Developer options") + " (item 2); press " + ui("Done ›") + ".",
    "If the phone has " + ui("Auto Blocker") + " or " + ui("Advanced Protection") + " turned on, turn them off "
    "(item 3); press " + ui("Done ›") + ".",
    "In " + ui("Developer options") + " turn on " + ui("Wireless debugging") + " and tap " + ui("Allow") + " for the network. Phonestra notices by itself and opens item 4.",
    "Tap the " + ui("Wireless debugging") + " label, then " + ui("Pair device with pairing code")
    + ": the phone shows a 6-digit code.",
    "Type the 6 digits into the field of item 4. There is nothing to press: with the sixth digit Phonestra pairs, "
    "connects and saves the phone.",
    "When " + ui("Done! “<phone name>” is connected via Wi-Fi.") + " appears, press " + ui("Start using the phone") + ": the drawer opens.",
]) + note("the phone's address and port don't need to be typed anywhere: Phonestra finds them on the network by itself.",
          "No addresses.")

S4 = table(["Message under the field", "What to do"], [
    [ui("The field activates when you open the code screen on the phone."), "On the phone, open "
     + ui("Pair device with pairing code") + ". If the field stays gray, check that the phone "
     "and the PC are on the same network."],
    [ui("I can see the code screen: type the 6 digits, I'll take care of the rest."), "Type the 6 digits shown by the phone."],
    [ui("Code not accepted (…). Reopen “Pair device with pairing code” on the phone and type the new code.")
     + "", "The code was wrong or expired: on the phone, close and reopen the code screen, then type the new one."],
], "«TAB» — The code field") + \
    p("After pairing, Phonestra removes the expiry of this PC's authorization, so the connection doesn't "
      "have to be redone after a few days of not being used (" + rif("What stays switched on on the phone") + ").")

S5 = p("The " + ui("Android 10 or earlier? Connect with the cable") + " "
       "button, at the bottom right, opens the fallback route: the first connection with the USB cable, designed for "
       "phones that don't have Wireless debugging. Afterward the cable is unplugged and Phonestra works over Wi-Fi "
       "as always.", lead=True) + \
    warn("the apps in the windows still require Android 14 or later: with an older phone Phonestra "
         "cannot show them, whichever route is used for the first connection.", "Android 14.") + \
    table(["Step", "What the window asks"], [
        [ui("Connect the USB cable"), "Connect the phone to the PC with a cable that carries data "
         "(some cables only charge). If nothing happens: from the phone's notification shade, " + ui("USB")
         + " notification, choose " + ui("File transfer") + "."],
        [ui("Turn on USB debugging"), "Unlock the " + ui("Developer options") + " and turn on " + ui("USB debugging") + ". The window shows the steps for the phone's brand, to browse with the arrows under the "
         "drawn phone."],
        [ui("Allow the connection"), "A request appears on the phone: check "
         + ui("Always allow from this computer") + " and tap " + ui("Allow") + "."],
        [ui("Allow the Wi-Fi network"), "Phonestra turns on Wireless debugging; the first time "
         "on each network the phone asks for permission: tap " + ui("Allow") + "."],
        [ui("Done"), "The phone is set up and connects over Wi-Fi: you can unplug the cable."],
    ], "«TAB» — The steps of the cable route") + ul([
        "The window moves on by itself: it notices every step done on the phone.",
        "Without the " + ui("Always allow from this computer") + " checkmark the cable works but Wi-Fi doesn't: "
        + ui("“Always allow” missing") + " appears and the request must be repeated by "
        "unplugging and replugging the cable.",
        "On Xiaomi, Redmi and Poco phones there is an extra step, " + ui("One more step for Xiaomi") + ": turn on " + ui("USB debugging (Security settings)") + ", otherwise the apps are visible but don't respond to mouse and keyboard.",
        "If after 20 seconds the window is still at the first step, the " + ui("What the PC sees") + " box appears with the list of USB devices: it is useful to photograph for someone helping remotely.",
    ])

S6 = p("After the first connection there is nothing more to do: at startup Phonestra looks for the phone on the "
       "network and connects by itself.", lead=True) + ul([
    "The phone must be on, unlocked, on the same Wi-Fi network and with " + ui("Wireless debugging") + " turned on.",
    "Android sometimes turns Wireless debugging off by itself, for example when switching Wi-Fi networks. Just turn "
    "it back on in " + ui("Developer options") + ": the code doesn't need to be redone, the PC stays paired.",
    "On a new network the phone asks again for " + ui("Allow") + " for that network.",
]) + tip("many phones let you add " + ui("Wireless debugging") + " to the Quick Settings tiles in the notification shade: "
         "that way it turns back on with one tap.", "Turning it back on quickly.")

CHAPTER = ("Connecting the phone", [
    ("Before you start", S1),
    ("The “Add a phone” window", S2),
    ("Step by step", S3),
    ("The 6-digit code", S4),
    ("The cable route", S5),
    ("Later connections", S6),
])
