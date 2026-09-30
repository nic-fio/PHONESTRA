import re

from build import HERE, c, file_map, p

J = "telefono/aiuto/src/phonestra/"


def capitoli():
    """Una riga per ogni capitolo dei due manuali, col suo titolo."""
    righe = []
    for cartella, nome in (("tecnico", "Manuale tecnico"), ("utente", "Manuale utente")):
        for f in sorted((HERE / cartella).glob("ch[0-9][0-9]_*.py")):
            titolo = re.search(r'^CHAPTER = \("([^"]+)"', f.read_text(), re.M).group(1)
            righe.append((f"docs/sorgenti/{cartella}/{f.name}", f"{nome}, capitolo {int(f.name[2:4])}: {titolo}"))
    return righe


GRUPPI = [
    ("Interfaccia", [
        ("src/avvisi.rs", "Notifiche del telefono come notifiche del sistema (D-Bus), icone delle app"),
        ("src/bin/phonestra.rs", c("main") + ": un processo per tutte le finestre, collegamento, drawer, appunti del "
         "PC, Ctrl+C e SIGTERM, chiusura e riavvio per cambiare telefono"),
        ("src/cassetto.rs", "Il drawer: pillola del telefono, pagine App, Notifiche e Preferenze, telefoni, strumenti, "
         "telefono disegnato con lo specchio, trasferimenti"),
        ("src/finestra.rs", "La finestra di un'app: vista (video, mouse, tastiera, zoom), sessioni, pannello, "
         "registrazione, screenshot"),
        ("src/foto.rs", c("PHONESTRA_FOTO") + ": immagini delle finestre per le prove dell'interfaccia"),
        ("src/prepara.rs", "«Aggiungi un telefono» senza cavo: elenco delle impostazioni, spunte da mDNS, codice a 6 cifre"),
        ("src/procedura.rs", "La procedura di riserva col cavo, per famiglia di marca"),
        ("src/ricevi.rs", "«Ricevi file…»: posti, Recenti, elenco e miniature delle cartelle del telefono, scelta dei file"),
    ]),
    ("Collegamento e dati", [
        ("src/app.rs", "Il jar incorporato (" + c("AIUTO") + "), l'aiutante, elenco delle app con la riga fine, "
         "sfondo, miniature, codificatori"),
        ("src/azioni.rs", "Installare, disinstallare, inviare file"),
        ("src/collegamento.rs", "Il telefono attivo: ricerca, collegamento, custode del collegamento, componente, giro "
         "dei 3 s, pannello «in mano», chiamate, chiusura"),
        ("src/configurazione.rs", "Cartelle XDG, chiave ADB, " + c("telefoni.toml") + ", " + c("preferenze.toml")),
        ("src/lib.rs", "I moduli della libreria, " + c("esecutore()") + ", diagnosi"),
        ("src/notifiche.rs", "Lettura di notifiche, batteria e rete dai " + c("dumpsys")),
        ("src/rete.rs", "mDNS scritto a mano: telefoni col Debug wireless, schermata del codice"),
        ("src/telefono.rs", "Collegamento col cavo o Wi-Fi con " + c("adb_client") + ", per la procedura di riserva e "
         "il primo collegamento"),
        ("src/usb.rs", "Telefoni col cavo letti da sysfs, senza permessi"),
    ]),
    ("Client ADB", [
        ("src/adb/abbina.rs", "Associazione col codice: TLS, SPAKE2, HKDF, AES-GCM"),
        ("src/adb/flusso.rs", "Parti pure del controllo di flusso: banner, OPEN, OKAY, saldo del delayed ack"),
        ("src/adb/messaggio.rs", "Messaggi ADB: intestazione di 24 byte, lettura e scrittura"),
        ("src/adb/mod.rs", c("Adb") + ", " + c("Canale") + ", " + c("Trasporto") + ": collegamento Wi-Fi, "
         "smistamento, controllo di flusso"),
        ("src/adb/shell.rs", "Il servizio " + c("shell,v2") + ": ingresso, uscita, errori, codice d'uscita"),
        ("src/adb/sync.rs", "Il protocollo " + c("sync:") + ": copiare file sul telefono, elencare le sue cartelle, "
         "ricevere file"),
        ("src/adb/tls.rs", "TLS del Debug wireless: certificato del client dalla chiave di Phonestra"),
    ]),
    ("Componente, lato PC", [
        ("src/appunti.rs", "Copie fatte sul telefono → appunti del PC, con limiti e rimbalzi"),
        ("src/audio_nostro.rs", "Canale audio, pacchetti, orari, margine, riproduzione GStreamer, copie per la registrazione"),
        ("src/componente.rs", c("Componente") + " e " + c("Condiviso") + ": avvio, preambolo, " + c("CIAO")
         + ", battito, smistamento, residui"),
        ("src/input_nostro.rs", c("InputNostro") + ": codifica di tocchi, rotellina, tasti, testo, appunti"),
        ("src/video_nostro/flusso.rs", "Opzioni dello schermo e lettura dei pacchetti video"),
        ("src/video_nostro/mod.rs", c("SessioneNostra") + ", " + c("ComandiVideo") + ", eventi, pannello"),
    ]),
    ("Componente sul telefono", [
        (J + "Aiuto.java", "Ingresso del jar: comandi brevi dell'aiutante e servizio"),
        (J + "Appunti.java", c("IClipboard") + " diretto: lettura, scrittura, sensibili, ascoltatore"),
        (J + "Audio.java", "Cattura, lettura e codifica dell'audio; strumento di misura"),
        (J + "Autotest.java", "Controllo all'avvio delle API nascoste, senza usarle"),
        (J + "CanaleAudio.java", "Il canale audio: cattura loopback, AAC, quattro thread"),
        (J + "Codifica.java", c("MediaCodec") + " hardware da " + c("Surface") + ", fotogramma chiave a comando"),
        (J + "Contesto.java", "Contesto Android per " + c("app_process") + ", pacchetto " + c("com.android.shell")),
        (J + "Custode.java", "Il custode del servizio: script di " + c("sh") + ", elenco delle azioni"),
        (J + "EventiApp.java", c("TaskStackListener") + ": orientamento, schermata protetta, task spostati o chiusi"),
        (J + "Input.java", "Messaggi di input, coda, iniezione, dita, scalatura"),
        (J + "Miniature.java", "Miniature di foto e video per «Ricevi file…» (" + c("BitmapFactory") + " ridotto, EXIF, "
         "fotogramma dei video)"),
        (J + "Nascoste.java", "Adattatori delle API nascoste, per riflessione"),
        (J + "Pannello.java", "Pannello fisico acceso o spento, frequenza a 60 Hz prima di spegnere, ripristino col custode"),
        (J + "Protetta.java", "Schermata protetta con " + c("captureDisplay") + " e " + c("containsSecureLayers")),
        (J + "Protocollo.java", "Formato dei messaggi del canale comandi e del preambolo"),
        (J + "Pulizia.java", "Chiusure in ordine inverso a fine prova"),
        (J + "Servizio.java", "Il servizio: segreto, socket, canali, battito, codici d'uscita, " + c("TIPI")),
        (J + "SessioneVideo.java", "Schermo virtuale o specchio, codificatore, ridimensionamento, ridisegno forzato"),
        (J + "Sistema.java", "Pezzi comuni: display manager degli schermi virtuali, avvio delle app, " + c("togliTask")
         + ", comandi interni, immagini"),
        (J + "Video.java", "Messaggi video, registro delle sessioni, canale " + c("video:<id>")),
    ]),
    ("Prove e misure (PC)", [
        ("src/adb/misura.rs", c("phonestra-prova throughput") + ": velocità e latenza del trasporto"),
        ("src/adb/prove.rs", "Il client ADB contro un finto adbd in memoria"),
        ("src/bin/prova/audio_componente.rs", c("phonestra-prova audio-componente")),
        ("src/bin/prova.rs", c("phonestra-prova") + ": i comandi delle prove da riga di comando"),
        ("src/misura_audio.rs", "Analisi dell'audio dello studio: righe di misura, livelli, WAV"),
        ("src/prova_input.rs", c("phonestra-prova input-componente") + ": appunti, tocchi, testo"),
        ("src/video_nostro/prova.rs", c("phonestra-prova video-componente")),
        ("tests/manuale.rs", "Questo manuale allineato ai sorgenti (lancia " + c("build.py --controlla") + ")"),
    ]),
    ("Prove e misure (telefono)", [
        (J + "Codificatori.java", "Elenco dei codificatori audio e video del telefono"),
        (J + "InputProva.java", "Comandi delle prove dell'input: schermo di prova, firma, appunti"),
        (J + "VideoProva.java", "Strumento di misura del video dello studio"),
    ]),
    ("Costruzione e strumenti", [
        ("costruzione/AppRun", "Avvio dell'AppImage: variabili di GTK e GStreamer, librerie di riserva"),
        ("costruzione/Containerfile", "Contenitore Ubuntu 22.04 con GTK 4.14, libadwaita 1.5, " + c("gst-plugin-gtk4")),
        ("costruzione/compila.sh", "Compila una libreria da tarball con meson, nel contenitore"),
        ("costruzione/prova-distribuzioni.sh", "L'AppImage su Ubuntu, Debian, Fedora e Arch in contenitori"),
        ("costruzione/raccogli.sh", "Raccoglie eseguibile e librerie nell'AppDir e crea l'AppImage"),
        ("telefono/aiuto/costruisci.sh", "Compila il componente: " + c("javac") + ", D8, " + c("phonestra-aiuto.jar")),
        ("docs/sorgenti/build.py", "Genera i due manuali: funzioni per testo, tabelle e figure SVG, numeri, controlli"),
    ] + capitoli()),
]

S1 = p("Ogni file di sorgente con le sue righe, contate a ogni generazione del manuale. Se un file manca dalla mappa, "
       "o la mappa cita un file che non c'è più, la generazione si ferma e " + c("cargo test") + " fallisce.",
       lead=True) + file_map(GRUPPI, "«TAB» — I file del progetto")

CHAPTER = ("Appendice B — Mappa dei file", [
    ("I file del progetto", S1),
])
