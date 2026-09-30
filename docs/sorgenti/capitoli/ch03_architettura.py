from build import arrow, box, c, fig, note, p, rif, seq, steps, table, text, zone

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

S1 = p("Phonestra ha due lati: il programma sul PC, in Rust, e il componente sul telefono, in Java. Si parlano solo "
       "attraverso ADB, su un'unica connessione Wi-Fi cifrata.", lead=True) + LATI + \
    p("Sul PC ogni livello chiama solo quelli sotto: l'interfaccia non parla con ADB, il client ADB non sa niente di "
      "video o audio. Sul telefono c'è un solo processo per collegamento, il servizio, che serve tutte le finestre, "
      "lo schermo del drawer, l'audio e gli appunti. Il custode è un processo di shell a parte che resta vivo anche "
      "quando il servizio muore; la linea tratteggiata è il pipe con cui il servizio gli passa le azioni di ripristino.")

AVVIO = seq([("main", "bin/phonestra.rs", "navy"), ("adw::Application", "una per sessione", "blue"),
             ("Collegamento", "collegamento.rs", "blue"), ("Drawer", "cassetto.rs", "light")], [
    (0, 0, "gst::init()"),
    (0, 1, "application_id io.github.nic_fio.Phonestra"),
    (1, 2, "primo_telefono()"),
    ("sep", "senza telefoni configurati: prepara::apri («Aggiungi un telefono») e basta"),
    (1, 2, "esecutore().spawn(mantieni())"),
    (1, 3, "cassetto::apri"),
    (3, 2, "si iscrive a stato, guasto, info, notifiche", True),
    ("nota", 1, "ultima finestra chiusa → Collegamento::chiudi, attesa di Stato::Chiuso, uscita"),
], "«FIG» — Da main al drawer", width=900)

S2 = AVVIO + \
    p("Un secondo avvio non apre un secondo collegamento: " + c("adw::Application") + " è unica per sessione e "
      + c("connect_activate") + " riporta in primo piano il drawer (lo riapre se era chiuso). Con un nome di pacchetto "
      "come argomento (" + c("phonestra com.android.chrome") + ") si apre subito anche quella app, ma solo al primo "
      "avvio: l'istanza già aperta non riceve l'argomento.") + \
    p("Ctrl+C e SIGTERM chiudono le finestre come farebbe l'utente, così il telefono viene rimesso a posto. Quando il "
      "drawer chiede di cambiare telefono (" + rif("Più telefoni") + "), " + c("main") + " alla fine rilancia il "
      "programma: " + c("$APPIMAGE") + " se c'è, altrimenti l'eseguibile corrente.")

S3 = p(c("Collegamento::mantieni") + " è il cuore del programma. Gira in un compito tokio finché Phonestra resta "
       "aperto e ricomincia da capo a ogni caduta.", lead=True) + steps([
    "<b>Trova il telefono</b>: prima l'ultimo indirizzo che ha funzionato, poi la ricerca mDNS del servizio "
    + c("_adb-tls-connect._tcp") + " col numero di serie salvato (" + c("rete::indirizzo_attivo") + "). La porta "
    "cambia a ogni accensione del Debug wireless, quindi va sempre riletta.",
    "<b>Apre il collegamento</b>: " + c("Adb::wifi") + " (TCP, STLS, TLS col certificato di Phonestra), con 30 s di "
    "tempo massimo.",
    "<b>Legge e salva i valori dell'utente</b> (tempo di spegnimento dello schermo e volume multimediale), anche in "
    + c("telefoni.toml") + ". Si leggono una volta sola per tutti i ricollegamenti; a ogni ricollegamento si guarda "
    "se l'utente ha cambiato il tempo di spegnimento nel frattempo.",
    "<b>Legge densità e forma dello schermo</b> (" + c("wm density; wm size") + ", prima i valori «Override» scelti "
    "dall'utente, poi quelli «Physical»): la densità serve ai display virtuali, la proporzione alla colonna delle app "
    "solo verticali.",
    "<b>Avvia il custode del collegamento</b>: un " + c("exec:") + " di shell che porta al massimo tempo di "
    "spegnimento e volume e li rimette quando il canale si chiude (" + rif("I due custodi") + ").",
    "<b>Pubblica</b> l'" + c("Adb") + " (" + c("watch") + ") e lo stato " + c("Collegato") + ": drawer e finestre "
    "ripartono da soli.",
    "<b>Avvia il componente</b> in un compito (" + c("gira_componente") + "): servizio sul telefono, poi " + c("Condiviso")
    + " pubblicato per finestre e drawer, poi l'audio (dopo lo specchio e 5 s, " + rif("L'ordine d'avvio") + ").",
    "<b>Ascolta gli appunti</b> del telefono (" + c("appunti::ascolta") + ").",
    "<b>Giro dei 3 s</b> finché il collegamento regge (sezione seguente).",
    "<b>Chiusura</b> quando l'utente chiude Phonestra (" + rif("La chiusura") + ").",
]) + p("Tra un tentativo e l'altro l'attesa cresce da 2 a 10 s; «Riconnetti ora» (" + c("riconnetti_ora")
       + ") la interrompe. Lo stato si pubblica come " + c("Stato::{Cerco, Collegato, Bloccato, Perso, Chiuso}") + ".")

S4 = p("Un solo " + c("exec:") + " ogni 3 s porta blocco, chiamate e notifiche, e fa anche da controllo che il "
       "telefono risponda: se non risponde entro 5 s il collegamento si considera caduto. Batteria e rete e, col "
       "telefono «in mano», " + c("lastUserActivityTime") + " sono comandi a parte, con 5 s di tempo massimo "
       "ciascuno: se falliscono, il collegamento non cade.", lead=True) + \
    table(["Comando", "Quando", "Serve a"], [
        [c("dumpsys window | grep -m1 -o 'isKeyguardShowing=[a-z]*'"), "ogni giro",
         "sapere se il telefono è bloccato (stato " + c("Bloccato") + ") e se l'ha sbloccato l'utente a mano"],
        [c("dumpsys telephony.registry | grep -o 'mCallState=[12]'"), "ogni giro",
         "chiamate in arrivo (1) e in corso (2); con due SIM c'è una riga per SIM (" + rif("Chiamate") + ")"],
        [c("notifiche::COMANDO_NOTIFICHE"), "ogni giro", "le notifiche del drawer (" + rif("Notifiche e avvisi") + ")"],
        [c("notifiche::COMANDO_INFO"), "al primo giro e poi ogni 10 (30 s)", "batteria e rete per il telefono disegnato"],
        [c("dumpsys power | grep -m1 lastUserActivityTime="), "solo col telefono «in mano»",
         "spegnere il pannello dopo il tempo di spegnimento senza tocchi (" + rif("Il telefono in mano") + ")"],
    ], "«TAB» — I comandi del giro dei 3 s") + \
    p("Le schermate protette non passano da qui: le segnala il componente, sessione per sessione ("
      + rif("Eventi delle app") + ").")

S5 = p("Quando l'utente chiude Phonestra, " + c("usa") + " esce dal giro e rimette il telefono com'era, un passo alla "
       "volta, con un tempo massimo per ciascuno.", lead=True) + steps([
    "Musica o video che suonano vanno in pausa (" + c("cmd media_session dispatch pause") + ", al massimo 3 s): "
    "altrimenti, tolta la cattura, ripartirebbero dall'altoparlante del telefono.",
    "L'" + c("Adb") + " si ritira; le finestre chiudono le loro sessioni e tolgono le app dalle recenti (si aspetta "
    "fino a 5 s).",
    "Il componente riceve " + c("FINE") + " e si chiude (al massimo 12 s); il suo custode riaccende il pannello.",
    "Si chiude il custode del collegamento, che rimette volume e tempo di spegnimento.",
    "Si controlla per 3 s che il tempo di spegnimento non sia più quello di Phonestra; se il custode non l'ha "
    "rimesso, lo si rimette direttamente. Se nemmeno così, resta scritto in " + c("telefoni.toml") + " e si rimette "
    "al collegamento successivo.",
])

S6 = table(["Dove", "Cosa gira", "Come si parla"], [
    ["Thread principale di GTK", "Tutta l'interfaccia; " + c("glib::spawn_future_local") + " per i compiti che "
     "toccano i widget", "Legge i " + c("watch") + " del " + c("Collegamento") + ", manda comandi con canali " + c("mpsc")],
    ["Runtime tokio (" + c("phonestra::esecutore()") + ")", "Collegamento, client ADB, componente, audio, video, input",
     c("watch") + " per gli stati, " + c("mpsc") + " per i messaggi, " + c("Notify") + " per i risvegli"],
    ["GStreamer", "Decodifica video e audio, riproduzione, registrazione", c("appsrc") + " riempiti dai compiti tokio"],
], "«TAB» — Thread e compiti") + \
    note("un " + c("Widget") + " non esce mai dal thread di GTK; un " + c("Adb") + ", un " + c("Condiviso") + " o un "
         + c("Mittente") + " si clonano e vanno dove servono.", "Regola pratica.")

CHAPTER = ("Architettura", [
    ("I due lati", S1),
    ("Avvio del programma", S2),
    ("Vita di un collegamento", S3),
    ("Il giro dei 3 s", S4),
    ("La chiusura", S5),
    ("Thread e compiti", S6),
])
