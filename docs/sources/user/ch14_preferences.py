# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

from build import c, note, p, rif, table, ui


S1 = p("The " + ui("Preferences") + " page opens from the drawer's sidebar. Every change takes effect "
       "immediately (the language at the next start) and is saved automatically: there is no Save button.", lead=True) + \
    table(["Group", "Item", "What it does", "Default"], [
        [ui("APP WINDOWS"), ui("Esc goes back"), ui("The Esc key acts like Android's “Back”. "
         "Turn it off if an app uses Esc for something else."), "on"],
        [ui("NOTIFICATIONS"), ui("Pop-up alert"), "A system alert for every new notification on the phone ("
         + rif("Pop-up alerts") + ").", "on"],
        [ui("NOTIFICATIONS"), ui("App name only"), "No sender and no text in the alerts.", "off"],
        [ui("NOTIFICATIONS"), ui("Apps that can alert"), "One switch per app: which apps may raise alerts ("
         + rif("Choosing the alerts") + ").", ui("all ›") + ""],
        [ui("FILES"), ui("Files sent to the phone"), ui("The phone folder where they arrive.") + " One of " + ui("Download") + ", " + ui("Documents") + ", " + ui("Pictures") + ", "
         + ui("Camera") + ", " + ui("Music") + ", " + ui("Movies") + ".", ui("Download")],
        [ui("FILES"), ui("Files received from the phone"), ui("The PC folder where they arrive.") + " The button opens the " + ui("Where to save the files received from the phone") + " window.", ui("Downloads") + ""],
        [ui("PHONE APPS"), ui("App list"), ui("It updates by itself; use the button if a newly installed app is missing.") + " The "
         "button is " + ui("Update now") + ".", "—"],
        [ui("LANGUAGE"), ui("Interface language"), ui("Takes effect the next time Phonestra starts.")
         + " One of " + ui("Automatic (system)") + ", "
         + ui("Italiano") + ", " + ui("English") + ": the automatic choice follows the language of the PC (English "
         "unless the PC is set to Italian). The change applies at the next start of Phonestra.", ui("Automatic (system)")],
    ], "«TAB» — The Preferences page") + \
    note("the preferences apply to all phones. Favorites and the name, on the other hand, belong to each phone.",
         "For all phones.")

S2 = p("The preferences are kept in the file " + c("preferenze.toml") + ", in the folder " + c("~/.config/Phonestra")
       + " (" + rif("The folders on the PC") + "). There is no need to open it: the " + ui("Preferences") + " page is "
       "enough.", lead=True) + \
    p("To go back to the initial preferences, close Phonestra and delete " + c("preferenze.toml") + ": at the next "
      "start every item goes back to its default value. The connected phone is not lost, because it is stored in "
      "another file.") + \
    note("if you choose the " + ui("Downloads") + " folder again in " + ui("Files received from the phone") + ", "
         "Phonestra follows the desktop's downloads folder even if it changes in the future.", "Downloads.")

CHAPTER = ("Preferences", [
    ("The “Preferences” page", S1),
    ("Where preferences are saved", S2),
])
