from build import c, note, p, rif, table, ui


S1 = p("The " + ui("Preferences") + " (Preferenze) page opens from the drawer's sidebar. Every change takes effect "
       "immediately (the language at the next start) and is saved automatically: there is no Save button.", lead=True) + \
    table(["Group", "Item", "What it does", "Default"], [
        [ui("APP WINDOWS") + " (FINESTRE DELLE APP)", ui("Esc goes back") + " (Esc torna indietro)", ui("The Esc key acts like Android's “Back”. "
         "Turn it off if an app uses Esc for something else.") + " (Il tasto Esc fa come «Indietro» di Android. Disattivalo "
         "se un'app usa Esc per altro.)", "on"],
        [ui("NOTIFICATIONS") + " (NOTIFICHE)", ui("Pop-up alert") + " (Avviso a comparsa)", "A system alert for every new notification on the phone ("
         + rif("Pop-up alerts") + ").", "on"],
        [ui("NOTIFICATIONS"), ui("App name only") + " (Solo il nome dell'app)", "No sender and no text in the alerts.", "off"],
        [ui("NOTIFICATIONS"), ui("Apps that can alert") + " (App che possono avvisare)", "One switch per app: which apps may raise alerts ("
         + rif("Choosing the alerts") + ").", ui("all ›") + " (tutte ›)"],
        [ui("FILES") + " (FILE)", ui("Files sent to the phone") + " (File inviati al telefono)", ui("The phone folder where they arrive.") + " (Cartella "
         "del telefono in cui arrivano.) One of " + ui("Download") + ", " + ui("Documents") + " (Documenti), " + ui("Pictures") + " (Immagini), "
         + ui("Camera") + " (Fotocamera), " + ui("Music") + " (Musica), " + ui("Movies") + " (Video).", ui("Download")],
        [ui("FILES"), ui("Files received from the phone") + " (File ricevuti dal telefono)", ui("The PC folder where they arrive.") + " (Cartella "
         "del PC in cui arrivano.) The button opens the " + ui("Where to save the files received from the phone") + " (Dove salvare i file ricevuti dal telefono) window.", ui("Downloads") + " (Scaricati)"],
        [ui("PHONE APPS") + " (APP DEL TELEFONO)", ui("App list") + " (Elenco delle app)", ui("It updates by itself; use the button if a newly installed app is missing.") + " (Si aggiorna da solo; usa il pulsante se manca un'app appena installata.) The "
         "button is " + ui("Update now") + " (Aggiorna ora).", "—"],
        [ui("LANGUAGE") + " (LINGUA)", ui("Interface language") + " (Lingua dell'interfaccia)", ui("Takes effect the next time Phonestra starts.")
         + " (Vale dal prossimo avvio di Phonestra.) One of " + ui("Automatic (system)") + " (Automatica (del sistema)), "
         + ui("Italiano") + ", " + ui("English") + ": the automatic choice follows the language of the PC (English "
         "unless the PC is set to Italian). The change applies at the next start of Phonestra.", ui("Automatic (system)")],
    ], "«TAB» — The Preferences page") + \
    note("the preferences apply to all phones. Favorites and the name, on the other hand, belong to each phone.",
         "For all phones.")

S2 = p("The preferences are kept in the file " + c("preferenze.toml") + ", in the folder " + c("~/.config/Phonestra")
       + " (" + rif("The folders on the PC") + "). There is no need to open it: the " + ui("Preferences") + " (Preferenze) page is "
       "enough.", lead=True) + \
    p("To go back to the initial preferences, close Phonestra and delete " + c("preferenze.toml") + ": at the next "
      "start every item goes back to its default value. The connected phone is not lost, because it is stored in "
      "another file.") + \
    note("if you choose the " + ui("Downloads") + " (Scaricati) folder again in " + ui("Files received from the phone") + " (File ricevuti dal telefono), "
         "Phonestra follows the desktop's downloads folder even if it changes in the future.", "Downloads.")

CHAPTER = ("Preferences", [
    ("The “Preferences” page", S1),
    ("Where preferences are saved", S2),
])
