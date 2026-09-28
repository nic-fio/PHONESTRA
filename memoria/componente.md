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

## 11. Input (modulo 3, 28 set 2026)

Tocchi, rotellina, tasti, testo, «indietro» e appunti sul canale `comandi`.
Scopo: **le stesse funzioni di oggi con scrcpy** (`sessione::Comandi`,
`appunti.rs`), niente di più (`decisioni-utente.md`). Codice nostro, scrcpy
solo come documentazione (`studio/input.md` §2–§4, §6). **Non ancora
collegato all'interfaccia**: scrcpy resta in uso. **Niente è stato provato sul
telefono.**

| Dove | File | Cosa fa |
|---|---|---|
| telefono | `Input.java` | messaggi 0x40–0x4f, coda del thread «input», iniezione, dita, scalatura, appunti |
| telefono | `Appunti.java` | `IClipboard` diretto: lettura, descrizione (sensibile), scrittura, ascoltatore |
| telefono | `InputProva.java` | comandi delle prove: schermo virtuale, firma dell'immagine, appunti dell'utente salvati |
| PC | `src/input_nostro.rs` | `InputNostro` (stessi metodi di `Comandi`), codifica, `Appunti`, richieste |
| PC | `src/prova_input.rs` | `phonestra-prova input-componente <prova>` |

Nei file comuni: in `Servizio.java` una voce nel `default` dello `switch`
(`Input.nostro(tipo)` → `Input.ricevi`) e `Servizio.custode()`; in
`componente.rs` il `Mittente` (clonabile: ogni finestra avrà il suo
`InputNostro` senza possedere il `Componente`). Stub nuovi per javac:
`MotionEvent`, `KeyEvent`, `KeyCharacterMap`, `InputEvent`, `SystemClock`,
`IOnPrimaryClipChangedListener`.

### Messaggi (fascia 0x40–0x4f, big-endian)

| Tipo | Nome | Contenuto |
|---|---|---|
| 0x40 | `TOCCHI` | `display i32 · larghezza u16 · altezza u16 · n u8 · n × (dito i64 · azione u8 · x i32 · y i32 · pressione f32)` |
| 0x41 | `ROTELLINA` | `display i32 · x i32 · y i32 · larghezza u16 · altezza u16 · orizzontale f32 · verticale f32` |
| 0x42 | `TASTO` | `display i32 · azione u8 · codice u32 · ripetizione u32 · meta u32` |
| 0x43 | `TESTO` | `display i32 · testo UTF-8` |
| 0x44 | `INDIETRO` | `display i32 · azione u8` |
| 0x45 | `APPUNTI_SCRIVI` | `display i32 · incolla u8 · testo UTF-8`; risposta vuota se l'id non è 0 |
| 0x46 | `APPUNTI_LEGGI` | domanda vuota; risposta `stato u8 · testo` |
| 0x47 | `APPUNTI_ASCOLTA` | `attivo u8`; risposta vuota se l'id non è 0 |
| 0x48 | `APPUNTI_CAMBIATI` | servizio → PC, spontaneo: `stato u8 · testo` |
| 0x4c | `CONTEGGI` | domanda di diagnosi; risposta `chiave=valore` (iniettati, falliti, scartati, avvisi_appunti, ultimo_errore) |
| 0x4d | `PROVA` | domanda: comando di prova in testo (`InputProva.java`) |

Stato degli appunti: 0 vuoti o non di testo, 1 testo, 2 sensibili (senza
testo), 3 sconosciuti (senza testo: nel dubbio non passano, come oggi).
Gli eventi (id 0) non hanno risposta: il PC non aspetta il telefono.
`larghezza`/`altezza` = misura dell'immagine su cui il PC ha calcolato le
coordinate (oggi la misura del video).

### Come funziona (uguale a oggi salvo dove detto)

- **Coda**: il thread che legge i comandi non inietta; mette i messaggi in coda
  al thread «input» (uno solo: l'ordine resta). I comandi `PROVA` hanno un
  thread ciascuno.
- **Iniezione**: `InputManagerGlobal.injectInputEvent` (2 o 3 parametri) in
  modo **asincrono**, con `InputEvent.setDisplayId` sempre, per riflessione.
  Un rifiuto o un'eccezione si conta (`falliti`); nel log il primo errore e poi
  uno ogni 100.
- **Dita**: come scrcpy, ogni identificativo del PC (−1 mouse, −2 dito
  generico, 10/11 pizzico) diventa un dito con numero locale 0–9 (il più
  piccolo libero), al massimo 10. `SOURCE_TOUCHSCREEN`, `TOOL_TYPE_FINGER`,
  pulsanti 0 anche per il «mouse» (oggi il mouse manda solo il pulsante
  principale, che scrcpy tratta da dito). Ogni evento porta tutte le dita
  appoggiate; `POINTER_DOWN/UP | indice << 8` dalla seconda; `downTime` del
  primo «giù». Un messaggio con più dita = un evento per dito, nell'ordine
  (come `dita_insieme` oggi). Hover e pulsanti del mouse non ci sono: Phonestra
  non li usa.
- **Scalatura e misure vecchie**: coordinate × (misura logica dello schermo /
  misura del PC), misura da `DisplayManagerGlobal.getDisplayInfo` a ogni
  evento. scrcpy scarta ogni evento calcolato su una misura diversa da quella
  del video; qui, finché il modulo video non dichiara la misura
  (`Input.dimensioneVideo(display, l, a)`, poi vale il confronto esatto come
  scrcpy), si scarta se le **proporzioni** differiscono oltre il 2 % (gli
  arrotondamenti del video a multipli di 8 restano sotto). Il modulo video
  chiamerà anche `Input.dimentica(display)` alla chiusura di uno schermo.
- **Rotellina**: `ACTION_SCROLL`, `SOURCE_MOUSE`, `AXIS_VSCROLL`/`HSCROLL` con
  i valori `f32` così come sono, limitati a ±16 dal PC (oggi la virgola fissa
  di scrcpy fa lo stesso).
- **Tasti**: `KeyEvent(ora, ora, azione, codice, ripetizione, meta,
  VIRTUAL_KEYBOARD, 0, 0, SOURCE_KEYBOARD)`; la ripetizione resta 0 come oggi.
- **Testo**: `KeyCharacterMap.load(VIRTUAL_KEYBOARD).getEvents` un carattere
  alla volta; un carattere senza tasto si salta e si conta tra gli
  `scartati` (il PC manda con `testo` solo l'ASCII, il resto con `incolla`).
  Nessuna scomposizione in tasto morto (`KeyComposition`): per l'ASCII non
  serve.
- **Indietro**: `KEYCODE_BACK` giù/su; solo per lo schermo principale non
  interattivo `POWER` al rilascio, come `BACK_OR_SCREEN_ON` di scrcpy.
- **Incolla**: appunti, poi `KEYCODE_PASTE` giù/su allo schermo. **Differenza
  voluta**: scrcpy legge sempre il testo degli appunti per non riscrivere lo
  stesso; qui si legge solo se il clip attuale è nostro (etichetta
  «Phonestra», dalla descrizione): leggere il clip di un'altra app può far
  comparire l'avviso «… ha incollato dagli appunti».
- **Appunti via `IClipboard`** (non `ClipboardManager`): niente Looper
  principale da far girare (l'ascoltatore è un Binder, chiamato su un thread
  del Binder) e niente `semclipboard` Samsung (#6224). Firme scelte per forma
  (la più lunga con testi e interi dopo i parametri fissi), pacchetto
  `com.android.shell`, utente 0, dispositivo 0.
- **Copie sul telefono → PC** (sostituisce la sessione scrcpy
  `clipboard_autosync` più l'aiutante `appunti-sensibili`): con l'ascolto
  acceso (`APPUNTI_ASCOLTA 1`), a ogni cambiamento il servizio legge prima la
  descrizione (nessun avviso) e manda `APPUNTI_CAMBIATI`: stato 2 senza testo
  se sensibile, altrimenti il testo (clip non di testo: niente, come scrcpy).
  Le scritture del servizio non tornano indietro: bandiera durante
  `setPrimaryClip` e, poiché l'avviso può arrivare dopo, anche un testo uguale
  all'ultimo messo da noi entro 3 s. Il limite di 200 000 byte e il controllo
  dei rimbalzi restano sul PC (`appunti.rs`, `Collegamento::e_un_rimbalzo`).

### Prove sul telefono (`phonestra-prova input-componente …`)

Telefono sbloccato, Phonestra chiuso. Ogni prova avvia il servizio, fa i suoi
controlli («ok»/«NO»), poi **ripulisce sempre** (appunti dell'utente rimessi,
schermi chiusi con i loro task, ascolto spento, `FINE`) e controlla: servizio
uscito con 0, nessun processo o file del componente, nessuno schermo
`phonestra-prova-input` in `dumpsys display`. Il testo degli appunti
dell'utente non viene mai stampato né mandato al PC (resta nella memoria del
servizio). Se il servizio muore a metà, il custode toglie i task delle app
avviate e lo schermo sparisce col processo; gli appunti dell'utente invece
restano quelli della prova.

1. `appunti` — scrittura e rilettura con accentate; nessun avviso per le
   scritture di Phonestra (anche ripetute); copia «di un'altra app» → avviso
   col testo; copia sensibile → avviso senza testo e lettura senza testo;
   ascolto spento → nessun avviso.
2. `tocchi` — schermo virtuale 720×1280/320 (flag delle prove video, con
   `TRUSTED|OWN_FOCUS`) con le Impostazioni: rotellina −5 scatti (la firma
   cambia), frazioni e ritorno in cima, trascinamento col dito generico con
   coordinate su metà misura (scalatura), pizzico a due dita in un solo
   messaggio, tocco al 60 % dell'altezza che apre una voce (attività diversa
   in `dumpsys activity activities` o firma cambiata), «indietro» che torna,
   tocco con misura vecchia scartato (2 eventi), nessun evento fallito.
3. `testo` — ricerca di Google (`android.search.action.GLOBAL_SEARCH`, pacchetto
   `com.google.android.googlequicksearchbox`) su uno schermo virtuale:
   «Phonestra prova 123» un carattere alla volta, Ctrl+A, Ctrl+C → gli appunti
   riletti coincidono e l'avviso della copia arriva al PC; Ctrl+A, incolla
   «àèìòù €», « fine», Ctrl+A, Ctrl+C → «àèìòù € fine». Se la ricerca di Google
   non c'è o non dà il fuoco al campo: `--app PACCHETTO`, `--azione
   AZIONE[:PACCHETTO]`, `--tocca X,Y` (pixel dello schermo di prova).
4. `tutte` — le tre di seguito.

Opzioni comuni: `--misura LxA`, `--dpi D`. Con `PHONESTRA_DEBUG=1` si vedono
anche i messaggi del servizio (`[servizio] …`).

### Verificato e ipotesi

✅ Sul PC: jar compilato; `cargo test` (codifica di tutti i messaggi, stati
degli appunti, pressione come oggi, messaggi mandati dai metodi di
`InputNostro`, lettura di `dumpsys activity activities`, differenza tra
firme); un banco Java temporaneo sulle parti pure di `Input.java` (dita:
`POINTER_DOWN/UP` e indici, numero locale libero, undicesimo dito scartato;
scalatura: stessa misura, 2×, proporzioni diverse, arrotondamento del video,
misura dichiarata dal video).

🔶 Da verificare sul telefono:
- `IClipboard` chiamato direttamente su One UI 8.5: firme scelte, scrittura
  senza `SecurityException`, ascoltatore Binder chiamato senza Looper;
  sottoclasse di `IOnPrimaryClipChangedListener.Stub` accettata da ART;
- `ClipDescription.getLabel` per riconoscere i clip nostri; ripristino del
  clip dell'utente (un clip con URI di un'altra app potrebbe essere rifiutato:
  la prova lo stampa);
- `DisplayManagerGlobal.getDisplayInfo(id).logicalWidth/logicalHeight` per gli
  schermi virtuali;
- che la ricerca di Google abbia il campo col fuoco e accetti Ctrl+A/Ctrl+C
  da tasti iniettati; che il tocco al 60 % delle Impostazioni Samsung apra una
  voce (se no, il controllo guarda anche la firma);
- tempi: nessuna misura della latenza fatta.

### Cosa resta per collegarlo a Phonestra

- In `finestra.rs`: al posto di `sessione::Comandi` un `InputNostro` con
  `componente.mittente()` e l'id dello schermo della finestra (dal modulo
  video). I metodi hanno lo stesso nome e significato. Restano fuori
  dall'input (modulo video o comandi): `avvia_app`, `ridimensiona`,
  `pannello`, `ricomincia_video`, `chiudi`.
- Il modulo video deve chiamare `Input.dimensioneVideo` quando cambia la
  misura del flusso e `Input.dimentica` quando chiude uno schermo.
- In `appunti.rs`: al posto della sessione scrcpy `clipboard_autosync` e di
  `app::appunti_sensibili`, `ascolta_appunti(true)` all'apertura del
  collegamento e gli `APPUNTI_CAMBIATI` letti da `Componente::ricevi`
  (`Appunti::da_avviso`): `Testo` → stessi controlli di oggi (lunghezza,
  rimbalzo), `Sensibili`/`Sconosciuti` → non passa.
