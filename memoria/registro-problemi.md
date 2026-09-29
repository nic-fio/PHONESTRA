# Registro dei problemi

Problema → causa → soluzione → stato, con il rimando ai dettagli. Le decisioni
e il loro perché stanno in `decisioni-utente.md`; le misure in
`prove-collegamento.md` (§ = sezione).

Stato: ✅ risolto · ⚠️ aggirato o in parte · ❌ aperto.

## 28 settembre 2026

### Audio e video

| Problema | Causa | Soluzione | Stato |
|---|---|---|---|
| Reel di Facebook senza audio | con il volume multimediale del telefono a 0 l'app di Facebook non avvia l'audio (Chrome sì) | durante il collegamento il volume è al massimo; il custode (e una copia in `telefoni.toml`) rimette il valore dell'utente anche dopo una caduta | ✅ §41 |
| Micro-interruzioni dell'audio (Facebook, Chrome) | nel **codice di scrcpy**: con «playback» vuoti di 50–120 ms (~1,5/s), con «output» interruzioni già nell'audio compresso; stessa cattura di Android col nostro codice: perfetta | componente nostro: cattura loopback, AAC, orari dal conteggio dei campioni, lettura a priorità −19 su thread separati | ✅ §41–47 |
| Orari dei pacchetti audio a raffica (1–3 ms e 30–40 ms invece di 20) | scrcpy con Opus usa l'ora di uscita dal codificatore | orari dal conteggio dei campioni (componente) e orari regolari sul PC | ✅ §41, §42 |
| Micro-interruzioni dei reel tornate con video e input dal componente | **ordine di avvio**: se la cattura audio parte prima che le sessioni iniziali del collegamento siano partite, il lettore di Facebook nelle finestre resta a secco (meccanismo interno di Android ancora da capire) | la cattura parte 5 s dopo lo specchio del drawer e riparte se lo specchio si ricrea; video e input nostri ora predefiniti | ✅ §48–49 |
| Micro-interruzioni tornate dopo un ricollegamento (telefono bloccato e sbloccato) | al ricollegamento l'audio non aspettava lo specchio nuovo: contatore degli specchi ancora del collegamento precedente | si aspetta uno specchio aperto con il servizio in uso, poi 5 s | ✅ §53 |
| Ipotesi «la compressione software causa le interruzioni» | — | verificata e scartata: AAC software perfetto quanto il PCM | ✅ §42 |
| Ipotesi «il colpevole è Java» | — | scartata: il registratore Samsung (Java) cattura pulito; la compressione la fanno i codificatori del telefono | ✅ §42, studio |
| Video di scrcpy a 24–37 fotogrammi/s, ripartenze di 1–2 s | scrcpy ricrea il codificatore a ogni «ricomincia video» | componente: `REQUEST_SYNC_FRAME`, 60/s | ✅ §43 (collegamento a Phonestra in corso) |
| A schermo fermo il fotogramma chiave a comando non arrivava | il codificatore Qualcomm ignora `repeat-previous-frame-after` | ridisegno forzato (Surface staccata e riattaccata) se entro 80 ms non esce | ✅ §46 |
| Canale video non chiuso dal telefono | `close()` su un socket con una lettura bloccata in un altro thread non lo chiude | `shutdownInput/Output` prima di `close` | ✅ §46 |
| «Task rimasto» nella prova del video | `removeTask` è asincrono | controllo ripetuto per 3 s | ✅ §46 |
| A telefono bloccato lo schermo virtuale non disegna | senza `ALWAYS_UNLOCKED`; sui Samsung il blocco fa comunque cadere il Debug wireless | nessuna: il collegamento si chiude e si riapre allo sblocco | ⚠️ §43 |
| Gesto «indietro» bloccato (scrcpy #6007, S23+/S26 Android 16) | schermo virtuale `ALWAYS_UNLOCKED` aperto durante blocco e sblocco | sui Samsung via Wi-Fi il caso non si presenta (al blocco tutto si chiude); provato: gesto funzionante | ⚠️ §43 |

### Input e appunti

| Problema | Causa | Soluzione | Stato |
|---|---|---|---|
| Avvisi degli appunti doppi | Samsung notifica ogni copia due volte | il componente scarta lo stesso avviso entro 0,5 s | ✅ §46 |
| Numeri di messaggio uguali per video e input (0x40–0x4f) | due agenti in parallelo | input spostato su 0x50–0x5f; test che vieta i doppioni | ✅ §46 |
| Avviso «ha incollato dagli appunti» (problema aperto delle specifiche) | incolla = appunti + tasto PASTE | l'input nostro rilegge gli appunti solo se il clip è suo; tastiera italiana UHID rimandata | ⚠️ studio input |

### Trasporto (nostro ADB)

| Problema | Causa | Soluzione | Stato |
|---|---|---|---|
| Latenza dei messaggi sotto carico fino a 115 ms; audio in ritardo sul video nostro | un blocco video da 1 MiB fa aspettare tutto sullo stesso TCP | blocchi da 64 KiB predefiniti: audio in sincrono, nessuna caduta in uso reale (la chiusura a 73 MB della prova di throughput non si è ripetuta) | ✅ §50, `adb.md` |
| Con il *delayed ack* il telefono rifiuta ogni canale | il telefono non registra il nostro annuncio, benché lo offra | predefinito riportato al trasporto di prima; da indagare | ❌ `adb.md` |

### Componente

| Problema | Causa | Soluzione | Stato |
|---|---|---|---|
| Copie dell'aiutante rimaste in `/data/local/tmp` | un processo interrotto di colpo non esegue il `rm` finale | il custode cancella le copie e il jar del servizio lo cancella appena partito | ✅ §43–44 |
| Il telefono non si accorge se il PC sparisce (Wi-Fi perso) | adbd sul Debug wireless non ha keepalive | battito ogni secondo; senza battito per 5 s il servizio esce e il custode ripulisce | ✅ §44 |
| Volume originale salvato come 15 invece di 3 | da capire (il valore salvato viene letto quando il volume è già al massimo) | **accettato dall'utente** (28 set 2026): «se Phonestra imposta il volume al massimo è ok, tanto il suono esce dalle casse del PC, e ci vuole poco per abbassarlo»; alla chiusura il volume può restare al massimo | ⚠️ §48 |
| Audio in ritardo di qualche decina di ms sul video dopo un po' d'uso | il margine audio cresceva a ogni ritardo e non scendeva più | discesa di un pacchetto di silenzio alla volta dopo 10 s di calma: **non scatta** (AAC quasi costante, il silenzio non si riconosce dalla dimensione); da fare: silenzio marcato dal telefono | ⚠️ §52 |
| Memoria del servizio ~145 MB | costo di partenza di ART | da confrontare con i processi di scrcpy | ❌ §44 |

### Progetto e repository

| Problema | Causa | Soluzione | Stato |
|---|---|---|---|
| Dati personali nella storia (schermate, numero di serie, rete Wi-Fi, indirizzi) | materiale di prova committato | storia ripulita e repository nuovo pubblico `nic-fio/PHONESTRA` con un solo commit iniziale; regola in `CLAUDE.md` | ✅ |
| Nome provvisorio che richiamava il marchio Android | scelto in fretta all'inizio | **Phonestra**, dopo aver scartato nomi registrati o già usati (PHONIX, LINDROID, MOBIX, MOBILE DECK, ANDESK, OBLO, FENESTRA) | ✅ decisioni |
| Nella finestra «Informazioni» un'icona generica del telefono, e nell'AppImage un'icona provvisoria | `phone-symbolic` del tema e `costruzione/phonestra.svg` fatti prima del logo | simbolo ufficiale (`grafica/icone/phonestra-256.png`), copiato nella cache e aggiunto al tema delle icone | ✅ |
| Configurazione `~/.config/Phonestra` azzerata a metà mattina | causa non trovata | telefono riassociato; se si ripete, indagare | ❌ |
| Pannello del telefono che non si spegne più dopo mezz'ora d'uso | tempo di spegnimento a 30 min e tocchi dal PC che non contano come attività: il telefono si addormentava, si bloccava, il collegamento cadeva e al ritorno Phonestra lo credeva «sbloccato a mano» senza controllare | tempo di spegnimento al massimo durante il collegamento; «sbloccato a mano» solo se il primo controllo trova il telefono sbloccato | ✅ §55 |
| Tempo di spegnimento cambiato dall'utente durante il collegamento sovrascritto alla chiusura | il custode rimetteva sempre il valore letto all'avvio | lo rimette solo se sul telefono c'è ancora quello di Phonestra; ai ricollegamenti vale quello nuovo dell'utente | ✅ §56 |
| Telefono che si blocca e collegamento che cade dopo un riavvio di Phonestra o del servizio | il custode riaccendeva il pannello addormentando e risvegliando il telefono (blocco) | riaccensione in Java con una copia del jar lasciata al custode | ✅ §51 |
| App sparite dal drawer | elenco interrotto a metà da una caduta del collegamento, preso per buono | l'aiutante chiude l'elenco con «fine»; senza, si rilegge al ricollegamento | ✅ §51 |
| Copie di lavoro degli agenti visibili a git | worktree in `.claude/` | `.claude/` in `.gitignore` | ✅ |
| Commit con un test fallito (`461b442`) | test non rilanciato dopo aver cambiato un predefinito | corretto nel commit successivo; prima di ogni commit build, test, clippy | ✅ |
