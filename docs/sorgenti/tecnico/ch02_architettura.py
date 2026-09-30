from build import arrow, box, c, fig, note, p, rif, table, text, zone

S1 = p("Phonestra è nato da poche regole, scritte prima del codice e rispettate in ogni pezzo. Spiegano scelte che "
       "altrimenti sembrerebbero strane, come un client ADB scritto da zero o un custode per ogni impostazione "
       "cambiata.", lead=True) + \
    table(["Principio", "Che cosa vuol dire"], [
        ["<b>Niente sul telefono</b>", "Nessuna app installata. Il componente si copia in " + c("/data/local/tmp")
         + " a ogni collegamento e si cancella alla fine."],
        ["<b>Il telefono torna com'era</b>", "Ogni impostazione cambiata ha un custode che la rimette, anche se il PC "
         "sparisce o il servizio muore di colpo (" + rif("I due custodi") + ")."],
        ["<b>ADB nostro</b>", "Nessun " + c("adb") + " da installare: TLS, associazione col codice, mDNS e canali "
         "multipli sono scritti qui (" + rif("Il client ADB") + ")."],
        ["<b>Codice nostro</b>", "Il componente del telefono è scritto da zero: niente scrcpy, niente codice di terzi "
         "da cui dipendere."],
        ["<b>Misurare prima di scrivere</b>", "Studio, poi misure sul telefono vero, poi codice un pezzo alla volta. "
         "Le misure restano in " + c("memoria/") + "."],
        ["<b>Un file, nessuna traccia</b>", "Un'AppImage che contiene tutto; sul PC solo " + c("~/.config/Phonestra")
         + " e " + c("~/.cache/Phonestra") + " (" + rif("Configurazione e dati sul PC") + ")."],
    ], "«TAB» — I principi che hanno formato Phonestra")

LATI = fig(
    zone(20, 12, 420, 262, "PC Linux (Rust)")
    + zone(460, 12, 420, 262, "Telefono Android 14+ (Java)")
    + box(36, 44, 170, 50, "Interfaccia", "cassetto · finestra · ricevi", "navy")
    + box(256, 44, 170, 50, "Pezzi lato PC", "audio · video · input · appunti", "blue")
    + box(36, 124, 170, 50, "Collegamento", "collegamento.rs", "blue")
    + box(256, 124, 170, 50, "Componente", "componente.rs (Condiviso)", "blue")
    + box(36, 204, 170, 50, "mDNS", "rete.rs", "dark")
    + box(256, 204, 170, 50, "Client ADB", "src/adb: TCP + TLS, canali", "dark")
    + box(480, 44, 380, 50, "Audio · Video · Input · Appunti · Pannello", "classi del servizio", "light")
    + box(480, 124, 180, 50, "Servizio", "app_process, uid 2000", "blue")
    + box(690, 124, 170, 50, "Custode", "sh con setsid", "amber")
    + box(480, 204, 180, 50, "adbd", "Debug wireless", "dark")
    + arrow(121, 96, 121, 122) + arrow(208, 69, 254, 69) + arrow(341, 96, 341, 122)
    + arrow(208, 149, 254, 149) + arrow(121, 176, 121, 202) + arrow(341, 176, 341, 202)
    + arrow(428, 229, 478, 229, "#003a90") + text(453, 262, "TLS", 11, "#003a90", "700")
    + arrow(570, 202, 570, 176) + arrow(570, 122, 570, 96)
    + arrow(662, 149, 688, 149, "#475569", True),
    900, 290, "«FIG» — Le parti di Phonestra e chi chiama chi")

S2 = p("Phonestra ha due lati: il programma sul PC, in Rust, e il componente sul telefono, in Java. Si parlano solo "
       "attraverso ADB, su un'unica connessione Wi-Fi cifrata.", lead=True) + LATI + \
    p("Sul PC ogni livello chiama solo quelli sotto: l'interfaccia non parla con ADB, il client ADB non sa niente di "
      "video o audio. Sul telefono c'è un solo processo per collegamento, il servizio, che serve tutte le finestre, "
      "lo schermo del drawer, l'audio e gli appunti. Il custode è un processo di shell a parte che resta vivo anche "
      "quando il servizio muore; la linea tratteggiata è il pipe con cui il servizio gli passa le azioni di ripristino.") + \
    table(["Strato", "Responsabilità", "Moduli principali"], [
        ["Interfaccia", "Le finestre GTK: drawer, finestre delle app, primo collegamento, avvisi del sistema",
         c("cassetto.rs") + ", " + c("finestra.rs") + ", " + c("ricevi.rs") + ", " + c("prepara.rs") + ", "
         + c("procedura.rs") + ", " + c("avvisi.rs")],
        ["Collegamento", "Trovare il telefono, tenerlo collegato, il giro dei 3 s, il custode del collegamento, la "
         "chiusura", c("collegamento.rs") + ", " + c("rete.rs") + ", " + c("notifiche.rs") + ", "
         + c("configurazione.rs")],
        ["Pezzi lato PC", "Audio, video, input e appunti sui canali del servizio",
         c("audio_nostro.rs") + ", " + c("video_nostro/") + ", " + c("input_nostro.rs") + ", " + c("appunti.rs")],
        ["Componente", "Avvio del servizio, canale comandi, smistamento dei messaggi", c("componente.rs")],
        ["Client ADB", "Messaggi, TLS, canali, controllo di flusso", c("src/adb/")],
        ["Telefono", "Servizio, custode, classi dei pezzi", c("telefono/aiuto/src/phonestra/")],
    ], "«TAB» — Gli strati del programma e i loro moduli") + \
    note("lo stato che interfaccia e collegamento condividono vive nel " + c("Collegamento") + ", in canali "
         + c("watch") + ": " + c("stato()") + ", " + c("guasto()") + ", " + c("info()") + ", " + c("notifiche()")
         + ", " + c("adb()") + ", " + c("componente()") + ". Drawer e finestre si iscrivono e ripartono da soli "
         "quando un valore cambia (" + rif("Vita di un collegamento") + ").", "Dove vive lo stato.")

S3 = p("Tre mondi girano insieme nel processo di Phonestra: il thread di GTK, il runtime tokio e GStreamer. Ognuno ha "
       "il suo compito, e si parlano solo con canali.", lead=True) + \
    table(["Dove", "Cosa gira", "Come si parla"], [
        ["Thread principale di GTK", "Tutta l'interfaccia; " + c("glib::spawn_future_local") + " per i compiti che "
         "toccano i widget", "Legge i " + c("watch") + " del " + c("Collegamento") + ", manda comandi con canali "
         + c("mpsc")],
        ["Runtime tokio (" + c("phonestra::esecutore()") + ")", "Collegamento, client ADB, componente, audio, video, "
         "input", c("watch") + " per gli stati, " + c("mpsc") + " per i messaggi, " + c("Notify") + " per i risvegli"],
        ["GStreamer", "Decodifica video e audio, riproduzione, registrazione", c("appsrc") + " riempiti dai compiti "
         "tokio"],
    ], "«TAB» — Thread e compiti") + \
    note("un " + c("Widget") + " non esce mai dal thread di GTK; un " + c("Adb") + ", un " + c("Condiviso") + " o un "
         + c("Mittente") + " si clonano e vanno dove servono.", "Regola pratica.")

CHAPTER = ("Architettura generale", [
    ("Principi architetturali", S1),
    ("I due lati e gli strati", S2),
    ("Thread e compiti", S3),
])
