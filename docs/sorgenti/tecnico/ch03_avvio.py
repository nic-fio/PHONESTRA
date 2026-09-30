from build import arrow, box, c, fig, note, p, path, rif, seq, steps, table, text, ui

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

S1 = p("Il programma parte da " + c("main") + ", in " + c("src/bin/phonestra.rs") + ": prepara GStreamer, crea "
       "l'applicazione GTK e affida il telefono configurato a un " + c("Collegamento") + ", che vive in un compito "
       "tokio a sé.", lead=True) + AVVIO + \
    p("Un secondo avvio non apre un secondo collegamento: " + c("adw::Application") + " è unica per sessione e "
      + c("connect_activate") + " riporta in primo piano il drawer (lo riapre se era chiuso). Con un nome di pacchetto "
      "come argomento (" + c("phonestra com.android.chrome") + ") si apre subito anche quella app, ma solo al primo "
      "avvio: l'istanza già aperta non riceve l'argomento.") + \
    p("Ctrl+C e SIGTERM chiudono le finestre come farebbe l'utente, così il telefono viene rimesso a posto. Quando il "
      "drawer chiede di cambiare telefono (" + rif("Più telefoni") + "), " + c("main") + " alla fine rilancia il "
      "programma: " + c("$APPIMAGE") + " se c'è, altrimenti l'eseguibile corrente.")

STATI = fig(
    box(40, 40, 172, 54, "Cerco", "mDNS, poi Adb::wifi", "light")
    + box(365, 40, 172, 54, "Collegato", "si usano le app", "blue")
    + box(690, 40, 172, 54, "Bloccato", "le app non ricevono input", "amber")
    + box(200, 170, 172, 54, "Perso", "si riprova da soli", "dark")
    + box(690, 170, 172, 54, "Chiuso", "Phonestra finisce", "navy")
    + arrow(214, 60, 363, 60) + text(288, 52, "collegamento aperto", 11)
    + arrow(539, 58, 688, 58) + text(613, 50, "telefono bloccato", 11)
    + arrow(688, 80, 539, 80) + text(613, 98, "sbloccato", 11)
    + arrow(430, 96, 345, 168) + text(398, 140, "caduta", 11, "#334155", "400", "start")
    + arrow(110, 96, 235, 168) + text(196, 140, "non riuscito, o 30 s", 11, "#334155", "400", "start")
    + path([(198, 206), (70, 206), (70, 96)], "#475569", True) + text(134, 224, "attesa 2 → 10 s", 11)
    + arrow(374, 197, 688, 197, "#475569", True) + text(531, 189, "l'utente chiude Phonestra (da ogni stato)", 11),
    900, 240, "«FIG» — Gli stati del collegamento (Stato) e che cosa li cambia")

S2 = p(c("Collegamento::mantieni") + " è il cuore del programma. Gira in un compito tokio finché Phonestra resta "
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
]) + p("Lo stato si pubblica come " + c("Stato::{Cerco, Collegato, Bloccato, Perso, Chiuso}") + ". Tra un tentativo "
       "e l'altro l'attesa cresce da 2 a 10 s; «Riconnetti ora» (" + c("riconnetti_ora") + ") la interrompe.") + STATI

S3 = p("Un solo " + c("exec:") + " ogni 3 s porta blocco, chiamate e notifiche, e fa anche da controllo che il "
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

S4 = p("Quando l'utente chiude Phonestra, " + c("usa") + " esce dal giro e rimette il telefono com'era, un passo alla "
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

S5 = p("Phonestra usa un telefono alla volta (la scelta di più telefoni attivi insieme è stata scartata). Gli altri "
       "telefoni configurati compaiono nella barra laterale come «non attivo»; per passare a uno di loro il programma "
       "si chiude e riparte.", lead=True) + steps([
    "Il clic su un telefono non attivo chiede conferma se ci sono app aperte.",
    c("Telefoni::metti_primo") + " lo porta in cima a " + c("telefoni.toml") + ": è quello che si apre all'avvio.",
    "Il drawer segna " + c("cassetto::RIAVVIA") + " e chiude tutte le finestre, come una chiusura normale: il telefono "
    "di prima viene rimesso a posto.",
    c("main") + " rilancia il programma (" + c("$APPIMAGE") + " o l'eseguibile corrente), che si collega al nuovo "
    "telefono.",
]) + p(ui("Dimentica questo telefono…") + " fa lo stesso riavvio se restano altri telefoni; se non ne resta nessuno, "
       "Phonestra si chiude, e al prossimo avvio si apre «Aggiungi un telefono».")

S6 = p("Phonestra tiene i suoi dati in due cartelle del PC: la configurazione in " + c("~/.config/Phonestra") + ", le "
       "cose che si possono rifare in " + c("~/.cache/Phonestra") + ". " + c("telefoni.toml") + " e "
       + c("preferenze.toml") + " si leggono e si scrivono in " + c("configurazione.rs") + " (" + c("Telefoni")
       + ", " + c("Preferenze") + ").", lead=True) + \
    table(["File", "Contenuto"], [
        [c("~/.config/Phonestra/adbkey"), "Chiave privata RSA di Phonestra (permessi 600), diversa da quella di "
         + c("adb") + ": il telefono autorizza Phonestra come un computer a sé"],
        [c("~/.config/Phonestra/telefoni.toml"), "Per telefono: " + c("seriale") + ", " + c("nome") + ", "
         + c("modello") + ", " + c("android") + ", " + c("ultimo_indirizzo") + ", " + c("spegnimento_originale")
         + ", " + c("volume_originale") + ", " + c("preferiti") + "; il primo è quello che si apre all'avvio"],
        [c("~/.config/Phonestra/preferenze.toml"), c("esc_indietro") + ", " + c("avvisi") + ", " + c("solo_nome_app")
         + ", " + c("app_silenziate") + ", " + c("cartella_file") + ", " + c("cartella_ricevuti")
         + " (se manca: Scaricati); " + rif("Preferenze")],
        [c("~/.config/Phonestra/icone/"), "Icone delle app per gli avvisi del sistema"],
        [c("~/.cache/Phonestra/"), "Registro dei plugin di GStreamer, caricatori delle immagini e librerie di riserva "
         "dell'AppImage, il logo per la finestra «Informazioni» (" + c("icone/phonestra.png") + "): si può cancellare"],
    ], "«TAB» — I dati di Phonestra sul PC") + \
    p("Cancellare " + c("~/.config/Phonestra") + " riporta Phonestra allo stato iniziale. La cartella di "
      "configurazione segue " + c("$XDG_CONFIG_HOME") + "; screenshot, registrazioni e file ricevuti vanno nelle "
      "cartelle utente di XDG (Immagini, Video, Scaricati).") + \
    note("su un PC nuovo il telefono va associato di nuovo: chiave ADB e telefoni associati non sono nel "
         "repository.", "Un PC nuovo.")

CHAPTER = ("Avvio e ciclo di vita", [
    ("Avvio del programma", S1),
    ("Vita di un collegamento", S2),
    ("Il giro dei 3 s", S3),
    ("La chiusura", S4),
    ("Più telefoni", S5),
    ("Configurazione e dati sul PC", S6),
])
