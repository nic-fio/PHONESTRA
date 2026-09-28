# Componente nostro: lo scheletro (fase 1, 28 set 2026)

Il servizio di lunga durata che sostituirà scrcpy un pezzo alla volta
(`decisioni-utente.md`, «Via da scrcpy»; piano in `studio/README.md`, specifica
principale `studio/sistema.md`). Questa è **solo l'infrastruttura**: processo,
canali, segreto, battito, custode, adattatori delle API nascoste con autotest.
Audio, video e input arriveranno come nuovi tipi di canale e nuovi messaggi.
Phonestra (l'interfaccia) non lo usa ancora: scrcpy resta in uso.

Legenda come negli studi: ✅ verificato (sul PC, su codice o documentazione),
🔶 ipotesi da verificare sul telefono. **Nessuna parte è ancora stata provata
sul telefono.**

## 1. Pezzi e file

| Dove | File | Cosa fa |
|---|---|---|
| telefono | `telefono/aiuto/src/phonestra/Servizio.java` | il servizio: segreto, socket, canali, battito, guardiano, codici d'uscita |
| telefono | `Protocollo.java` | formato dei messaggi del canale comandi e del preambolo |
| telefono | `Custode.java` | avvia il custode (`sh` con `setsid`) e gli manda l'elenco delle azioni di ripristino |
| telefono | `Autotest.java` | prova all'avvio quali API nascoste ci sono, senza usarle |
| telefono | `Nascoste.java` | adattatori delle API nascoste (spostati da `Sistema.java`): servizi, varianti di firma, ripieghi |
| telefono | `Contesto.java` | contesto di sistema e della shell, preparato una volta sola (era `Aiuto.contesto()`) |
| PC | `src/componente.rs` | `Componente`: avvio, pronto, canale comandi, `CIAO`, battito, altri canali, chiusura |
| PC | `src/adb/shell.rs` | servizio `shell,v2,raw:` nel nostro client ADB |
| PC | `src/bin/prova.rs` | `phonestra-prova servizio [secondi] [--sparisci]` |

Il servizio è un comando dell'aiutante (`phonestra.Aiuto servizio`): stesso
jar, stessa compilazione (`telefono/aiuto/costruisci.sh`).

## 2. Architettura

```
PC (Rust)                                             Telefono (uid 2000)
Adb (TCP+TLS)
 ├─ sync:  copia phonestra-servizio-<casuale>.jar
 ├─ shell,v2,raw: export CLASSPATH=…; exec app_process / --nice-name=phonestra-servizio phonestra.Aiuto servizio
 │     ingresso: il segreto (32 cifre esadecimali)  ──►  Servizio (Java)
 │     uscita:   riga di pronto col nome del socket ◄──   ├─ Custode: setsid sh -c '…' phonestra-custode <jar>
 │     errori:   log  · codice d'uscita                   │    (legge dal pipe l'elenco delle azioni)
 ├─ localabstract:phonestra_<32 hex>  «comandi»     ◄─►   ├─ canale comandi (CIAO, BATTITO, FINE, …)
 └─ localabstract:phonestra_<32 hex>  «audio», «video:<id>» (futuri)
```

## 3. Formati

**Riga di pronto** (uscita del processo, una riga):
`phonestra-servizio pronto protocollo=1 socket=phonestra_<32 hex> pid=<pid>`.
Se l'avvio fallisce: `phonestra-servizio errore <causa>` e codice d'uscita 1.
Il PC accetta solo nomi `phonestra_` + 32 cifre esadecimali minuscole (il nome
finisce in un servizio ADB).

**Preambolo di ogni canale** (PC → servizio, appena aperto il socket):
`segreto (16 byte) · lunghezza del tipo u8 · tipo ASCII`. Tipi: `comandi`;
in futuro `audio`, `video:<id>` (il servizio sceglie il gestore dalla parte
prima dei due punti, tabella `Servizio.TIPI`). Il servizio legge il preambolo
con 3 s di tempo massimo; segreto sbagliato, uid diverso da 2000 o tipo
sconosciuto: il socket si chiude senza risposta.

**Messaggi del canale comandi**: intestazione di 8 byte big-endian
`tipo u8 · bandiere u8 · id u16 · lunghezza u32`, poi il contenuto (massimo
16 MB, oltre è un flusso rovinato e il canale si chiude). `id` lega la risposta
alla domanda (0 = spontaneo); bandiera `0x01` = risposta.

| Tipo | Nome | Direzione | Contenuto |
|---|---|---|---|
| 0x01 | `CIAO` | servizio → PC, primo messaggio | righe `chiave=valore` UTF-8 (sotto) |
| 0x02 | `BATTITO` | nei due sensi, ogni secondo | vuoto |
| 0x03 | `FINE` | PC → servizio; risposta uguale | vuoto; dopo la risposta il servizio esce (codice 0) |
| 0x04 | `ERRORE` | servizio → PC, come risposta | testo (per esempio «tipo sconosciuto 0x2a») |
| 0x10 | `PROVA_CUSTODE` | PC → servizio; risposta uguale | risposta: il percorso del file di prova |

Numerazione: 0x01–0x0f infrastruttura, 0x10–0x1f prove e diagnosi, dal 0x20 i
pezzi (audio, video, input…). Estendibile così: un tipo nuovo che il servizio
non conosce riceve `ERRORE` (il PC capisce che il jar è vecchio); il PC passa
i tipi che non conosce a `Componente::ricevi`. `protocollo` cambia solo se un
messaggio esistente cambia significato. Contenuti: testo `chiave=valore` per
le cose piccole; per i dati strutturati dei pezzi futuri (app, notifiche) lo
studio propone JSON, per tocchi e tasti binario compatto.

**CIAO**: `protocollo`, `android`, `sdk`, `produttore`, `modello`, `pid`,
`avvio_ms` (dall'inizio di `main` al pronto, senza l'avvio di ART),
`autotest_ms`, poi `autotest.<nome>=ok <dettagli>` oppure
`autotest.<nome>=manca <causa>`.

## 4. Ciclo di vita

1. Il PC copia il jar con nome casuale e avvia il servizio con `shell,v2,raw:`
   e `exec` (il servizio prende il posto di `sh`, così il SIGHUP di adbd arriva
   proprio a lui).
2. Il PC scrive il segreto sull'ingresso del processo.
3. Il servizio: legge il segreto → avvia il custode → cancella il proprio jar
   (il dex è già aperto) → autotest (prepara anche il contesto) → apre il
   `LocalServerSocket` astratto → stampa il pronto → accetta canali.
4. Il PC apre `comandi`, riceve il `CIAO`, poi battito ogni secondo nei due
   sensi (qualsiasi messaggio vale come segno di vita).
5. Fine. Codici d'uscita (il PC li legge dal canale `shell,v2`):

| Codice | Quando |
|---|---|
| 0 | `FINE` chiesto dal PC (chiusura ordinata) |
| 1 | errore d'avvio o eccezione non gestita |
| 3 | nessun messaggio dal PC per 5 s (PC sparito: adbd sul Wi-Fi non se ne accorge, `studio/sistema.md` §1.5) |
| 4 | canale comandi chiuso dal PC, o scrittura impossibile |
| 5 | segreto non arrivato o nessun canale comandi entro 10 s |
| 128+n | ucciso dal segnale n (129 = SIGHUP: canale d'avvio chiuso) |

Il servizio non rimette a posto niente da sé: esce (`System.exit`, e dopo 2 s
`Runtime.halt` se qualcosa si blocca) e **il ripristino lo fa sempre il
custode**. Così la strada è una sola, per la fine ordinata e per quella
improvvisa, e la prova normale verifica anche il custode. Un pezzo che rimette
a posto da sé una cosa (per esempio chiude uno schermo) toglie l'azione dal
custode (`Custode.togli`).

Lato PC, `Componente` considera il telefono perso dopo 5 s senza messaggi:
chiude il compito del canale comandi (`ricevi` restituisce `None`). Se il
`Componente` viene lasciato cadere senza `chiudi`, i compiti si fermano e il
servizio esce da solo dopo 5 s.

## 5. Custode

Processo separato, avviato dal servizio: `setsid sh -c '<script>'
phonestra-custode <jar>` (lo script è `Custode.SCRIPT`, su una riga, col nome
in testa per `ps`).

- **Perché `sh` e non un secondo `app_process`**: parte in millisecondi e pesa
  pochi MB invece dei 30–50 di ART; non ha bisogno del jar (che il servizio può
  cancellare subito); le azioni di ripristino sono comunque comandi di shell
  (`settings put`, `cmd media_session`, `am stack remove`, `rm`) che funzionano
  senza contesto Android. scrcpy usa un secondo `app_process` perché ripristina
  con le API Java; per noi non serve.
- **Sopravvivenza**: `setsid` (sessione nuova) e `trap '' HUP INT TERM PIPE`.
  Uscita e errori a `/dev/null`: se ereditasse l'uscita del servizio terrebbe
  aperto il canale d'avvio anche dopo la sua morte. Se `setsid` mancasse, parte
  senza (resta il `trap`) e l'autotest lo dice (`autotest.custode=ok senza-setsid`).
- **Come si accorge della morte**: legge dal pipe che solo il servizio tiene
  aperto; quando il servizio muore (anche con `kill -9`) il kernel chiude il
  pipe e lo script legge la fine del file.
- **Elenco delle azioni** (meccanismo generico): il servizio chiama
  `imposta(nome, ordine, comando)` e `togli(nome)`, e a ogni cambiamento manda
  l'elenco intero tra una riga `#inizio` e una `#fine`; il custode lo adotta
  solo quando arriva `#fine` (un elenco a metà non sostituisce il precedente).
  Una riga = un comando di shell, eseguito con `sh -c`, uno dopo l'altro: un
  comando che fallisce non ferma i successivi. Ordine: `ordine` crescente, a
  parità l'ultima aggiunta per prima. Ordini pensati per i pezzi futuri
  (`studio/sistema.md` §1.5): pausa dei media 100, volume 200, tempo di
  spegnimento 300, pannello 400, task degli schermi virtuali 500, blocco 600;
  la prova usa 900.
- **Pulizia dei file**: cancella il jar del servizio (se ha il nome atteso) e,
  in `/data/local/tmp`, le copie dimenticate `phonestra-servizio-*.jar` e
  `phonestra-aiuto.jar.*` più vecchie di un minuto (prove §43: un aiutante
  interrotto lascia la sua). Il minuto protegge un processo che sta partendo;
  a uno già partito il file non serve più. Il jar di scrcpy
  (`phonestra-server-*.jar`) non si tocca: lo gestisce scrcpy.
- Poi termina.

## 6. Adattatori e autotest

`Nascoste` raccoglie le chiamate per riflessione (già usate dalle prove del
video, spostate da `Sistema`): servizi da `ServiceManager` +
`Stub.asInterface` o dal metodo statico pubblico, `metodo(classe, nome,
parametri…)` con le varianti di firma in ordine di preferenza, `firme()` per
dire quali varianti ci sono, `inputManager()` con ripiego
`InputManagerGlobal` → `InputManager`. Regola: si guarda cosa c'è, non la
versione.

`Contesto` prepara il contesto una volta sola, un passo alla volta, e annota i
passi non riusciti invece di fermarsi (solo il `ConfigurationController` è
facoltativo: sui Samsung senza di lui `DisplayManagerGlobal` va in errore,
problema noto). Codice riscritto, scrcpy solo come documentazione.

`Autotest` (nessun effetto collaterale: si cercano classi e metodi, non si
chiamano):

| Voce | Cosa controlla |
|---|---|
| `contesto` | contesto della shell preparato, pacchetto `com.android.shell` |
| `permessi` | i permessi della shell che servono ai pezzi (`checkPermission`) |
| `display_manager` | `IDisplayManager.createVirtualDisplay` e `DisplayManagerGlobal` |
| `capture_display` | `IWindowManager.captureDisplay(int, argomenti, ascoltatore)` e la classe degli argomenti |
| `task_stack_listener` | classe `TaskStackListener`, `register/unregisterTaskStackListener` |
| `inject_input_event` | `injectInputEvent` (2 o 3 parametri) e `InputEvent.setDisplayId` |
| `audio_policy` | classi di `AudioPolicy`/`AudioMix`/`AudioMixingRule`, `createAudioRecordSink`, registrazione d'istanza o statica |
| `appunti` | `IClipboard`: `getPrimaryClip`, `getPrimaryClipDescription`, `addPrimaryClipChangedListener`; presenza di `semclipboard` |
| `custode` | vivo, con o senza `setsid` |

## 7. Sicurezza (studio/sistema.md §5)

- Socket astratto dal nome casuale a 128 bit; ogni canale deve venire da
  uid 2000 (`getPeerCredentials`) e cominciare col segreto (confronto a tempo
  costante). Nessuna porta TCP.
- Il segreto passa sull'ingresso del processo, non sulla riga di comando
  (`/proc/<pid>/cmdline` è leggibile da altri processi dello stesso uid).
- Nessun comando generico: l'unica azione di prova (`PROVA_CUSTODE`) crea e
  toglie un file fisso. Le azioni del custode le decide il servizio, non il PC.
- Niente resta: jar cancellato all'avvio e dal custode, servizio che esce dopo
  5 s senza PC, custode che termina dopo il ripristino.

## 8. Lato PC

```rust
let mut c = Componente::avvia(&adb).await?;        // copia, avvio, pronto, comandi, CIAO, battito
c.ciao.autotest(); c.ciao.mancanti();                // esiti dell'autotest
let canale = c.apri_canale("audio").await?;          // altri canali, già col preambolo
let id = c.manda(tipo, dati)?; c.richiesta(tipo, dati).await?; c.ricevi().await;
c.chiudi().await?;                                   // FINE, attesa dell'uscita, chiusura dei canali
```

`shell,v2` nel nostro client (`src/adb/shell.rs`): pacchetti
`id u8 · lunghezza u32 LE · dati` (0 ingresso, 1 uscita, 2 errori, 3 codice
d'uscita, 4 chiusura dell'ingresso). Costruito sopra `Canale`, senza toccare
`src/adb/mod.rs` (dove lavora in parallelo il *delayed ack*).

**Scelta dell'avvio: `shell,v2,raw:` e non `exec:`** (`studio/sistema.md`
§1.2): niente terminale in mezzo, errori separati dall'uscita (la riga di
pronto non si mescola coi log), codice d'uscita (il PC sa perché il servizio è
finito), ingresso usabile per il segreto, e il canale fa da «cavo di vita»
(chiuso → SIGHUP al servizio). `exec:` resta per i comandi brevi.

## 9. Prove da fare sul telefono (in ordine)

Telefono sbloccato, Phonestra chiuso (non serve, ma evita confusione nei `ps`).

1. `phonestra-prova banner` — `features=` deve contenere `shell_v2`.
2. `phonestra-prova servizio 10` — attesi: CIAO con `protocollo=1`, autotest
   tutto `ok` (annotare le voci `manca`), due processi (`phonestra-servizio`,
   il custode) con la RSS, battito ~10 ricevuti e ~10 mandati con silenzio
   massimo ~1 s, «chiusura: servizio uscito con codice 0», poi quattro `ok`
   e «prova riuscita».
3. `phonestra-prova servizio --sparisci` — attesi: «uscito con codice 3 (nessun
   messaggio dal PC per 5 s) dopo ~5 s», quattro `ok` dal secondo collegamento.
4. Custode con `kill -9`: `phonestra-prova servizio 60` e, mentre gira, da un
   altro terminale `phonestra-prova shell 'kill -9 <pid>'` col pid stampato
   nella prima riga («servizio avviato … (pid N)»).
   Attesi: «uscito con codice 137 (ucciso dal segnale 9)», il test finisce
   con «NO servizio uscito col codice atteso» ma gli altri tre controlli `ok`
   (processi spariti, file di prova tolto, nessun jar).
5. Stessa cosa con `kill -HUP <pid>` (atteso codice 129 e pulizia).
6. `phonestra-prova shell 'ls /data/local/tmp; ps -A -o PID,ARGS | grep [p]honestra'`
   — niente del componente rimasto.

## 10. Verificato e ipotesi

✅ Sul PC:
- compilazione del jar (`costruisci.sh`), `cargo build`, `cargo test`
  (formato dei messaggi a pezzi, preambolo, riga di pronto, CIAO, battito,
  pacchetti `shell,v2`, lettura dei residui), `cargo clippy --all-targets`;
- lo script del custode, eseguito da `Custode.java` su Linux (`dash` e `bash`,
  `setsid` di util-linux): dopo `Runtime.halt` del processo Java (come
  `kill -9`) esegue le azioni nell'ordine giusto, un comando che fallisce non
  ferma gli altri, un elenco a metà non sostituisce il precedente, poi termina;
- `Protocollo.java` produce gli stessi byte che `src/componente.rs` si aspetta
  (stessa intestazione del test Rust), preambolo e segreto controllati.

🔶 Da verificare sul telefono:
- `shell,v2,raw:` accettato dall'adbd del S23+ e codice d'uscita riportato
  come 128+segnale per i processi uccisi;
- `exec` dopo `export` in mksh e `--nice-name` visibile in `ps` come
  `phonestra-servizio`;
- `setsid` di toybox avviato da `ProcessBuilder` senza fork (il figlio non è
  capo di gruppo), e il custode che sopravvive al SIGHUP mandato da adbd e al
  riavvio di adbd al blocco Samsung (`block_usb_lock`);
- mksh esegue lo script come dash/bash; `find -mmin +1 -delete` in toybox;
- il pipe del custode non viene ereditato da altri figli del servizio (Java
  chiude i descrittori nei processi figli: altrimenti un figlio ancora vivo
  ritarderebbe il ripristino);
- `LocalSocket.getPeerCredentials` dà uid 2000 per i canali aperti da adbd;
- cancellare il jar subito dopo l'avvio non disturba ART (scrcpy fa lo
  stesso; il nostro `classes.dex` è compresso nel jar, quindi ART lo estrae in
  memoria all'apertura);
- tempi di avvio (`avvio_ms` e il totale stampato dal PC) e RSS del servizio;
- esiti dell'autotest su One UI 8.5 (firme di `IClipboard`, `captureDisplay`,
  `injectInputEvent`).

## Video (fase 2, 28 set 2026)

Il pezzo che sostituirà scrcpy per le finestre delle app e per lo schermo del
telefono nel drawer. **Stesse funzioni di oggi** (`decisioni-utente.md`):
schermo virtuale per app con la densità chiesta dal PC, specchio dello schermo
principale (lato massimo 1920, come `max_size=1920`), ridimensionamento
(`flex_display`), fotogramma chiave per la registrazione (`RESET_VIDEO`),
avvio dell'app e della pagina «Informazioni app», orientamento bloccato sullo
schermo virtuale, pannello fisico acceso/spento, app via dalle recenti alla
chiusura della finestra; in più, al posto dei `dumpsys` del PC, eventi per
l'orientamento chiesto e per le schermate protette. **Non ancora collegato a
Phonestra** (finestre e drawer usano ancora scrcpy). Codice nostro: scrcpy e
`VideoProva` solo come documentazione e misure (prove §43).

### File

| Dove | File | Cosa fa |
|---|---|---|
| telefono | `Video.java` | messaggi 0x40–0x46, registro delle sessioni, canale `video:<id>`, eventi |
| telefono | `SessioneVideo.java` | schermo virtuale o specchio, codificatore attuale, scrittura dei pacchetti, ridimensionamento, rotazione dello specchio, chiusura |
| telefono | `Codifica.java` | MediaCodec hardware da Surface, thread di lettura, fotogramma chiave a comando |
| telefono | `EventiApp.java` | `TaskStackListener`: orientamento, app spostata, task rimosso; controllo della schermata protetta |
| telefono | `Protetta.java` | `captureDisplay` rimpicciolito + `containsSecureLayers` |
| telefono | `Pannello.java` | `SurfaceControl.setDisplayPowerMode` sugli schermi fisici, ripristino col custode |
| PC | `src/video_nostro/mod.rs` | `Video` (smistamento), `SessioneNostra`, `ComandiVideo`, `Evento` |
| PC | `src/video_nostro/prova.rs` | `phonestra-prova video-componente app\|schermo` |

Nei file comuni: in `Servizio.java` la registrazione del canale `video`, sei
`case` e `Servizio.custode()`; in `componente.rs` i tipi `VIDEO_*`; in
`prova.rs` il comando. Altrove: `Sistema.avviaIntent` (un `Intent` qualsiasi,
per «Informazioni app»), `sessione::intestazione` (intestazione dei pacchetti
come funzione pura, per i test).

### Messaggi (canale comandi, contenuto `chiave=valore` a righe)

| Tipo | Nome | Domanda (PC → servizio) | Risposta |
|---|---|---|---|
| 0x40 | `VIDEO_APRI` | `larghezza altezza dpi codec` oppure `specchio=1 lato_massimo codec`; facoltativi `app=` / `informazioni=` | `id display codec larghezza altezza` (misura già allineata), `avvio=` |
| 0x41 | `VIDEO_CHIUDI` | `id [togli_task=1]` | vuota, a chiusura fatta |
| 0x42 | `VIDEO_AVVIA_APP` | `id app=<pacchetto>` o `id informazioni=<pacchetto>` | esito dell'avvio (testo) |
| 0x43 | `VIDEO_RIDIMENSIONA` | `id larghezza altezza` | `misura LxA` o `misura invariata …` |
| 0x44 | `VIDEO_CHIAVE` | `id` | vuota |
| 0x45 | `VIDEO_PANNELLO` | `acceso=0\|1` | `schermi=<quanti>` |
| 0x46 | `VIDEO_EVENTO` | — (servizio → PC, spontaneo) | — |

Fascia 0x40–0x4f scelta per non incrociare audio e input, sviluppati in
parallelo. Le domande si eseguono in ordine su un thread `video` del
servizio: il canale comandi (battito, input) non aspetta mai il video.
Errori: risposta `ERRORE` col testo. Lato PC `avvia_app`, `ridimensiona`,
`ricomincia_video`, `pannello` non aspettano la risposta (come i comandi di
scrcpy: un errore finisce nel log); `APRI` e `CHIUDI` sì.

**Eventi** (`VIDEO_EVENTO`, `evento=<nome>` e `id=<sessione>`):

| Evento | Coppie | Quando |
|---|---|---|
| `orientamento` | `display verticale=0\|1 valore=N` | al primo controllo con un'app in vista, poi quando cambia «solo verticale» |
| `protetta` | `display protetta=0\|1` | al primo controllo, poi quando cambia |
| `spostata` | `task display` | un task dello schermo passa su un altro schermo (app aperta anche sul telefono) |
| `rimosso` | `task` | un task dello schermo si chiude |
| `fine` | `motivo` | il telefono chiude la sessione da sé (codificatore fermo, canale non aperto entro 10 s, specchio non rifatto) |

**Canale `video:<id>`** (aperto dal PC subito dopo `VIDEO_APRI`, entro 10 s):
il servizio ci scrive i pacchetti **nel formato che il PC legge già** con
`sessione::leggi_pacchetto` (intestazione di 12 byte: misura =
`0x80000000 · larghezza u32 · altezza u32`; dati = `pts u64` in µs dal primo
fotogramma, bit 62 parametri, bit 61 chiave, `· lunghezza u32`, poi Annex B).
Il PC non ci scrive niente; se lo chiude la sessione si chiude (senza toccare
le recenti: come oggi quando cade il collegamento). Il codificatore parte
quando il canale è aperto: il primo pacchetto è la misura, poi i parametri e
il primo fotogramma chiave. Il codec non viaggia sul canale: è nella risposta.

### Scelte

- **Schermo virtuale**: API pubblica `createVirtualDisplay(nome, l, a, dpi,
  null, flag)` coi flag misurati in prove §43 (`FLAG_PROPOSTI`, senza
  `ALWAYS_UNLOCKED` né decorazioni), misura allineata a 8 e all'allineamento del
  codificatore **prima** di crearlo. Subito dopo, come oggi dal PC, `cmd window
  set-ignore-orientation-request -d <id> true; cmd window user-rotation -d <id>
  lock 0` (su un thread a parte).
- **Specchio**: `DisplayManager.createVirtualDisplay(nome, l, a, 0, surface)`
  (statica nascosta, `CAPTURE_VIDEO_OUTPUT`), misura dello schermo principale
  (`DisplayManagerGlobal.getDisplayInfo(0)`) ridotta a 1920 di lato. Ogni
  500 ms si rilegge la misura: se il telefono ruota, nuovo codificatore e nuovo
  specchio, poi si chiudono i vecchi e il PC riceve la misura nuova.
- **Codificatore**: il primo hardware non alias per il tipo; formato di prove
  §43 (8 Mbit/s, 60 fps dichiarati, chiave ogni 10 s, ripetizione dopo 100 ms,
  priorità 0, gamma limitata) più `prepend-sps-pps-to-idr-frames=1`; se
  `configure` lo rifiuta, senza (e allora dopo una richiesta il servizio
  rimanda i parametri salvati davanti al fotogramma chiave); ultimo ripiego il
  codificatore predefinito di Android. Il thread di lettura copia ogni uscita e
  la scrive intera in una sola `write` (intestazione e dati).
- **Fotogramma chiave** (`ricomincia_video`): `REQUEST_SYNC_FRAME`, senza
  ricreare niente (oggi scrcpy ricrea il codificatore: ripartenze di 1–2 s).
  Misurato in §43: ~40 ms.
- **Ridimensionamento**: misura allineata uguale → niente; diversa → nuovo
  codificatore preparato prima, poi misura al PC, `VirtualDisplay.resize(l, a,
  dpi)` + `setSurface`, poi chiusura del vecchio. I pacchetti del vecchio,
  ancora in volo, si scartano (si scrive solo quello «attuale»). Densità fissa
  (come oggi). Con `ridimensionabile=false` o per lo specchio il PC non lo manda.
- **Sospensione delle finestre nascoste**: non fatta, oggi non c'è
  (`PARAMETER_KEY_SUSPEND` resta per dopo).
- **Orientamento**: `onActivityRequestedOrientationChanged` (e
  `onTaskRequestedOrientationChanged` dove esiste) ricorda il valore chiesto
  per il task, finché in cima c'è la stessa attività; altrimenti vale quello
  del manifest (`topActivityInfo.screenOrientation`). Conta il primo task
  visibile dello schermo (`getAllRootTaskInfosOnDisplay`). Solo verticale =
  PORTRAIT, SENSOR_PORTRAIT, REVERSE_PORTRAIT, USER_PORTRAIT (come il
  `contains("PORTRAIT")` del PC). Differenza da oggi: i task trasparenti non
  sono distinti (`TaskInfo` non lo dice in Android 14).
- **Schermata protetta**: `captureDisplay` con `setFrameScale(0.05)` (serve
  solo il sì/no) → `containsSecureLayers`; dopo ogni gruppo di eventi (150 ms)
  e ogni 3 s come il giro del PC di oggi, perché una finestra protetta può
  comparire senza eventi dei task.
- **Pannello**: token degli schermi fisici da `SurfaceControl` o, da Android
  14, da `DisplayControl` in `services.jar` (class loader sul
  `SYSTEMSERVERCLASSPATH` e libreria `android_servers`), poi
  `setDisplayPowerMode(token, 0|2)`. È di tutto il telefono (un solo
  servizio): `Video::pannello`; `ComandiVideo::pannello` c'è per somiglianza
  con oggi. `cmd display power-off/power-on` (Android 15) scartato: passa da
  `requestDisplayPower`, che scrcpy ha visto bloccare l'input (studio/video.md §4.5).
- **Custode**: pannello spento → azione `pannello` (ordine 400), tolta alla
  riaccensione: `dumpsys power | grep -q mWakefulness=Awake && { input keyevent
  KEYCODE_SLEEP; sleep 1; input keyevent KEYCODE_WAKEUP; }` (dalla shell non c'è
  un modo di chiamare `setDisplayPowerMode`: si fa ripartire lo schermo; il
  telefono resta bloccato, ma col pannello acceso invece che nero col touch
  attivo). **Schermi**: nessuna azione, li chiude Android quando il processo
  muore (prove §43). **Task**: nessuna azione, di proposito: oggi quando il
  collegamento cade (telefono bloccato, Wi-Fi) scrcpy muore, le app passano sul
  telefono e alla riconnessione tornano nella finestra col loro stato;
  toglierle dal custode lo perderebbe a ogni caduta. Le app si tolgono dalle
  recenti solo quando l'utente chiude la finestra (`VIDEO_CHIUDI togli_task=1`),
  come oggi.

### Lato PC

```rust
let c = Componente::avvia(&adb).await?;
let (video, altri) = Video::avvia(c);        // smista risposte ed eventi; `altri`: messaggi degli altri pezzi
let SessioneNostra { codec, display, video: flusso, mut comandi, mut eventi, .. } =
    SessioneNostra::avvia(&video, &Opzioni { .. }).await?;   // stesse Opzioni di sessione.rs
comandi.avvia_app("com.android.chrome").await?;             // o informazioni_app
leggi_pacchetto(&mut flusso).await?;                        // come oggi
comandi.ridimensiona(l, a).await?; comandi.ricomincia_video().await?;
video.pannello(false)?;
while let Some(e) = eventi.recv().await { /* Evento::Orientamento, Protetta, Spostata… */ }
comandi.chiudi(true).await?;                                // true = via dalle recenti
video.chiudi().await?;                                      // FINE del servizio
```

`Video` possiede il `Componente` in un compito (`smista`): è il primo pezzo a
dover condividere il canale comandi fra più finestre. Quando arriveranno audio
e input, lo smistamento (risposte per id, eventi per sessione, il resto agli
altri) andrà spostato in `componente.rs` per tutti.

**Cosa resta per collegarlo a Phonestra**: un `Video` per collegamento in
`Collegamento` (al posto dei `Sessione::avvia` per finestra); in
`finestra::sessione` `SessioneNostra` al posto di `Sessione`, il display da
`SessioneNostra::display` (via `display_da_messaggio` e il compito che legge
il server), via i comandi `cmd window …` e `am start … APPLICATION_DETAILS`
(li fa il telefono), `togli_dalle_recenti` → `comandi.chiudi(true)`,
`chiedi_orientamento` e il suo compito → `Evento::Orientamento`,
`Collegamento::protetti` → `Evento::Protetta` (e via `COMANDO_FINESTRE` dal
giro dei 3 s); pannello per collegamento invece che per sessione (il
«rispegni» dopo la fine di un'altra sessione non serve più). Tocchi e tasti
dal pezzo input.

### Prove sul telefono (telefono sbloccato, Phonestra chiuso)

1. `phonestra-prova video-componente app` — Orologio su uno schermo
   1120×1992 H.264 per 15 s. Attesi: primo fotogramma con misura e parametri,
   5 fotogrammi chiave (ritardo medio ~40–80 ms rete compresa),
   ridimensionamento a 800×1400 con misura nuova e fotogrammi, stessa misura
   senza codificatore nuovo, pannello nero per 3 s e riacceso, eventi
   `Protetta { protetta: false }` e `Orientamento`, chiusura, nessun task né
   schermo rimasti, servizio uscito con codice 0, nessun processo né jar,
   ffprobe che legge il file; «prova riuscita».
2. `phonestra-prova video-componente app --codec h265 --app com.android.chrome --secondi 30`
   — fotogrammi/s negli ultimi secondi.
3. `phonestra-prova video-componente app --app com.x8bit.bitwarden` (o
   un'altra app con schermata protetta) — atteso `protetta: true`.
4. `phonestra-prova video-componente app --app com.facebook.katana` — atteso
   `Orientamento { verticale: true, … }`.
5. `phonestra-prova video-componente schermo --secondi 20` — specchio dello
   schermo principale; ruotando il telefono durante la prova è attesa una
   misura nuova (lati scambiati) nel riepilogo `misure`.
6. Caduta col pannello spento: `phonestra-prova video-componente app --secondi 60`
   e, nei 3 s di pannello nero, `phonestra-prova shell 'kill -9 <pid>'` (pid
   nella prima riga): il custode deve riaccendere lo schermo (telefono
   bloccato, schermata di blocco visibile). Poi
   `phonestra-prova shell 'ps -A | grep [p]honestra; ls /data/local/tmp'`.
7. Aprire sul telefono la stessa app mentre la prova 2 gira: atteso l'evento
   `Spostata`.

### Verificato e ipotesi

✅ Sul PC: compilazione del jar; `cargo build`, `cargo test` (intestazione dei
pacchetti letta dal PC con gli stessi byte prodotti da `SessioneVideo.java`,
richiesta e risposta di apertura, eventi, valori non validi), `cargo clippy`.

🔶 Da verificare sul telefono: tutto il resto, in particolare
`VirtualDisplay.resize` + `setSurface` a codifica in corso; lo specchio con la
statica nascosta e il suo rifacimento alla rotazione; `DisplayControl` caricato
dal servizio e `setDisplayPowerMode`; il ripristino del pannello dal custode;
`setFrameScale` accettato da `captureDisplay` e `containsSecureLayers` ancora
giusto a immagine rimpicciolita, e il suo costo ogni 3 s; l'orientamento da
`topActivityInfo.screenOrientation` per le app che lo dichiarano nel manifest;
`onTaskDisplayChanged` quando un'app viene aperta anche sul telefono; il
conteggio `"phonestra-` in `dumpsys display` come prova che lo schermo non c'è
più.
