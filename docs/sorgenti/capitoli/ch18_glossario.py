from build import c, dl, rif

VOCI = [
    ("adbd", "Il demone ADB del telefono: accetta i collegamenti del PC e avvia i servizi (" + c("shell") + ", "
     + c("sync:") + ", " + c("localabstract:") + ")."),
    ("Aiutante", "Il jar del componente usato per comandi brevi (elenco delle app, sfondo, miniature, misure)."),
    ("Annex B", "Formato del flusso H.264/H.265 con i codici d'inizio " + c("00 00 00 01") + " davanti a ogni unità."),
    ("app_process", "Il programma di Android che avvia codice Java fuori da un'app; lo usa anche la shell."),
    ("ART", "Android Runtime: la macchina virtuale che esegue il dex."),
    ("AudioPolicy", "API nascosta per instradare l'audio; col loopback manda il suono delle app a un registratore "
     "invece che all'altoparlante."),
    ("Componente", "Tutto ciò che Phonestra esegue sul telefono: il jar " + c("phonestra-aiuto.jar") + "."),
    ("Custode", "Processo di shell che rimette a posto il telefono quando finisce ciò a cui è legato ("
     + rif("I due custodi") + ")."),
    ("Debug wireless", "L'ADB via Wi-Fi di Android 11+, con TLS e associazione col codice."),
    ("Delayed ack", "Estensione di ADB con più dati in volo per canale e conferme che portano i byte ricevuti."),
    ("dex", "Il formato del codice Java compilato per Android (" + c("classes.dex") + "), prodotto da D8."),
    ("Drawer", "La finestra principale di Phonestra con le app del telefono (" + c("cassetto.rs") + ")."),
    ("In mano", "Stato del pannello: l'utente sta usando il telefono con le mani, il pannello resta acceso ("
     + rif("Il telefono in mano") + ")."),
    ("Loopback", "Cattura dell'audio che esce dalle app, con il telefono che intanto tace."),
    ("mDNS", "DNS sulla rete locale senza server: il telefono annuncia così il Debug wireless."),
    ("Pannello", "Lo schermo fisico del telefono, spento mentre si usano le app dal PC."),
    ("Schermo virtuale", "Un display in più, creato dal componente, dove gira l'app di una finestra."),
    ("Servizio", "Il processo di lunga durata del componente, uno per collegamento (" + c("phonestra-servizio") + ")."),
    ("Specchio", "La copia dello schermo principale del telefono, disegnata nel drawer."),
    ("Surface", "Il buffer grafico in cui lo schermo virtuale disegna e da cui il codificatore legge."),
    ("uid 2000", "L'utente della shell di ADB: i suoi permessi sono quelli del componente."),
]

CHAPTER = ("Glossario", [
    ("Termini", dl([(t, d) for t, d in VOCI], "gloss")),
])
