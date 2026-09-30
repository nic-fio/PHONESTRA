from build import c, code, key, note, p, rif, steps, table, ui, ul

S1 = p("GTK4 e libadwaita, stile fedele a libadwaita (barra del titolo GNOME, liste a schede, pulsanti a pillola). "
       "Phonestra segue il tema chiaro o scuro del sistema: " + c("segui_tema") + " vale per tutte le finestre, la "
       "tavolozza " + c("SCURO") + " di " + c("cassetto.rs") + " per drawer, «Aggiungi un telefono» e procedura (le "
       "finestre delle app hanno regole loro); i mockup sono "
       "solo chiari. Nessun file " + c(".desktop") + " installato. Le proposte grafiche da cui viene ogni schermata "
       "stanno in " + c("mockup/") + "; le regole in " + c("memoria/interfaccia.md") + ".", lead=True) + \
    p(c("cassetto.rs") + " è la finestra principale, il drawer. Si iscrive a quattro " + c("watch") + " del "
      + c("Collegamento") + ": " + c("stato()") + ", " + c("guasto()") + ", " + c("info()") + " (batteria e rete) e "
      + c("notifiche()") + "; al componente arriva attraverso " + c("finestra::vista") + ".") + \
    table(["Parte", "Che cosa contiene"], [
        ["Barra del titolo", "Il simbolo di Phonestra, il nome e la pillola del telefono con lo stato del collegamento. "
         "Col componente guasto la pillola diventa rossa: «Phonestra non parte sul telefono»."],
        ["Menu della pillola", "Intestazione con nome, «modello · Android N», Wi-Fi e batteria; " + ui("Riconnetti")
         + ", " + ui("Rinomina…") + ", " + ui("Spegni il Debug wireless alla chiusura") + " (spenta, «In arrivo»), "
         + ui("Dimentica questo telefono…") + "."],
        ["Barra laterale", ui("App") + " e " + ui("Notifiche") + " (col contatore); sezione " + ui("Telefoni")
         + " col telefono attivo, gli altri telefoni configurati («non attivo») e " + ui("Aggiungi telefono")
         + "; sezione " + ui("Strumenti") + " con " + ui("Installa app…") + ", " + ui("Invia file…") + ", "
         + ui("Ricevi file…") + "; in fondo " + ui("Preferenze") + " e " + ui("Informazioni") + " (finestra "
         "«Informazioni» di libadwaita col logo)."],
        ["Pagina App", "Ricerca, Preferiti e Tutte le app. Scrivendo nel drawer si cerca; " + key("Invio")
         + " apre la prima app trovata."],
        ["Pagina Notifiche", "Le notifiche del telefono raggruppate per app, al massimo 2 per app e poi «altre N "
         "notifiche di …»; " + ui("Nascondi") + " e " + ui("Nascondi tutte") + " agiscono solo in Phonestra, e una "
         "notifica nascosta ricompare se si aggiorna."],
        ["Telefono disegnato", "A destra: ora, Wi-Fi, batteria e lo schermo vero del telefono (lo specchio, "
         "interattivo, con " + c("finestra::vista(…, SCHERMO, …)") + "; riceve i tasti solo dopo un clic). Ci si "
         "trascinano sopra i file (" + rif("Installare e inviare") + ")."],
        ["Velo", "Quando il telefono non si può usare, un velo spiega perché; con collegamento perso o componente "
         "guasto offre «Riconnetti ora»."],
    ], "«TAB» — Le parti del drawer") + \
    p("L'elenco delle app viene dall'aiutante (" + c("app::elenco") + ", comando " + c("app <lato>") + ") ed è valido "
      "solo se finisce con la riga " + c("fine\\t<n>") + ": un elenco interrotto da una caduta non si usa e si rilegge "
      "al ricollegamento (" + c("elenco_intero") + "). Il menu dell'app (clic destro) ha " + ui("Apri") + " (o "
      + ui("Porta in primo piano") + " se è già aperta), " + ui("Chiudi app") + " se è aperta, preferiti, "
      + ui("Informazioni sull'app") + " e " + ui("Disinstalla…") + " (spenta per le app di sistema).")

S2 = p(c("finestra.rs") + ": una finestra per app, con " + c("finestra::vista") + " al centro (video, mouse, tastiera) "
       "e intorno la barra con " + ui("Indietro") + ", " + ui("Screenshot") + ", " + ui("Registra") + " e il menu ⋮.",
       lead=True) + \
    table(["Comando", "Scorciatoia", "Che cosa fa"], [
        [ui("Screenshot"), "", "Salva in " + c("<Immagini di XDG>/Phonestra/<app> AAAA-MM-GG HH.MM.SS.png")
         + " e copia negli appunti"],
        [ui("Copia screenshot") + " (menu ⋮)", key("Ctrl", "Maiusc", "C"), "Copia soltanto"],
        [ui("Registra"), "", "MP4 in " + c("<Video di XDG>/Phonestra") + "; il pulsante diventa " + c("● m:ss")
         + " (" + rif("Registrazione") + ")"],
        [ui("Ruota") + " (menu ⋮)", key("Ctrl", "R"), "Scambia i lati della finestra; niente se è ingrandita o se registra"],
        [ui("Chiudi app") + " (menu ⋮)", key("Ctrl", "W"), "Chiude la finestra e toglie l'app dalle recenti"],
    ], "«TAB» — I comandi della finestra di un'app") + ul([
        "Il display virtuale segue la finestra (" + rif("Ridimensionamento") + ").",
        "App solo verticali (l'evento " + c("orientamento") + "): finestra a misura fissa 9:16, niente ingrandimento "
        "né bordi da trascinare; a tutto schermo l'app sta in una colonna con la forma del telefono.",
        "Collegamento perso o componente guasto: l'ultima immagine resta, sfocata, con «Riconnetti ora» e «Chiudi»; "
        "la sessione riparte da sola quando il collegamento torna e l'app ricompare dov'era. Col telefono bloccato "
        "cambia solo il sottotitolo.",
        "Schermata protetta: un messaggio al posto dell'immagine nera.",
        "Alla chiusura della finestra l'app si toglie dalle recenti del telefono (" + c("ComandiVideo::chiudi(true)")
        + "); se era lei a comandare la riproduzione (YouTube, Facebook), la si mette in pausa. Alla chiusura "
        "dell'ultima sessione il pannello si riaccende (" + rif("Il telefono in mano") + ").",
    ])

S3 = table(["Scheda", "Voce", "Valore in " + c("preferenze.toml")], [
    ["Finestre delle app", "Esc torna indietro", c("esc_indietro")],
    ["Notifiche", "Avviso a comparsa", c("avvisi")],
    ["Notifiche", "Solo il nome dell'app", c("solo_nome_app")],
    ["Notifiche", "App che possono avvisare (un interruttore per app)", c("app_silenziate")],
    ["File", "File inviati al telefono: una delle 6 cartelle di " + c("azioni::CARTELLE")
     + " (Download, Documenti, Immagini, Fotocamera, Musica, Video)", c("cartella_file")],
    ["File", "File ricevuti dal telefono: una cartella del PC; scegliere Scaricati salva «niente», così segue la "
     "cartella del sistema", c("cartella_ricevuti")],
    ["App del telefono", "Elenco delle app: " + ui("Aggiorna ora"), "—"],
], "«TAB» — La pagina Preferenze del drawer (" + c("configurazione::Preferenze") + ")")

S4 = p("Phonestra usa un telefono alla volta (la scelta di più telefoni attivi insieme è stata scartata). Gli altri "
       "telefoni configurati compaiono nella barra laterale come «non attivo».", lead=True) + steps([
    "Il clic su un telefono non attivo chiede conferma se ci sono app aperte.",
    c("Telefoni::metti_primo") + " lo porta in cima a " + c("telefoni.toml") + ": è quello che si apre all'avvio.",
    "Il drawer segna " + c("cassetto::RIAVVIA") + " e chiude tutte le finestre, come una chiusura normale: il telefono "
    "di prima viene rimesso a posto.",
    c("main") + " rilancia il programma (" + c("$APPIMAGE") + " o l'eseguibile corrente), che si collega al nuovo "
    "telefono.",
]) + p(ui("Dimentica questo telefono…") + " fa lo stesso riavvio se restano altri telefoni; se non ne resta nessuno, "
       "Phonestra si chiude, e al prossimo avvio si apre «Aggiungi un telefono».")

S5 = p(c("prepara.rs") + " è «Aggiungi un telefono» senza cavo: l'elenco delle impostazioni da attivare sul telefono "
       "(stessa rete Wi-Fi, Opzioni sviluppatore, eventuali protezioni, Debug wireless con «Associa dispositivo con "
       "codice di associazione»). Per ogni voce la parola da cercare nelle Impostazioni e «Chiedi a Google», che apre "
       "la Modalità IA di Google con la domanda già scritta.", lead=True) + \
    p("Le voci che si vedono in rete si spuntano da sole: il Debug wireless acceso (" + c("_adb-tls-connect")
      + ") e la schermata del codice aperta (" + c("_adb-tls-pairing") + "), che attiva il campo delle 6 cifre. Con "
      "il codice Phonestra si associa (" + c("adb::abbina") + "), si collega con " + c("adb_client") + ", toglie la "
      "scadenza all'autorizzazione (" + c("settings put global adb_allowed_connection_time 0") + ") e salva il "
      "telefono.") + \
    p(c("procedura.rs") + " è la riserva col cavo USB, per Android 10 o precedente (si apre da «Android 10 o "
      "precedente? Collega col cavo»). Avanza da sola guardando il cavo ogni secondo (" + c("usb.rs") + " legge "
      + c("/sys/bus/usb/devices") + " senza aprire il dispositivo) e riconosce questi casi:") + \
    table(["Stato del cavo (" + c("usb.rs") + ")", "Come si riconosce", "Passo mostrato"], [
        [c("SoloRicarica"), "nessuna interfaccia utile, produttore Android noto (" + c("PRODUTTORI_ANDROID")
         + ")", "scegliere «Trasferimento file» dalla notifica USB"],
        [c("DebugSpento"), "interfaccia MTP (" + c("06/01/01") + ") senza ADB", "attivare Opzioni sviluppatore e Debug USB"],
        [c("DebugAttivoSoloRicarica"), "ADB (" + c("ff/42/01") + ") senza MTP", "in «Solo ricarica» il permesso "
         "di systemd (uaccess) manca e l'accesso può essere negato: scegliere «Trasferimento file»"],
        [c("DebugAttivo"), "MTP e ADB", "«Consenti sempre», poi il passaggio al Wi-Fi"],
    ], "«TAB» — Gli stati del cavo") + \
    p("Dopo il cavo, " + c("telefono.rs") + " riprova 3 volte con 1 s di pausa (l'errore «got AUTH» dice che manca "
      "ancora il consenso), mette " + c("adb_allowed_connection_time 0") + " e " + c("adb_wifi_enabled 1") + " e "
      "aspetta fino a 30 s la conferma dell'utente. Dopo 20 s fermi al primo passo compare il riquadro «Cosa vede il "
      "PC» con l'elenco dei dispositivi USB, da fotografare per chi aiuta a distanza.") + \
    p("Le istruzioni per marca vengono da " + c("dati/istruzioni.toml") + ", incorporato alla compilazione. Le "
      "famiglie si scelgono cercando le parole di " + c("marche") + " nel produttore letto dal cavo; l'ultima, "
      "«Altri telefoni», ha " + c("marche") + " vuoto e fa da ripiego.") + \
    code("""
[[famiglia]]
nome = "Samsung"
marche = ["samsung"]             # parole cercate nel produttore
verificata = true                # percorsi provati su un telefono vero
percorso_build = ["Impostazioni", "Informazioni sul telefono", "Informazioni sul software"]
voce_build = "Numero build"
percorso_debug = ["Impostazioni", "Opzioni sviluppatore"]
grigio = "…"                     # cosa fare se la voce Debug USB è grigia
sicurezza = "…"                  # passaggio in più (Xiaomi: impostazioni di sicurezza)
[famiglia.prima]                 # un'impostazione da cambiare prima di tutto
percorso = ["Impostazioni", "Sicurezza e privacy", "Blocco automatico"]
voce = "Blocco automatico"
""", "toml", "dati/istruzioni.toml") + \
    note("con " + c("PHONESTRA_PROVA_PASSO") + " tutte e due le finestre mostrano un passo preciso senza telefono, "
         "per le prove dell'interfaccia: " + c("acceso") + ", " + c("codice") + ", " + c("fatto") + " per "
         + c("prepara.rs") + "; " + c("debug") + ", " + c("consenti") + ", " + c("xiaomi") + ", " + c("wifi") + ", "
         + c("fatto") + " per " + c("procedura.rs") + ".", "Prove senza telefono.")

S6 = p(c("notifiche.rs") + " legge " + c("dumpsys notification --noredact") + " (titolo e testo anche delle notifiche "
       "private) nel giro dei 3 s del collegamento, e batteria (" + c("dumpsys battery") + ", «in carica» con status 2 "
       "o 5) e rete (" + c("cmd wifi status") + ") ogni 30 s. Scarta le notifiche in corso ("
       + c("ONGOING_EVENT") + ", " + c("FOREGROUND_SERVICE") + "), i riassunti di gruppo (" + c("GROUP_SUMMARY")
       + ") e quelle senza titolo né testo; le ordina dalla più recente.", lead=True) + \
    p(c("avvisi.rs") + " trasforma le notifiche nuove in notifiche del sistema col servizio D-Bus "
      + c("org.freedesktop.Notifications") + " (GNOME, KDE, Xfce): le notifiche di GTK su GNOME funzionano solo per i "
      "programmi con un file " + c(".desktop") + ", che Phonestra non installa. Le notifiche «già viste» si fissano "
      "alla prima lettura; oggi però la prima lettura avviene col drawer appena aperto, quando l'elenco è ancora "
      "vuoto, e così alla prima lettura vera le notifiche già presenti sul telefono fanno un avviso ("
      + rif("Problemi noti") + "). Il clic su un avviso apre l'app. Le preferenze permettono di spegnerli, "
      "di mostrare solo il nome dell'app (il testo diventa «Nuova notifica») e di silenziare singole app. Le icone "
      "delle app per gli avvisi si scrivono in " + c("~/.config/Phonestra/icone/<pacchetto>.png") + ".")

S7 = table(["File", "Contenuto"], [
    [c("~/.config/Phonestra/adbkey"), "Chiave privata RSA di Phonestra (permessi 600), diversa da quella di "
     + c("adb") + ": il telefono autorizza Phonestra come un computer a sé"],
    [c("~/.config/Phonestra/telefoni.toml"), "Per telefono: " + c("seriale") + ", " + c("nome") + ", " + c("modello")
     + ", " + c("android") + ", " + c("ultimo_indirizzo") + ", " + c("spegnimento_originale") + ", "
     + c("volume_originale") + ", " + c("preferiti") + "; il primo è quello che si apre all'avvio"],
    [c("~/.config/Phonestra/preferenze.toml"), c("esc_indietro") + ", " + c("avvisi") + ", " + c("solo_nome_app")
     + ", " + c("app_silenziate") + ", " + c("cartella_file") + ", " + c("cartella_ricevuti") + " (se manca: Scaricati)"],
    [c("~/.config/Phonestra/icone/"), "Icone delle app per gli avvisi del sistema"],
    [c("~/.cache/Phonestra/"), "Registro dei plugin di GStreamer, caricatori delle immagini e librerie di riserva "
     "dell'AppImage, il logo per la finestra «Informazioni» (" + c("icone/phonestra.png") + "): si può cancellare"],
], "«TAB» — I dati di Phonestra sul PC") + \
    p("Cancellare " + c("~/.config/Phonestra") + " riporta Phonestra allo stato iniziale. La cartella di "
      "configurazione segue " + c("$XDG_CONFIG_HOME") + "; screenshot, registrazioni e file ricevuti vanno nelle "
      "cartelle utente di XDG (Immagini, Video, Scaricati).")

CHAPTER = ("L'interfaccia", [
    ("Il drawer", S1),
    ("Le finestre delle app", S2),
    ("Preferenze", S3),
    ("Più telefoni", S4),
    ("Il primo collegamento", S5),
    ("Notifiche e avvisi", S6),
    ("Dati sul PC", S7),
])
