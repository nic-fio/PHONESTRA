from build import c, dl, p, rif


def v(titolo):
    return " Vedi " + rif(titolo) + "."


A_L = [
    ("adbd", "Il demone ADB del telefono: accetta i collegamenti del PC e avvia i servizi (" + c("shell") + ", "
     + c("sync:") + ", " + c("localabstract:") + ")." + v("Il client ADB")),
    ("Aiutante", "Il jar del componente usato per comandi brevi (elenco delle app, sfondo, miniature, misure)."
     + v("Che cos'è il componente")),
    ("Annex B", "Formato del flusso H.264/H.265 con i codici d'inizio " + c("00 00 00 01") + " davanti a ogni unità."
     + v("Il canale video:<id>")),
    ("app_process", "Il programma di Android che avvia codice Java fuori da un'app; lo usa anche la shell."
     + v("Che cos'è il componente")),
    ("ART", "Android Runtime: la macchina virtuale che esegue il dex."),
    ("AudioPolicy", "API nascosta per instradare l'audio; col loopback manda il suono delle app a un registratore "
     "invece che all'altoparlante." + v("La ricetta")),
    ("Battito", "Il messaggio " + c("BATTITO") + ", ogni secondo nei due sensi: dopo 5 s di silenzio l'altra parte "
     "si considera sparita." + v("Battito e codici d'uscita")),
    ("Canale", "Una connessione logica dentro il collegamento ADB, verso un servizio del telefono; tutti i canali "
     "condividono la stessa connessione." + v("Canali e controllo di flusso")),
    ("Collegamento", "Il telefono attivo e la sua connessione ADB (" + c("collegamento.rs") + "), mantenuta finché "
     "Phonestra resta aperto." + v("Vita di un collegamento")),
    ("Componente", "Tutto ciò che Phonestra esegue sul telefono: il jar " + c("phonestra-aiuto.jar") + "."
     + v("Il componente sul telefono")),
    ("Custode", "Processo di shell che rimette a posto il telefono quando finisce ciò a cui è legato."
     + v("I due custodi")),
    ("Debug wireless", "L'ADB via Wi-Fi di Android 11+, con TLS e associazione col codice."
     + v("Collegamento Wi-Fi e TLS")),
    ("Delayed ack", "Estensione di ADB con più dati in volo per canale e conferme che portano i byte ricevuti."
     + v("Canali e controllo di flusso")),
    ("dex", "Il formato del codice Java compilato per Android (" + c("classes.dex") + "), prodotto da D8."
     + v("Compilare il componente")),
    ("Drawer", "La finestra principale di Phonestra con le app del telefono (" + c("cassetto.rs") + ")."
     + v("Il drawer")),
    ("Giro dei 3 s", "Il controllo periodico del collegamento: blocco, chiamate e notifiche in un solo comando."
     + v("Il giro dei 3 s")),
    ("In mano", "Stato del pannello: l'utente sta usando il telefono con le mani, il pannello resta acceso."
     + v("Il telefono in mano")),
    ("Loopback", "Cattura dell'audio che esce dalle app, con il telefono che intanto tace." + v("La ricetta")),
]

M_Z = [
    ("mDNS", "DNS sulla rete locale senza server: il telefono annuncia così il Debug wireless."
     + v("Ricerca in rete: mDNS")),
    ("Pannello", "Lo schermo fisico del telefono, spento mentre si usano le app dal PC." + v("Spegnere il pannello")),
    ("Preambolo", "I primi byte di ogni canale del servizio: il segreto e il tipo del canale."
     + v("Canali e preambolo")),
    ("Schermo virtuale", "Un display in più, creato dal componente, dove gira l'app di una finestra."
     + v("Sessioni: schermo virtuale e specchio")),
    ("Servizio", "Il processo di lunga durata del componente, uno per collegamento (" + c("phonestra-servizio") + ")."
     + v("Avvio del servizio")),
    ("Sessione", "Uno schermo virtuale o lo specchio, col suo canale " + c("video:<id>") + "."
     + v("Sessioni: schermo virtuale e specchio")),
    ("Specchio", "La copia dello schermo principale del telefono, disegnata nel drawer."
     + v("Sessioni: schermo virtuale e specchio")),
    ("Surface", "Il buffer grafico in cui lo schermo virtuale disegna e da cui il codificatore legge."
     + v("Codificatore e fotogramma chiave")),
    ("uid 2000", "L'utente della shell di ADB: i suoi permessi sono quelli del componente."
     + v("Sicurezza del servizio")),
]

CHAPTER = ("Glossario", [
    ("Termini A–L", p("Definizioni dei termini tecnici usati nel manuale. I rimandi indicano la sezione di "
                      "approfondimento.", lead=True) + dl(A_L, "gloss")),
    ("Termini M–Z", dl(M_Z, "gloss")),
])
