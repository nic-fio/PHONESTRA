# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import key, note, p, rif, table, tip, ui, ul, warn


S1 = p("In the app windows, and on the phone screen in the drawer, the mouse acts as your finger.", lead=True) + \
    table(["On the PC", "On the phone"], [
        ["Click", "Tap."],
        ["Dragging with the left button", "A swiping finger: moves pages, drags objects."],
        ["Left button held down", "Long press of the finger."],
        ["Right click", "Long press: selects the word under the pointer and opens the Android menu (Copy, "
         "Select all…)."],
        ["Wheel, two-finger scrolling on the touchpad", "Scrolls the page at the point under the pointer."],
        [key("Ctrl") + " + wheel", "Zooms at the point under the pointer, like a two-finger pinch."],
        ["Pinch on the touchpad", "Zoom, like a two-finger pinch on the phone's screen."],
        ["Mouse “back” button (if present)", "Back."],
    ], "«TAB» — Mouse and touchpad") + \
    note("some apps have their own zoom with the plain wheel, without " + key("Ctrl") + ": maps, for "
         "example.", "Maps.")

S2 = p("The PC keyboard types into the app that has the focus. Some keys do extra things:", lead=True) + \
    table(["Key", "What it does"], [
        "In the apps",
        [key("Esc"), "Back (it can be turned off: " + ui("Preferences") + " › " + ui("Esc goes back") + ")."],
        [key("Enter") + ", " + key("Backspace") + ", " + key("Delete") + ", " + key("Tab"), "As on the keyboard of a "
         "phone or a tablet."],
        [key("←") + " " + key("→") + ", " + key("Home") + ", " + key("End"), "Move the cursor in the text."],
        [key("↑") + " / " + key("↓"), "Scroll the page by one step; while typing they move the cursor."],
        [key("Page Up") + " / " + key("Page Down"), "Scroll the page by one screen."],
        [key("Ctrl") + " + letter", "The app's shortcut: " + key("Ctrl", "A") + " selects all, "
         + key("Ctrl", "C") + " copies, " + key("Ctrl", "X") + " cuts, " + key("Ctrl", "Z") + " undoes (if the app "
         "supports them)."],
        [key("Ctrl", "V") + ", " + key("Shift", "Insert"), "Pastes the text copied on the PC into the app ("
         + rif("The clipboard") + ")."],
        [key("Ctrl", "+") + " / " + key("Ctrl", "−"), "Zoom at the center of the window."],
        "Window commands",
        [key("Ctrl", "R"), ui("Rotate") + ": swaps the window's width and height."],
        [key("Ctrl", "Shift", "C"), ui("Copy screenshot") + ": copies the image of the app to the clipboard."],
        [key("Ctrl", "W"), ui("Close app") + ": closes the window and the app."],
        "Desktop shortcuts",
        [key("Alt") + " + key", "They stay with the desktop (for example " + key("Alt", "F4") + " closes the window, "
         + key("Alt", "Tab") + " switches windows)."],
    ], "«TAB» — Keyboard and shortcuts") + \
    tip("in the drawer, keystrokes go to the app search. To type on the phone screen in the drawer, "
        "click the screen first.", "Keys in the drawer.")

S3 = p("The on-screen keyboard of the phone does not appear: you type with the PC's keyboard, using its key "
       "layout (Italian, if the PC is set to Italian).", lead=True) + ul([
    "Letters, numbers and common symbols reach the app as if they had been typed on the phone.",
    "Accented letters (à, è, ì, ò, ù) and less common symbols go through the phone's clipboard: Phonestra "
    "puts them in the phone's clipboard and pastes them into the app.",
    "Shortcuts with " + key("Ctrl") + " reach the app as they would from a keyboard connected to the phone.",
]) + warn("typing an accented letter replaces the phone's clipboard with that letter. If something "
          "had been copied on the phone to be pasted, paste it first.", "Phone clipboard.")

S4 = p("The clipboard works in both directions, automatically, for plain text only.", lead=True) + \
    table(["Direction", "How it works", "What does not get through"], [
        ["From the phone to the PC", "Text copied on the phone (also in a Phonestra window) is immediately in the "
         "PC's clipboard: you can paste it into any program.", "Text marked as sensitive (for example "
         "passwords copied from a password manager), very long text, images and files."],
        ["From the PC to the phone", "With " + key("Ctrl", "V") + " (or " + key("Shift", "Insert") + ") in a "
         "Phonestra window, the text copied on the PC goes into the app.", "Passwords copied from a password manager on the PC ("
         + ui("Password not sent to the phone") + "); text that is too long (" + ui("Text too long: use "
         "file transfer") + "); images and files."],
    ], "«TAB» — The clipboard in both directions") + ul([
        "If there is no text in the PC's clipboard, " + key("Ctrl", "V") + " shows " + ui("There is no text in the PC's clipboard") + ".",
        "Text copied on the PC goes to the phone only when you paste it, and only to the connected phone.",
        "For images and files, use " + ui("Send files…") + " and " + ui("Receive files…") + " ("
        + rif("Apps and files") + "). A Phonestra screenshot still ends up in the PC's clipboard as an image.",
    ]) + note("on some desktops (GNOME, for example) a program can change the clipboard only while one of its "
              "windows is active. If text copied on the phone cannot be pasted on the PC, click a Phonestra "
              "window and try again.", "GNOME.")

CHAPTER = ("Mouse, keyboard and clipboard", [
    ("Mouse and touchpad", S1),
    ("Keyboard and shortcuts", S2),
    ("Typing with the PC keyboard", S3),
    ("The clipboard", S4),
])
