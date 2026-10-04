from build import dl, rif

VOCI = [
    ("Android", "The phone's operating system. Phonestra works with Android 14 and later."),
    ("AppImage", "A Linux program in a single file, which starts without being installed. Phonestra is distributed "
     "this way."),
    ("Clipboard", "The copied text, ready to be pasted. Phonestra passes it between phone and PC ("
     + rif("The clipboard") + ")."),
    ("Desktop pop-up", "The desktop notification (pop-up alert) that Phonestra shows when a notification arrives on "
     "the phone."),
    ("Developer options", "A hidden menu in Android's Settings; it is unlocked by tapping the build number 7 times."),
    ("Device pairing", "The first meeting between phone and PC, with the 6-digit code: afterwards, the phone "
     "recognizes the PC on its own."),
    ("Drawer", "Phonestra's main window, with the phone's apps, notifications and screen ("
     + rif("The drawer at a glance") + ")."),
    ("In hand", "The phone while you are using it with your hands: the screen stays on until you go back to the PC ("
     + rif("The phone in hand") + ")."),
    ("Phone screen in the drawer", "The live copy of the phone's main screen, on the right of the drawer, to be used "
     "with the mouse."),
    ("Pill", "The rounded box at the top of the drawer with the phone's name and status."),
    ("Pinned apps", "The favorite apps, chosen with a right-click and shown at the top of the Apps page."),
    ("Protected screen", "A screen that an app does not allow to be shown outside the phone: in Phonestra it stays "
     "black, with a message."),
    ("Screen timeout", "How long the phone waits without being touched before it turns the screen off and locks."),
    ("Toast", "A short message that appears at the bottom of the window and disappears by itself."),
    ("Wi-Fi guest network", "A separate Wi-Fi network for guests, offered by many routers: devices connected to it "
     "cannot see each other, and Phonestra does not find the phone."),
    ("Wireless debugging", "A setting in Android's Developer options that lets a paired PC connect to the phone over "
     "Wi-Fi (in Phonestra's Italian interface: “Debug wireless”). Phonestra uses it for everything."),
]

CHAPTER = ("Glossary", [
    ("Terms", dl([(t, d) for t, d in VOCI], "gloss")),
])
