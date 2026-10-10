# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import c, fig, key, note, p, pill, rif, table, text, tip, ui, ul


def rett(x, y, w, h, fill, stroke="", r=8, extra=""):
    s = f' stroke="{stroke}"' if stroke else ""
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}"{s}{extra}/>'


def numero(x, y, n):
    return (f'<circle cx="{x}" cy="{y}" r="10" fill="#003a90"/>'
            + text(x, y + 4, str(n), 11, "#ffffff", "800", "middle", False))


def voce(x, y, t, colore="#334155", peso="400", size=11):
    return text(x, y, t, size, colore, peso, "start", False)


ICONE = ["#3b82f6", "#16a34a", "#d97706", "#8b5cf6", "#ef4444", "#0ea5e9", "#64748b"]


def icone(x, y, n, primo=0):
    s = ""
    for i in range(n):
        s += rett(x + i * 56, y, 32, 32, ICONE[(i + primo) % len(ICONE)], r=9)
        s += rett(x + i * 56 - 4, y + 38, 40, 5, "#cbd5e1", r=2)
    return s


DRAWER = fig(
    rett(30, 12, 840, 304, "#ffffff", "#e5e7eb", 12)
    # barra del titolo
    + rett(30, 12, 840, 42, "#eef2f7", "#e5e7eb", 12)
    + rett(46, 25, 16, 16, "#0050C0", r=4) + voce(70, 38, "Phonestra", "#003a90", "700", 13)
    + rett(335, 21, 230, 24, "#ffffff", "#cbd5e1", 12)
    + '<circle cx="352" cy="33" r="4" fill="#16a34a"/>'
    + voce(362, 37, "Phone  · connected via Wi-Fi", "#334155", "600", 11)
    # barra laterale
    + rett(38, 62, 168, 246, "#f8fafc", "#e5e7eb", 10)
    + rett(46, 70, 152, 22, "#3584e4", r=6) + voce(58, 85, "Apps", "#ffffff", "700")
    + voce(58, 110, "Notifications", "#334155", "600") + rett(170, 99, 20, 15, "#3584e4", r=7)
    + text(180, 110, "3", 9.5, "#ffffff", "700", "middle", False)
    + voce(52, 136, "MY PHONES", "#94a3b8", "700", 9.5)
    + '<circle cx="62" cy="152" r="3.5" fill="#16a34a"/>' + voce(72, 156, "Phone", "#334155", "600")
    + voce(150, 156, "active", "#64748b", "400", 9.5)
    + voce(58, 178, "+ Add phone", "#3584e4", "600")
    + voce(52, 202, "TOOLS", "#94a3b8", "700", 9.5)
    + voce(58, 222, "Install app…") + voce(58, 240, "Send files…") + voce(58, 258, "Receive files…")
    + voce(58, 284, "Preferences", "#334155", "600") + voce(58, 302, "About")
    # pagina App
    + rett(222, 64, 440, 26, "#ffffff", "#cbd5e1", 13) + voce(240, 81, "Search for an app", "#94a3b8")
    + rett(222, 98, 440, 72, "#f1f5f9", "", 12) + voce(236, 114, "FAVORITES", "#94a3b8", "700", 9.5)
    + icone(246, 122, 3)
    + '<circle cx="262" cy="166" r="2.5" fill="#3584e4"/>'
    + rett(222, 178, 440, 130, "#f1f5f9", "", 12) + voce(236, 194, "ALL APPS", "#94a3b8", "700", 9.5)
    + icone(246, 202, 7, 2) + icone(246, 254, 7, 4)
    # telefono
    + rett(690, 62, 156, 246, "#1e1e22", "", 22)
    + rett(697, 69, 142, 232, "#7fb8ff", "", 16)
    + voce(708, 86, "12:30", "#ffffff", "700", 10)
    + text(768, 180, "the real screen", 12, "#ffffff", "700", "middle", False)
    + text(768, 196, "of the phone", 12, "#ffffff", "700", "middle", False)
    # numeri
    + numero(582, 33, 1) + numero(214, 70, 2) + numero(670, 104, 3) + numero(856, 76, 4),
    900, 326, "«FIG» — The drawer: phone pill (1), sidebar (2), page (3), phone screen (4)")

S1 = p("The drawer is Phonestra's main window: it opens at startup and gathers the phone's apps, "
       "notifications, preferences and the phone's screen. The name comes from the Android <i>app drawer</i>, "
       "the place where Android keeps its apps.", lead=True) + DRAWER + \
    table(["No.", "Part", "What it is for", "Where"], [
        ["1", "Phone pill", "Name and connection status; a click opens the phone menu.",
         rif("The phone pill")],
        ["2", "Sidebar", "The " + ui("Apps") + ", " + ui("Notifications") + " and " + ui("Preferences") + " pages, the "
         "phones and the tools.", rif("The sidebar")],
        ["3", "Page", "The content of the chosen page: here the " + ui("Apps") + " page, with the search box, "
         + ui("FAVORITES") + " and " + ui("ALL APPS") + ".", rif("Opening an app")],
        ["4", "Phone screen", "The phone's main screen, live, to be used with the mouse.",
         rif("The phone screen in the drawer")],
    ], "«TAB» — The parts of the drawer") + ul([
        "The drawer follows the desktop's light or dark theme.",
        "When you type while the drawer is in the foreground, the text goes into the " + ui("Search for an app") + " search box: " + key("Enter") + " opens the first app found.",
        "Closing the drawer does not close the open apps; Phonestra quits when no window is left ("
        + rif("Closing Phonestra") + ").",
    ])

S2 = p("In the middle of the title bar is the phone pill: a colored dot, the phone's name and the "
       "connection status.", lead=True) + \
    table(["Dot", "Status in the pill", "In the sidebar", "Meaning"], [
        [pill("gray", "off"), ui("connecting…"), ui("connecting…"), "Phonestra is looking for the phone on the network."],
        [pill("green", "ok"), ui("connected via Wi-Fi"), ui("active"), "Everything works."],
        [pill("orange", "snooze"), ui("locked: unlock it"), ui("locked"), "The phone is locked: unlock "
         "it, and Phonestra reconnects by itself."],
        [pill("orange", "snooze"), ui("reconnecting…"), ui("reconnecting…"), "The connection dropped: "
         "Phonestra retries by itself."],
        [pill("red", "wait"), ui("Phonestra won't start on the phone"), ui("won't start"), "Connected, but the part of "
         "Phonestra that shows the apps does not start on the phone (" + rif("Common problems") + ")."],
        [pill("gray", "off"), ui("closed"), ui("closed"), "Phonestra is shutting down."],
    ], "«TAB» — The connection states") + \
    p("Clicking the pill opens the phone menu:") + \
    table(["Item", "What it does"], [
        ["Header", "The phone's name, model and Android version, Wi-Fi network and battery."],
        [ui("Reconnect"), "Tries to connect again right away, without waiting for the next attempt. It appears only when "
         "the phone cannot be used."],
        [ui("Rename…"), "Changes the phone's name in Phonestra (" + rif("Renaming a phone") + ")."],
        [ui("Turn off Wireless debugging on exit"), "Disabled, marked " + ui("Coming soon") + ": Phonestra does not currently "
         "turn off Wireless debugging on exit (" + rif("What stays switched on on the phone") + ")."],
        [ui("Forget this phone…"), "Removes the phone from Phonestra (" + rif("Forgetting a phone") + ")."],
    ], "«TAB» — The phone menu")

S3 = table(["Item", "What it does", "Where"], [
    [ui("Apps"), "The page with the phone's apps.", rif("Opening an app")],
    [ui("Notifications"), "The page with the phone's notifications; the number next to it says how many there are.",
     rif("The “Notifications” page")],
    ["<b>" + ui("MY PHONES") + "</b>", "The active phone, with its status; the other phones connected in the past, "
     "marked " + ui("not active") + ".", rif("Switching to another phone")],
    [ui("Add phone"), "Opens " + ui("Add a phone") + " to connect another phone.",
     rif("Adding another phone")],
    ["<b>" + ui("TOOLS") + "</b> (STRUMENTI)", "", ""],
    [ui("Install app…"), "Installs an app on the phone from an " + c(".apk") + " file.", rif("Installing an app")],
    [ui("Send files…"), "Copies files from the PC to the phone.", rif("Sending files to the phone")],
    [ui("Receive files…"), "Copies files from the phone to the PC.", rif("Receiving files from the phone")],
    [ui("Preferences"), "The preferences page.", rif("The “Preferences” page")],
    [ui("About"), "Phonestra's name, version and author.", ""],
], "«TAB» — The sidebar items")

VELO = table(["Title on the screen", "Text", "What to do"], [
    [ui("Connecting…"), ui("The phone must be on, unlocked and on the same Wi-Fi network."), "Wait; "
     "if it lasts, check the phone and the network (" + rif("Common problems") + ")."],
    [ui("Phone locked"), ui("Unlock it to continue: I'll reconnect on my own."), "Unlock the phone."],
    [ui("Connection lost"), ui("Retrying on my own in the background.") + " " + ui("If the phone is locked, unlock it.")
     + "",
     "Wait, or press " + ui("Reconnect now") + "."],
    [ui("Phonestra won't start on the phone"), "The phone cannot start the part of Phonestra that shows the "
     "apps; the reason is given in brackets.", "Press " + ui("Reconnect now") + "; if that is not enough, restart the phone."],
], "«TAB» — The messages on the phone screen in the drawer")

S4 = p("On the right of the drawer is the drawn phone. When the connection works, it contains the phone's real "
       "screen, live, even if the phone's physical screen is off.", lead=True) + ul([
    "You use it with the mouse just like the app windows: a click is a tap, a drag is a swiping finger ("
    + rif("Mouse and touchpad") + ").",
    "It is there for the things that are not apps: the Home screen, the notification shade, quick settings, recent "
    "apps, widgets. You open them as on the phone, with the mouse instead of your finger.",
    "It receives keystrokes only after a click on the screen; a click elsewhere sends the keys back to the app search.",
    "Dragging files onto the drawn phone sends them to the phone (" + rif("Sending files to the phone") + ").",
    "Below the screen, a card appears for a transfer in progress (" + rif("The transfer in progress") + ").",
]) + p("When the phone cannot be used, a veil over the screen explains why:") + VELO + \
    note("above the drawn phone's screen, the time is the PC's; battery and network are the phone's, "
         "updated every half minute.", "Time and battery.")

S5 = p("Short messages appear at the bottom of the window and disappear after a few seconds. In the drawer, for example:",
       lead=True) + ul([
    ui("Wait for the current transfer to finish") + ": a file transfer is already in progress.",
    ui("The phone is not connected") + ": the requested action needs the connection.",
    ui("Rereading the phone's apps…") + ": after " + ui("Update now") + " in the preferences.",
    ui("“<name>” added: you'll find it in “My phones”") + ": after " + ui("Add phone") + ".",
]) + tip("messages that end with the name of a folder (for example after a screenshot or a received file) "
         "tell you where the file is on the PC.", "Where the files are.")

CHAPTER = ("The drawer at a glance", [
    ("How the drawer is laid out", S1),
    ("The phone pill", S2),
    ("The sidebar", S3),
    ("The phone screen in the drawer", S4),
    ("Short messages", S5),
])
