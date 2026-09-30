from build import flow, key, note, p, rif, steps, table, tip, ui


PERCORSO = flow([
    ("Download", "the AppImage", "soft"),
    ("Connect", "the phone, once", "blue"),
    ("Open", "an app from the drawer", "blue"),
    ("Use", "mouse and keyboard", "blue"),
    ("Close", "the phone is as it was", "navy"),
], "«FIG» — From the downloaded file to the first app on the PC")

S1 = p("This path takes you from a freshly downloaded Phonestra to the first app used on the PC, in five stages. Each "
       "stage points to the chapter that explains it in detail.", lead=True) + PERCORSO + \
    table(["Stage", "What to do", "Where"], [
        ["Download", "Download the AppImage, make it executable and start it.", rif("Installation and first start")],
        ["Connect", "Follow " + ui("Aggiungi un telefono") + " (Add a phone): four settings on the phone and a "
         "6-digit code.", rif("Connecting the phone")],
        ["Open", "One click on an app in the drawer.", rif("Your first app")],
        ["Use", "The mouse acts as your finger, the PC keyboard types into the app.", rif("Mouse, keyboard and clipboard")],
        ["Close", "Close all of Phonestra's windows.", rif("Closing Phonestra")],
    ], "«TAB» — The stages of the path")

S2 = steps([
    "With the phone unlocked and the drawer open, wait until the phone pill says "
    + ui("collegato via Wi-Fi") + " (connected via Wi-Fi) and the apps appear.",
    "Click an app, for example the messaging app. Or type its name and press " + key("Enter") + ".",
    "The app opens in its own window. The phone's screen is off (Phonestra turns it off as soon as it connects): the "
    "phone stays on and unlocked, and Phonestra uses it without the screen (" + rif("The phone's screen") + ").",
    "Use the app with the mouse: a click is a tap, the wheel scrolls, the PC keyboard types.",
    "To go back to the app's previous screen, press the " + ui("Indietro") + " (Back) button at the top left, "
    "or " + key("Esc") + ".",
    "To close the app, close its window: the app closes on the phone too.",
]) + tip("a blue dot appears under the icon of an app that is open in a window. A second click on the icon does not "
         "open another window: it brings the already open one to the front.", "The blue dot.")

S3 = table(["On the phone", "With Phonestra"], [
    ["Back button", "The window's " + ui("Indietro") + " (Back) button, " + key("Esc") + " or the mouse's “back” "
     "button."],
    ["Home screen", "The drawer: it is Phonestra's “Home”. The app windows are already in the desktop's taskbar."],
    ["Recent apps", "The desktop's taskbar with the open windows; or the phone screen in the drawer."],
    ["Notification shade and quick settings", "The phone screen in the drawer: drag down from the top with the mouse."],
    ["Volume buttons", "The PC's volume: the phone's audio comes out of the PC's speakers (" + rif("Audio from the PC")
     + ")."],
], "«TAB» — Where to find the phone's controls") + \
    note("the app windows deliberately have no Home and Recent apps buttons: in Phonestra each app is a "
         "PC window, and windows are managed by the desktop.", "Why Home and Recents are missing.")

CHAPTER = ("First steps", [
    ("The path", S1),
    ("Your first app", S2),
    ("Home, Back and the notification shade", S3),
])
