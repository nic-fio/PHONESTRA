from build import arrow, box, c, fig, note, p, rif, seq, steps, table, zone

S1 = p("Phonestra parla il protocollo di ADB da sé, in " + c("src/adb/") + ": niente server " + c("adb") + ", niente "
       "programmi da installare. Tutti i canali (comandi, audio, video di ogni finestra, shell) viaggiano su una sola "
       "connessione TCP cifrata.", lead=True) + \
    p("La libreria " + c("adb_client") + " legge tutti i canali dallo stesso collegamento senza smistare i messaggi: "
      "regge un comando alla volta, non video, audio e comandi insieme. Il client di " + c("src/adb/") + " ha un "
      "compito di lettura che smista ogni messaggio al canale giusto per identificativo locale, e ogni canale rispetta "
      "il controllo di flusso di ADB.") + \
    p(c("adb_client") + " resta per i comandi brevi fuori dal collegamento vero e proprio: il cavo USB ("
      + c("telefono.rs") + ") nella procedura di riserva e in " + c("phonestra-prova prepara") + ", e il primo "
      "collegamento Wi-Fi di «Aggiungi un telefono», che dopo l'associazione legge modello e versione e toglie la "
      "scadenza all'autorizzazione (" + c("telefono::Collegamento::wifi") + ", " + rif("Il primo collegamento") + ").")

S2 = p("ADB è fatto di pochi messaggi, tutti con la stessa forma: un'intestazione di 24 byte little-endian "
       "(comando, " + c("arg0") + ", " + c("arg1") + ", lunghezza dei dati, somma dei byte, comando xor "
       + c("0xffffffff") + ") seguita dai dati (" + c("messaggio.rs") + ").", lead=True) + table(["Comando", "Significato", "Uso in Phonestra"], [
    [c("CNXN"), "Presentazione: versione, " + c("max_payload") + ", funzioni",
     "Il nostro annuncia " + c("host::features=shell_v2,cmd,stat_v2") + " (e " + c("delayed_ack") + " se attivo)"],
    [c("STLS"), "Passaggio a TLS", "Sempre, sul Debug wireless"],
    [c("OPEN"), "Apre un canale verso un servizio (" + c("shell,v2,raw:…") + ", " + c("sync:") + ", "
     + c("localabstract:…") + ")", c("Adb::apri") + ", con 10 s di tempo massimo per la risposta"],
    [c("OKAY"), "Canale accettato, o dati confermati", "Controllo di flusso"],
    [c("WRTE"), "Dati su un canale", c("Canale::scrivi") + " / " + c("leggi")],
    [c("CLSE"), "Chiusura di un canale", c("Canale::chiudi") + ", " + c("Chiusore")],
], "«TAB» — I messaggi di ADB")

TLS = seq([("Phonestra", "Adb::wifi", "navy"), ("adbd", "Debug wireless", "dark")], [
    (0, 1, "TCP (5 s, TCP_NODELAY)"),
    (0, 1, "CNXN in chiaro: funzioni e max_payload"),
    (1, 0, "STLS", True),
    (0, 1, "STLS, poi TLS col certificato di Phonestra"),
    (1, 0, "CNXN dopo il TLS: max_payload comune, funzioni del telefono", True),
], "«FIG» — L'apertura del collegamento Wi-Fi", width=760)

S3 = p("Sul Debug wireless il collegamento comincia in chiaro e passa subito a TLS. " + c("Adb::wifi") + " lo apre "
       "in quattro passi; il telefono riconosce il PC dalla sua chiave pubblica autorizzata.", lead=True) + TLS + steps([
    "TCP verso l'indirizzo trovato con mDNS (5 s di tempo massimo, " + c("TCP_NODELAY") + ").",
    c("CNXN") + " in chiaro: il telefono legge le nostre funzioni e il " + c("max_payload") + " da qui, non dal "
    + c("CNXN") + " dopo il TLS.",
    "Il telefono risponde " + c("STLS") + "; noi rispondiamo " + c("STLS") + " e parte il TLS (" + c("tls.rs") + "). "
    "Il certificato del client è autofirmato con la chiave RSA di Phonestra (" + c("~/.config/Phonestra/adbkey")
    + "): il telefono riconosce il PC dalla chiave pubblica autorizzata. Il certificato del telefono non si controlla "
    "contro un'autorità (è autofirmato): l'identità la garantisce l'associazione.",
    "Dopo il TLS arriva il " + c("CNXN") + " del telefono: il suo " + c("arg1") + " è il minimo tra i due "
    + c("max_payload") + ", il testo contiene le funzioni del telefono.",
])

MUX = fig(
    box(20, 34, 190, 50, "Canale comandi", "il componente", "blue")
    + box(245, 34, 190, 50, "Canale audio", "uno alla volta", "blue")
    + box(470, 34, 190, 50, "Canali video:<id>", "uno per sessione", "blue")
    + box(690, 34, 190, 50, "exec: e shell,v2", "comandi brevi, servizio", "blue")
    + "".join(arrow(x - 14, 86, x - 14, 122) + arrow(x + 14, 122, x + 14, 88, "#475569", True)
              for x in (115, 340, 565, 785))
    + zone(20, 124, 860, 94, "Adb — si clona e si passa ovunque")
    + box(50, 152, 250, 52, "Adb::invia", "scritture, una alla volta (Mutex)", "navy")
    + box(328, 152, 250, 52, "leggi_sempre", "legge e smista per identificativo", "navy")
    + box(606, 152, 250, 52, "Posta", "gli OKAY che non possono aspettare", "navy")
    + arrow(452, 220, 452, 250)
    + box(262, 252, 380, 46, "Una connessione TCP + TLS", "", "dark")
    + arrow(644, 275, 698, 275)
    + box(700, 252, 180, 46, "adbd", "Debug wireless", "dark"),
    900, 310, "«FIG» — Tanti canali su una connessione: frecce piene le scritture, tratteggiate i dati smistati")

S4 = p("Tutti i canali di un collegamento condividono una sola connessione. Ogni canale ha il suo identificativo "
       "locale; chi legge smista, chi scrive aspetta il suo turno.", lead=True) + MUX + \
    p("Un " + c("Adb") + " si clona e si passa ovunque. Un compito legge dal socket e smista ("
       + c("leggi_sempre") + "); le scritture vanno direttamente sul socket, una alla volta sotto un " + c("Mutex")
       + " (" + c("Adb::invia") + "). Un secondo compito, la «posta», manda solo le conferme che non possono aspettare "
       "chi scrive (gli " + c("OKAY") + " della conferma alla lettura).") + \
    p(c("Adb::apri(servizio)") + " dà un " + c("Canale") + " con " + c("scrivi") + ", " + c("leggi")
      + " (annullabile), " + c("leggi_esatti") + ", " + c("leggi_tutto") + " e " + c("chiudi") + "; "
      + c("Canale::chiusore()") + " dà un oggetto che chiude il canale da un altro compito. " + c("Adb::esegui")
      + " è la scorciatoia per un comando breve (servizio " + c("exec:") + ") che restituisce l'uscita.") + \
    p("Senza " + c("delayed ack") + " ogni canale ha un solo " + c("WRTE") + " in volo: il successivo parte dopo "
      "l'" + c("OKAY") + ". I parametri del trasporto sono in " + c("Trasporto") + ":") + \
    table(["Valore", "Predefinito", "Variabile per le prove", "Perché"], [
        [c("delayed_ack"), "spento", c("PHONESTRA_ADB_DELAYED_ACK=1"), "Il 28 set 2026 adbd rifiutava ogni "
         + c("OPEN") + " quando era annunciato (" + rif("Appendice C — Problemi noti") + ")."],
        [c("max_payload"), "64 KiB", c("PHONESTRA_ADB_PAYLOAD=1m"), "Una sola connessione per tutti i canali: un "
         + c("WRTE") + " da 1 MiB tiene il filo ~200 ms e l'audio aspetta dietro al video; uno da 64 KiB ~13 ms. "
         "Con 64 KiB l'audio è in sincrono (misure §50)."],
        ["finestra", "256 KiB", c("PHONESTRA_ADB_FINESTRA=512k"), "Conta solo col " + c("delayed ack") + ": byte "
         "in volo per canale."],
    ], "«TAB» — I parametri del trasporto") + \
    p("I dettagli del " + c("delayed ack") + " (saldo, " + c("OKAY") + " di 4 byte, comportamento di adbd) sono in "
      + c("memoria/adb.md") + " e nelle parti pure di " + c("flusso.rs") + ", provate da " + c("adb/prove.rs")
      + " contro un finto adbd in memoria.")

S5 = p(c("shell.rs") + " avvia un processo senza terminale, con ingresso, uscita, errori e codice d'uscita "
       "separati. Sul canale viaggiano pacchetti " + c("id u8 · lunghezza u32 LE · dati") + ".", lead=True) + \
    table(["id", "Verso", "Contenuto"], [
        ["0", "PC → telefono", "Ingresso del processo"],
        ["1", "telefono → PC", "Uscita"],
        ["2", "telefono → PC", "Uscita d'errore"],
        ["3", "telefono → PC", "Codice d'uscita (1 byte)"],
        ["4", "PC → telefono", "Chiusura dell'ingresso"],
    ], "«TAB» — I pacchetti di " + c("shell,v2")) + \
    p("È il modo in cui parte il servizio di Phonestra: l'uscita porta la riga di pronto senza mescolarsi coi log, "
      "l'ingresso porta il segreto, il codice d'uscita dice perché il servizio è finito, e se il canale cade adbd "
      "manda SIGHUP al processo. Per i comandi brevi basta " + c("Adb::esegui") + " (servizio " + c("exec:") + ").")

S6 = p(c("sync.rs") + " implementa il protocollo " + c("sync:") + " nei due sensi.", lead=True) + \
    table(["Funzione", "Richieste", "Uso"], [
        [c("invia") + ", " + c("invia_a_blocchi"), c("SEND") + " con percorso e permessi, blocchi " + c("DATA")
         + ", " + c("DONE"), "copiare il componente in " + c("/data/local/tmp") + ", i file che l'utente invia al "
         "telefono e gli " + c(".apk") + " da installare (con avanzamento e annullamento)"],
        [c("elenca"), c("STA2") + ", poi " + c("LIS2"), "elencare una cartella del telefono per «Ricevi file…»: voci "
         + c("DNT2") + " da 72 byte dopo l'identificativo, dimensioni a 64 bit"],
        [c("e_cartella"), c("STA2"), "sapere se un percorso esiste ed è una cartella (i posti di «Ricevi file…»)"],
        [c("ricevi"), c("RECV"), "copiare un file sul PC scrivendo i blocchi " + c("DATA") + " direttamente nel "
         "file, senza tenerlo in memoria"],
    ], "«TAB» — Le funzioni di " + c("sync.rs")) + \
    note("nella memoria condivisa (FUSE) l'elenco non contiene «.» e «..»: una cartella vuota e una che non esiste "
         "darebbero la stessa risposta, per questo prima si chiede " + c("STA2") + ".")

S7 = p(c("abbina.rs") + " rifà " + c("adb pair") + " di Android 11+ dai sorgenti di Android e di BoringSSL: la "
       "schermata «Associa dispositivo con codice di associazione» del telefono mostra 6 cifre, l'utente le scrive in "
       "Phonestra.", lead=True) + steps([
    "TLS direttamente sulla porta di associazione, col certificato di Phonestra.",
    "Password = le 6 cifre + 64 byte esportati dal TLS (etichetta " + c("adb-label\\0") + "): nessuno può mettersi in mezzo.",
    "SPAKE2 su Ed25519 (" + c("curve25519-dalek") + "), Phonestra nel ruolo «alice».",
    "Dalla chiave comune, con HKDF-SHA256, una chiave AES-128-GCM (" + c("ring") + ").",
    "Scambio cifrato dei " + c("PeerInfo") + ": noi la chiave pubblica ADB di Phonestra, il telefono il suo identificativo.",
]) + p("Da quel momento il telefono accetta la chiave di Phonestra nei collegamenti Wi-Fi, come dopo un «Consenti "
       "sempre» col cavo.")

S8 = \
    p(c("rete.rs") + " costruisce e legge a mano i pacchetti DNS: domanda PTR per " + c("_adb-tls-connect._tcp.local")
      + " (collegamento) o " + c("_adb-tls-pairing._tcp.local") + " (schermata del codice) col bit QU, cioè risposta "
      "diretta al nostro socket. È il metodo che ha trovato il telefono quando la ricerca di " + c("adb") + " non "
      "vedeva niente.", lead=True) + \
    table(["Regola", "Perché"], [
        ["L'istanza si chiama " + c("adb-<seriale>-<suffisso>"), "Dal nome si riconosce il telefono salvato."],
        ["La porta viene dal record SRV", "Cambia a ogni riavvio del Debug wireless."],
        ["La domanda si ripete ogni secondo", "Qualche telefono risponde solo alla seconda."],
        ["Un SRV con durata 0 toglie il telefono trovato", "È l'«addio» di un servizio che si spegne."],
        [c("indirizzo_attivo") + ": ultimo indirizzo buono (800 ms), poi fino a tre ricerche da 3 s",
         "Il caso comune è veloce; un indirizzo si accetta solo se la porta risponde davvero."],
    ], "«TAB» — Come si cerca il telefono in rete")

CHAPTER = ("Il client ADB", [
    ("Perché un client nostro", S1),
    ("I messaggi di ADB", S2),
    ("Collegamento Wi-Fi e TLS", S3),
    ("Canali e controllo di flusso", S4),
    ("Il servizio shell,v2", S5),
    ("Copiare file: sync:", S6),
    ("Associazione col codice", S7),
    ("Ricerca in rete: mDNS", S8),
])
