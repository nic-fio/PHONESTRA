from build import fig, key, note, p, rif, steps, table, text, tip, ui, ul, warn


def rett(x, y, w, h, fill, stroke="", r=8, extra=""):
    s = f' stroke="{stroke}"' if stroke else ""
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}"{s}{extra}/>'


def filo(x1, y1, x2, y2):
    return (f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="#94a3b8" stroke-width="1.3"/>'
            f'<circle cx="{x2}" cy="{y2}" r="2.5" fill="#94a3b8"/>')


FINESTRA = fig(
    rett(330, 14, 240, 286, "#ffffff", "#cbd5e1", 12)
    + rett(330, 14, 240, 48, "#eef2f7", "#cbd5e1", 12)
    + rett(340, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + text(353, 43, "‹", 18, "#334155", "700", "middle", False)
    + text(426, 35, "App name", 12, "#0f172a", "700", "middle", False)
    + text(426, 51, "Telefono", 10, "#64748b", "400", "middle", False)
    + rett(476, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + rett(482, 33, 14, 10, "#475569", r=2)
    + rett(506, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + '<circle cx="519" cy="38" r="5.5" fill="#e01b24"/>'
    + rett(536, 25, 26, 26, "#ffffff", "#cbd5e1", 13) + text(549, 43, "⋮", 15, "#334155", "700", "middle", False)
    + rett(340, 72, 220, 218, "#f1f5f9", "#cbd5e1", 8, ' stroke-dasharray="4 3"')
    + text(450, 172, "the phone's app", 13, "#475569", "700", "middle", False)
    + text(450, 192, "click = tap, wheel = scroll", 11, "#64748b", "400", "middle", False)
    # etichette a sinistra
    + filo(300, 38, 340, 38) + text(292, 42, "Back (Esc)", 12, "#003a90", "700", "end")
    + filo(300, 96, 400, 52) + text(292, 92, "App name and", 12, "#003a90", "700", "end")
    + text(292, 108, "connection status", 12, "#003a90", "700", "end")
    + filo(300, 180, 340, 180) + text(292, 176, "The app: it resizes", 12, "#003a90", "700", "end")
    + text(292, 192, "along with the window", 12, "#003a90", "700", "end")
    # etichette a destra
    + filo(600, 96, 549, 51) + text(608, 92, "More (⋮): Ruota,", 12, "#003a90", "700", "start")
    + text(608, 108, "Copia screenshot, Chiudi app", 12, "#003a90", "700", "start")
    + filo(600, 146, 519, 51) + text(608, 150, "Record", 12, "#003a90", "700", "start")
    + filo(600, 190, 489, 51) + text(608, 194, "Screenshot", 12, "#003a90", "700", "start"),
    900, 310, "«FIG» — An app's window")

S1 = p("Each phone app opens in its own window. There are many ways to open one, and several apps can be kept "
       "open at the same time.", lead=True) + ul([
    "A click on the app's icon in the drawer's " + ui("App") + " page.",
    "Typing the name into the " + ui("Cerca un'app") + " (Search for an app) search box (just start typing with the drawer in front) and "
    "pressing " + key("Enter") + ": the first app found opens.",
    "From the " + ui("PREFERITI") + " (FAVORITES) card, at the top of the page, with the apps chosen as favorites.",
    "With a click on a notification, in the " + ui("Notifiche") + " (Notifications) page or in the system alert ("
    + rif("Notifications") + ").",
]) + p("If the app is already open, its window comes back to the front: each app has only one window. The list shows "
       "the apps that have an icon in the phone's app screen, including " + ui("Impostazioni") + " (Settings) and "
       + ui("Fotocamera") + " (Camera).") + \
    note("the app list is read at connection time and after every installation done by Phonestra. If an app "
         "just installed on the phone is missing, " + ui("Preferenze") + " (Preferences) › " + ui("Aggiorna ora") + " (Refresh now) rereads it.",
         "Newly installed apps.")

S2 = p("Right-clicking an app in the drawer opens its menu:", lead=True) + \
    table(["Item", "What it does"], [
        [ui("Apri") + " (Open)", "Opens the app in its window. If it is already open, the item becomes " + ui("Porta in primo piano")
         + " (Bring to front) and " + ui("aperta in una finestra") + " (open in a window) appears under the name."],
        [ui("Aggiungi ai preferiti") + " (Add to favorites) / " + ui("Togli dai preferiti") + " (Remove from favorites)", "Adds the app to or removes it from the "
         + ui("PREFERITI") + " card. Favorites are remembered for each phone; they are hidden during a search."],
        [ui("Informazioni sull'app") + " (App info)", "Opens Android's app information page in a window, " + ui("Informazioni · <nome dell'app>") + ": "
         "permissions, storage, notifications, force stop."],
        [ui("Chiudi app") + " (Close app)", "Closes the app's window (only if it is open)."],
        [ui("Disinstalla…") + " (Uninstall…)", "Uninstalls the app, after confirmation (" + rif("Uninstalling an app") + "). Disabled for system "
         "apps: " + ui("App di sistema: non si può disinstallare") + " (System app: cannot be uninstalled)."],
    ], "«TAB» — An app's menu")

S3 = p("An app's window has a bar at the top with a few buttons; everything else is the phone's app.",
       lead=True) + FINESTRA + \
    table(["Button", "Shortcut", "What it does"], [
        [ui("Indietro") + " (Back, left arrow)", key("Esc"), "Like Android's Back button."],
        ["Title", "", "The app's name and, below it, the phone's name or the connection status ("
         + ui("collegamento…") + ", " + ui("scollegato") + ", " + ui("Telefono bloccato: sbloccalo per continuare")
         + ")."],
        [ui("Screenshot (salvato e copiato)") + " (saved and copied)", "", "Saves an image of the app and copies it to the clipboard ("
         + rif("Screenshots and recording") + ")."],
        [ui("Registra lo schermo") + " (Record the screen)", "", "Starts and stops recording a video of the app."],
        [ui("Altri comandi") + " (More commands, ⋮) › " + ui("Ruota") + " (Rotate)", key("Ctrl", "R"), "Rotates the window: swaps width and "
         "height."],
        [ui("Altri comandi") + " (⋮) › " + ui("Copia screenshot") + " (Copy screenshot)", key("Ctrl", "Shift", "C"), "Copies the image "
         "of the app to the clipboard, without saving it."],
        [ui("Altri comandi") + " (⋮) › " + ui("Chiudi app"), key("Ctrl", "W"), "Closes the window and the app."],
    ], "«TAB» — The commands of an app's window") + \
    p("App windows are ordinary desktop windows: you can move them, tile them side by side, make them full "
      "screen, and they appear in the desktop's taskbar with the app's name.")

S4 = p("The app adapts to the window: when you widen the window, the app gets more room, as on a tablet; when you "
       "narrow it, it goes back to looking like the phone. The same happens when you maximize the window to full screen.", lead=True) + ul([
    ui("Ruota") + " (" + key("Ctrl", "R") + ") swaps the window's width and height: a tall, narrow window "
    "becomes wide and short. It does not work with a maximized window or during a recording.",
    "To enlarge the app's content (a map, a photo), use zoom: " + key("Ctrl") + " + wheel, "
    + key("Ctrl", "+") + " and " + key("Ctrl", "−") + ", or a pinch on the touchpad (" + rif("Mouse and touchpad") + ").",
]) + note("some apps accept only the phone's portrait shape (for example some social networks). For these the window "
          "has a fixed size: it cannot be maximized or resized by dragging its edges. If it was wide, it goes back "
          "to portrait by itself.", "Portrait-only apps.")

S5 = steps([
    "Close the window with the " + ui("×") + " at the top right, or " + ui("Altri comandi") + " › "
    + ui("Chiudi app") + ", or " + key("Ctrl", "W") + ".",
    "The app closes on the phone too: it is removed from the recent apps.",
    "If the app was playing music or a video, it is paused, so the phone does not keep playing on "
    "its own.",
]) + note("the app is not force-stopped: it keeps receiving its notifications, as when you close it from "
          "the recent apps on the phone.", "Closed, not stopped.")

VELO = table(["Title", "Text", "Buttons"], [
    [ui("Riconnessione…") + " (Reconnecting…)", ui("Riprovo da solo. Se il telefono è bloccato, sbloccalo: l'app torna qui dov'era.") + " (Retrying on my own. If the phone "
     "is locked, unlock it: the app will come back here, where it was.)",
     ui("Riconnetti ora") + " (Reconnect now), " + ui("Chiudi") + " (Close)"],
    [ui("Phonestra non parte sul telefono") + " (Phonestra won't start on the phone)", "The phone cannot start the part of Phonestra that shows the "
     "apps: " + ui("Riconnetti ora") + " to try again, and if that is not enough, restart the phone.",
     ui("Riconnetti ora") + ", " + ui("Chiudi")],
], "«TAB» — The veil over an app's window")

S6 = p("If the connection drops (the Wi-Fi is interrupted, the phone locks), the windows do not close. The last "
       "image of the app stays, blurred, with a veil that explains what is happening.", lead=True) + VELO + ul([
    "Phonestra retries by itself, every few seconds. " + ui("Riconnetti ora") + " retries right away.",
    "When the connection comes back, the app reappears in its window, where it was.",
    ui("Chiudi") + " closes the window.",
    "If the phone is only locked, the veil does not appear: the subtitle changes to "
    + ui("Telefono bloccato: sbloccalo per continuare") + " (Phone locked: unlock it to continue). After unlocking, Phonestra reconnects by itself.",
])

S7 = p("Some apps protect certain screens: banking apps, passwords, paid videos. Android does not let them "
       "be shown outside the phone.", lead=True) + \
    p("Instead of the black image, Phonestra shows " + ui("Schermata protetta") + " (Protected screen): " + ui("Questa app non permette "
      "di mostrare questa schermata fuori dal telefono. Le altre schermate dell'app funzionano normalmente.") + " (This app does not allow this screen "
      "to be shown outside the phone. The app's other screens work normally.)") + \
    tip("for protected screens, use the phone in hand: when you unlock it, its screen turns back on ("
        + rif("The phone in hand") + ").", "With the phone in hand.") + \
    warn("Phonestra does not get around protected screens: it is a security choice made by the apps and by Android.", "No tricks.")

CHAPTER = ("App windows", [
    ("Opening an app", S1),
    ("An app's menu", S2),
    ("Anatomy of a window", S3),
    ("Size, rotation and portrait-only apps", S4),
    ("Closing an app", S5),
    ("When the connection drops", S6),
    ("Protected screens", S7),
])
