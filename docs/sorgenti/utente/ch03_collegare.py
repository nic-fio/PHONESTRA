from build import c, flow, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("The first connection is done only once for each phone, without a cable, in a few minutes. Phonestra "
       "asks you to turn on four things on the phone; how to do it on your model you can find out with the search "
       "in the phone's Settings.", lead=True) + ul([
    "The phone: Android 14 or later, charged, unlocked, within reach.",
    "The PC and the phone on the same Wi-Fi network.",
    "Phonestra running: at first start " + ui("Aggiungi un telefono") + " (Add a phone) opens by itself; for an "
    "additional phone, open it from the drawer with " + ui("Aggiungi telefono") + " (Add phone) ("
    + rif("Adding another phone") + ").",
])

VOCI = table(["No.", "Item in the window", "What to do on the phone", "Word to search for in Settings"], [
    ["1", ui("Telefono e PC sulla stessa rete Wi-Fi"), "(Phone and PC on the same Wi-Fi network) Check that the phone "
     "is connected to the same Wi-Fi network as the PC (not the “guest” one, not mobile data).", "—"],
    ["2", ui("Sblocca le Opzioni sviluppatore"), "(Unlock Developer options) Open the phone information and tap "
     + ui("Numero build") + " (Build number) 7 times (some brands call it something else); the phone asks for the PIN "
     "and unlocks the " + ui("Opzioni sviluppatore") + " (Developer options).", c("numero build")],
    ["3", ui("Spegni le protezioni che bloccano il collegamento"), "(Turn off the protections that block the "
     "connection) Only if present: " + ui("Blocco automatico")
     + " (Auto Blocker) on Samsung, " + ui("Protezione avanzata") + " (Advanced Protection) on Google Pixel. If you "
     "can't find them, the phone doesn't have them.",
     c("blocco automatico")],
    ["4", ui("Debug wireless: accendilo e associa questo PC"), "(Wireless debugging: turn it on and pair this PC) Turn "
     "on the " + ui("Debug wireless")
     + " (Wireless debugging) switch and tap " + ui("Consenti") + " (Allow) for the network; then tap the "
     + ui("Debug wireless") + " label (not the switch), then " + ui("Associa dispositivo con codice di associazione")
     + " (Pair device with pairing code).", c("debug wireless")],
], "«TAB» — The four items of “Aggiungi un telefono”")

S2 = p("The window is called " + ui("Aggiungi un telefono") + ". On the left, under the heading " + ui("Prepara il telefono")
       + " (Prepare the phone), is the list of the four things to do; on the right, the " + ui("COSA VEDO IN RETE")
       + " (WHAT I SEE ON THE NETWORK) and " + ui("TI SEI BLOCCATO?") + " (STUCK?) boxes and the button for the cable "
       "route.", lead=True) + VOCI + \
    p("Each item opens with a click and contains:") + ul([
        "why it is needed, in plain words;",
        "the " + ui("DOVE SI TROVA") + " (WHERE TO FIND IT) box, with the word to type into the search in the phone's "
        "Settings;",
        ui("Fatto ›") + " (Done), to press when the item is done (the first three);",
        ui("Chiedi a Google ↗") + " (Ask Google), which opens a Google search in the browser with the question already "
        "typed: it explains how to do it on your model, often with a video.",
    ]) + \
    table(["Label to the right of the item", "Meaning"], [
        [ui("da fare sul telefono"), "(to do on the phone) Item still to be done."],
        [ui("✓ fatto"), "(done) Marked as done with " + ui("Fatto ›") + "."],
        [ui("✓ visto da Phonestra"), "(seen by Phonestra) Phonestra sees Wireless debugging turned on: the items before "
         "it are necessarily done."],
        [ui("me ne accorgo da solo"), "(I'll notice by myself) Phonestra is waiting to see Wireless debugging turned on "
         "on the network."],
        [ui("✓ acceso: ora il codice"), "(on: now the code) Wireless debugging is on: the pairing code is still needed."],
        [ui("scrivi il codice"), "(type the code) The phone shows the code: type it into the field of item 4."],
        [ui("associo…"), "(pairing…) Phonestra is pairing with the phone."],
        [ui("✓ collegato"), "(connected) Done: the phone is connected."],
    ], "«TAB» — The status of each item") + \
    note("the items that Phonestra can see on the network get checked by themselves. The " + ui("COSA VEDO IN RETE")
         + " box says what it sees at that moment: " + ui("○ Nessun telefono col Debug wireless acceso")
         + " (no phone with Wireless debugging on), " + ui("● Un telefono col Debug wireless acceso")
         + " (a phone with Wireless debugging on), " + ui("● Schermata del codice aperta") + " (code screen open).",
         "Automatic checkmarks.") + \
    tip("if you get stuck, the " + ui("TI SEI BLOCCATO?") + " box suggests taking a photo of the window and "
        "sending it to whoever recommended Phonestra: the window shows how far you have got.", "Getting help.")

FLUSSO = flow([
    ("Same network", "home Wi-Fi", "soft"),
    ("Developer", "options: 7 taps", "blue"),
    ("Protections", "only if present", "blue"),
    ("Wireless debug.", "turned on", "blue"),
    ("Code", "6 digits", "navy"),
    ("Connected", "once only", "green"),
], "«FIG» — The first connection, from the network to the connected phone", width=960)

S3 = FLUSSO + steps([
    "Check that the phone is on the same Wi-Fi network as the PC; press " + ui("Fatto ›") + ".",
    "Unlock the " + ui("Opzioni sviluppatore") + " (item 2); press " + ui("Fatto ›") + ".",
    "If the phone has " + ui("Blocco automatico") + " or " + ui("Protezione avanzata") + " turned on, turn them off "
    "(item 3); press " + ui("Fatto ›") + ".",
    "In " + ui("Opzioni sviluppatore") + " turn on " + ui("Debug wireless") + " and tap " + ui("Consenti")
    + " for the network. Phonestra notices by itself and opens item 4.",
    "Tap the " + ui("Debug wireless") + " label, then " + ui("Associa dispositivo con codice di associazione")
    + ": the phone shows a 6-digit code.",
    "Type the 6 digits into the field of item 4. There is nothing to press: with the sixth digit Phonestra pairs, "
    "connects and saves the phone.",
    "When " + ui("Fatto! «<nome del telefono>» è collegato via Wi-Fi.") + " (Done! “&lt;phone name&gt;” is connected "
    "via Wi-Fi.) appears, press " + ui("Inizia a usare il telefono") + " (Start using the phone): the drawer opens.",
]) + note("the phone's address and port don't need to be typed anywhere: Phonestra finds them on the network by itself.",
          "No addresses.")

S4 = table(["Message under the field", "What to do"], [
    [ui("Il campo si attiva quando apri la schermata del codice sul telefono."), "(The field activates when you open "
     "the code screen on the phone.) On the phone, open "
     + ui("Associa dispositivo con codice di associazione") + ". If the field stays gray, check that the phone "
     "and the PC are on the same network."],
    [ui("Vedo la schermata del codice: scrivi le 6 cifre, al resto penso io."), "(I can see the code screen: type the "
     "6 digits, I'll take care of the rest.) Type the 6 digits shown by the phone."],
    [ui("Codice non accettato (…). Riapri «Associa dispositivo con codice di associazione» sul telefono e scrivi il "
        "codice nuovo."), "(Code not accepted (…). Reopen “Pair device with pairing code” on the phone and type the "
     "new code.) The code was wrong or expired: on the phone, close and reopen the code screen, then type the new one."],
], "«TAB» — The code field") + \
    p("After pairing, Phonestra removes the expiry of this PC's authorization, so the connection doesn't "
      "have to be redone after a few days of not being used (" + rif("What stays switched on on the phone") + ").")

S5 = p("The " + ui("Android 10 o precedente? Collega col cavo") + " (Android 10 or earlier? Connect with the cable) "
       "button, at the bottom right, opens the fallback route: the first connection with the USB cable, designed for "
       "phones that don't have Wireless debugging. Afterward the cable is unplugged and Phonestra works over Wi-Fi "
       "as always.", lead=True) + \
    warn("the apps in the windows still require Android 14 or later: with an older phone Phonestra "
         "cannot show them, whichever route is used for the first connection.", "Android 14.") + \
    table(["Step", "What the window asks"], [
        [ui("Collega il cavo USB"), "(Connect the USB cable) Connect the phone to the PC with a cable that carries data "
         "(some cables only charge). If nothing happens: from the phone's notification shade, " + ui("USB")
         + " notification, choose " + ui("Trasferimento file") + " (File transfer)."],
        [ui("Attiva il Debug USB"), "(Turn on USB debugging) Unlock the " + ui("Opzioni sviluppatore")
         + " and turn on " + ui("Debug USB")
         + " (USB debugging). The window shows the steps for the phone's brand, to browse with the arrows under the "
         "drawn phone."],
        [ui("Consenti il collegamento"), "(Allow the connection) A request appears on the phone: check "
         + ui("Consenti sempre da questo computer") + " (Always allow from this computer) and tap " + ui("Consenti")
         + "."],
        [ui("Consenti la rete Wi-Fi"), "(Allow the Wi-Fi network) Phonestra turns on Wireless debugging; the first time "
         "on each network the phone asks for permission: tap " + ui("Consenti") + "."],
        [ui("Fatto"), "(Done) The phone is set up and connects over Wi-Fi: you can unplug the cable."],
    ], "«TAB» — The steps of the cable route") + ul([
        "The window moves on by itself: it notices every step done on the phone.",
        "Without the " + ui("Consenti sempre da questo computer") + " checkmark the cable works but Wi-Fi doesn't: "
        + ui("Manca «Consenti sempre»") + " (“Always allow” missing) appears and the request must be repeated by "
        "unplugging and replugging the cable.",
        "On Xiaomi, Redmi and Poco phones there is an extra step, " + ui("Un passaggio in più per Xiaomi") + " (One "
        "more step for Xiaomi): turn on " + ui("Debug USB (impostazioni di sicurezza)") + " (USB debugging (Security "
        "settings)), otherwise the apps are visible but don't respond to mouse and keyboard.",
        "If after 20 seconds the window is still at the first step, the " + ui("Cosa vede il PC") + " (What the PC "
        "sees) box appears with the list of USB devices: it is useful to photograph for someone helping remotely.",
    ])

S6 = p("After the first connection there is nothing more to do: at startup Phonestra looks for the phone on the "
       "network and connects by itself.", lead=True) + ul([
    "The phone must be on, unlocked, on the same Wi-Fi network and with " + ui("Debug wireless") + " turned on.",
    "Android sometimes turns Wireless debugging off by itself, for example when switching Wi-Fi networks. Just turn "
    "it back on in " + ui("Opzioni sviluppatore") + ": the code doesn't need to be redone, the PC stays paired.",
    "On a new network the phone asks again for " + ui("Consenti") + " for that network.",
]) + tip("many phones let you add " + ui("Debug wireless") + " to the Quick Settings tiles in the notification shade: "
         "that way it turns back on with one tap.", "Turning it back on quickly.")

CHAPTER = ("Connecting the phone", [
    ("Before you start", S1),
    ("The “Aggiungi un telefono” window", S2),
    ("Step by step", S3),
    ("The 6-digit code", S4),
    ("The cable route", S5),
    ("Later connections", S6),
])
