from build import dl, rif

VOCI = [
    ("Android", "Il sistema del telefono. Phonestra funziona con Android 14 e successivi."),
    ("AppImage", "Un programma per Linux in un file unico, che si avvia senza installarlo. Phonestra si distribuisce "
     "così."),
    ("Appunti", "Il testo copiato, pronto per essere incollato. Phonestra li passa tra telefono e PC ("
     + rif("Gli appunti") + ")."),
    ("Associazione", "Il primo incontro tra telefono e PC, col codice di 6 cifre: dopo, il telefono riconosce il PC "
     "da solo."),
    ("Avviso a comparsa", "La notifica del desktop che Phonestra mostra quando arriva una notifica sul telefono."),
    ("Debug wireless", "Un'impostazione delle Opzioni sviluppatore di Android che permette a un PC associato di "
     "collegarsi al telefono via Wi-Fi. Phonestra la usa per tutto."),
    ("Drawer", "La finestra principale di Phonestra, con le app, le notifiche e lo schermo del telefono ("
     + rif("Il drawer in breve") + ")."),
    ("In mano", "Il telefono che si sta usando con le mani: lo schermo resta acceso finché non si torna al PC ("
     + rif("Il telefono in mano") + ")."),
    ("Opzioni sviluppatore", "Un menu nascosto delle Impostazioni di Android; si sblocca toccando 7 volte il numero di "
     "build."),
    ("Pillola", "Il riquadro arrotondato in alto nel drawer con il nome e lo stato del telefono."),
    ("Preferiti", "Le app scelte col clic destro, mostrate in cima alla pagina App."),
    ("Rete ospiti", "Una rete Wi-Fi separata, per gli ospiti, che molti router offrono: i dispositivi collegati lì non "
     "si vedono tra loro, e Phonestra non trova il telefono."),
    ("Schermata protetta", "Una schermata che un'app non lascia mostrare fuori dal telefono: in Phonestra resta nera "
     "con un messaggio."),
    ("Schermo del telefono nel drawer", "La copia in diretta dello schermo principale del telefono, a destra del "
     "drawer, da usare col mouse."),
    ("Tempo di spegnimento", "Dopo quanto tempo senza tocchi il telefono spegne lo schermo e si blocca."),
    ("Toast", "Un messaggio breve che compare in basso nella finestra e sparisce da solo."),
]

CHAPTER = ("Glossario", [
    ("Termini", dl([(t, d) for t, d in VOCI], "gloss")),
])
