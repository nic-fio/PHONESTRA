from build import c, note, p, rif, table, ui


S1 = p("The " + ui("Preferenze") + " (Preferences) page opens from the drawer's sidebar. Every change takes effect "
       "immediately and is saved automatically: there is no Save button.", lead=True) + \
    table(["Group", "Item", "What it does", "Default"], [
        [ui("FINESTRE DELLE APP") + " (app windows)", ui("Esc torna indietro") + " (Esc goes back)", ui("Il tasto Esc fa come «Indietro» di Android. "
         "Disattivalo se un'app usa Esc per altro.") + " (the Esc key acts like Android's Back; turn it off if an app "
         "uses Esc for something else)", "on"],
        [ui("NOTIFICHE") + " (notifications)", ui("Avviso a comparsa") + " (pop-up alert)", "A system alert for every new notification on the phone ("
         + rif("Pop-up alerts") + ").", "on"],
        [ui("NOTIFICHE") + " (notifications)", ui("Solo il nome dell'app") + " (only the app name)", "No sender and no text in the alerts.", "off"],
        [ui("NOTIFICHE") + " (notifications)", ui("App che possono avvisare") + " (apps that can alert)", "One switch per app: which apps may raise alerts ("
         + rif("Choosing the alerts") + ").", ui("tutte ›") + " (all)"],
        [ui("FILE") + " (files)", ui("File inviati al telefono") + " (files sent to the phone)", ui("Cartella del telefono in cui arrivano.") + " (the phone "
         "folder where they arrive) One of " + ui("Download") + ", " + ui("Documenti") + " (Documents), " + ui("Immagini") + " (Pictures), "
         + ui("Fotocamera") + " (Camera), " + ui("Musica") + " (Music), " + ui("Video") + ".", ui("Download")],
        [ui("FILE") + " (files)", ui("File ricevuti dal telefono") + " (files received from the phone)", ui("Cartella del PC in cui arrivano.") + " (the PC "
         "folder where they arrive) The button opens the " + ui("Dove salvare i file ricevuti dal telefono")
         + " (Where to save the files received from the phone) window.", ui("Scaricati") + " (Downloads)"],
        [ui("APP DEL TELEFONO") + " (phone apps)", ui("Elenco delle app") + " (app list)", ui("Si aggiorna da solo; usa il pulsante se manca un'app "
         "appena installata.") + " (it updates by itself; use the button if a newly installed app is missing) The "
         "button is " + ui("Aggiorna ora") + " (Update now).", "—"],
    ], "«TAB» — The Preferenze page") + \
    note("the preferences apply to all phones. Favorites and the name, on the other hand, belong to each phone.",
         "For all phones.")

S2 = p("The preferences are kept in the file " + c("preferenze.toml") + ", in the folder " + c("~/.config/Phonestra")
       + " (" + rif("The folders on the PC") + "). There is no need to open it: the " + ui("Preferenze") + " (Preferences) page is "
       "enough.", lead=True) + \
    p("To go back to the initial preferences, close Phonestra and delete " + c("preferenze.toml") + ": at the next "
      "start every item goes back to its default value. The connected phone is not lost, because it is stored in "
      "another file.") + \
    note("if you choose the " + ui("Scaricati") + " (Downloads) folder again in " + ui("File ricevuti dal telefono") + " (Files received from the phone), "
         "Phonestra follows the desktop's downloads folder even if it changes in the future.", "Scaricati.")

CHAPTER = ("Preferences", [
    ("The “Preferenze” page", S1),
    ("Where preferences are saved", S2),
])
