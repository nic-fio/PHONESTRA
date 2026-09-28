# Studio: sistema e trasporto del componente Phonestra sul telefono

> 28 settembre 2026. Riguarda Android 14–16 (API 34–36): Galaxy S23+ (Android 16, One UI 8.5) e Galaxy S26.
> **Fonti lette per questo studio**: i sorgenti di `packages/modules/adb`
> (ultimo commit di `main` su android.googlesource.com, 25 mar 2025, cioè
> l'adbd di Android 16); `frameworks/base` e `art` di Android 16 (copia
> LineageOS `lineage-23.0` su GitHub, perché le pagine web di googlesource
> rispondevano 503); scrcpy `master` (commit `19c1261`, 5 set 2026); il codice di
> Phonestra (`src/adb/`, `src/sessione.rs`, `src/collegamento.rs`,
> `src/app.rs`, `src/notifiche.rs`, `telefono/aiuto/`).
> **Legenda**: ✅ **fatto**, verificato nel sorgente citato. 🔶 **ipotesi**,
> ragionata ma non provata sul telefono. 🧪 **da misurare**.

## Riepilogo e raccomandazioni

1. **Un solo processo di servizio per collegamento**
   (`app_process … phonestra.Servizio`), di lunga durata, al posto di scrcpy
   per ogni finestra più un aiutante avviato a ogni richiesta. Accanto a lui
   gira un **custode separato** (processo figlio con `setsid`, come il
   `CleanUp` di scrcpy) che, quando il servizio muore per qualsiasi causa,
   rimette a posto il telefono: pausa dei media, volume, tempo di
   spegnimento, pannello, task dei display virtuali.
2. **I canali sono socket `localabstract:`**, cioè flussi ADB separati verso un
   `LocalServerSocket` dal nome casuale. Ce n'è uno per i **comandi e
   servizi** (domanda e risposta più eventi spontanei), uno per l'**audio** e
   uno per il **video di ogni finestra**. L'avvio passa da `shell,v2,raw:`,
   senza PTY: così i log arrivano separati e si legge il codice d'uscita.
   `exec:` resta solo per i comandi brevi.
3. **Nel nostro client ADB va attivato il `delayed_ack`**, dichiarandolo nel
   CNXN. L'adbd del telefono lo annuncia sempre ✅. Oggi ogni canale ha un solo
   `WRTE` in volo, sia dal PC sia dal telefono, e quindi al massimo un blocco
   per andata e ritorno Wi-Fi. È il limite più probabile per video e audio
   quando il Wi-Fi ha latenze variabili 🔶.
4. Conviene **abbassare la dimensione massima dei messaggi** dichiarata
   (per esempio 64 KB invece di 1 MB): tutti i canali condividono una sola
   connessione TCP/TLS, e un blocco video da 1 MB bloccherebbe l'audio e i
   comandi che gli vengono dietro 🔶🧪.
5. **Formato dei messaggi**: intestazione fissa
   `tipo u8 · bandiere u8 · id u16 · lunghezza u32` (big-endian) seguita dal
   contenuto. Per tocchi e tasti il contenuto è binario compatto; per app,
   notifiche e stato è JSON (`org.json` sul telefono, `serde_json` sul PC). Le
   icone viaggiano come PNG binario, non in base64. I pacchetti audio e video
   restano nel formato che Phonestra già legge (12 byte più i dati).
6. **Il servizio controlla chi si collega**: per prima cosa un segreto
   passato sulla riga di comando, poi l'uid di chi apre il socket
   (`LocalSocket.getPeerCredentials()`), che deve essere 2000 (adbd/shell). Il
   PC manda un **battito** ogni secondo; se il servizio non ne riceve per 5 s,
   termina e il custode ripristina tutto. Serve perché adbd non attiva il
   keepalive TCP sul Debug wireless ✅, quindi un PC sparito senza chiudere la
   connessione può non essere notato per minuti.
7. **Le notifiche arrivano subito, senza `dumpsys`** 🔶. La shell ha
   `STATUS_BAR_SERVICE` ✅, che è proprio il permesso controllato da
   `INotificationManager.registerListener`, cioè il meccanismo con cui SystemUI
   si registra come ascoltatore ✅. Se la prova funziona, avremmo anche la
   cancellazione delle notifiche sul telefono e forse aprire l'azione giusta
   e la risposta rapida.
8. **Il resto dello stato arriva a eventi**, invece di un `dumpsys` ogni 3 s:
   - blocco: `KeyguardManager.addKeyguardLockedStateListener`, che richiede
     `SUBSCRIBE_TO_KEYGUARD_LOCKED_STATE`, permesso che la shell ha ✅;
   - media: `MediaSessionManager`, con `MEDIA_CONTENT_CONTROL` ✅;
   - volume: `AudioManager`;
   - app installate o rimosse: `PackageManager.getChangedPackages()` (API
     pubblica, basta interrogarla ogni tanto);
   - appunti: `IClipboard` con un ascoltatore, grazie a
     `READ_CLIPBOARD_IN_BACKGROUND` ✅.
9. **Le API nascoste non sono bloccate per `app_process`**: ART parte con la
   politica `kDisabled`, e solo lo zygote la attiva per le app ✅. Le
   differenze tra versioni e marche si gestiscono come fa scrcpy: un
   *wrapper* per ogni servizio, i metodi cercati per nome e firma con varianti
   di ripiego, e un autotest all'avvio che dice al PC quali funzioni ci sono.
   Se una funzione manca si spegne quella, il servizio non si ferma.
10. **Sicurezza**: la shell può molto (iniettare eventi, catturare l'audio,
    leggere lo schermo, `WRITE_SECURE_SETTINGS`…). Il componente espone solo
    comandi precisi, mai un «esegui un comando qualsiasi», e non resta in vita
    dopo la sessione. Rispetta il segno «sensibile» degli appunti e le
    schermate protette: la shell comunque non ha
    `CAPTURE_SECURE_VIDEO_OUTPUT` ✅. Tratta le notifiche come dati sensibili:
    niente nei log, e la shell riceve anche i codici OTP.

### Architettura proposta, in breve

```
PC (Phonestra, Rust)                              Telefono (uid 2000)
──────────────────────────────                    ─────────────────────────────────────────
Adb (1 TCP+TLS, delayed_ack, max 64 KB)
 ├─ sync:  → copia phonestra.jar (una volta)
 ├─ shell,v2,raw: CLASSPATH=… app_process /data/local/tmp --nice-name=phonestra phonestra.Servizio <nome> <segreto>
 │        (stdout: righe di stato; stderr: log; exit code)      ──►  Servizio (Java, Looper + thread)
 │                                                                  ├─ Custode (app_process figlio, setsid,
 │                                                                  │   legge dal pipe lo stato da ripristinare)
 ├─ localabstract:phonestra_<nome>  «C» comandi/servizi/eventi  ◄─►  ├─ Controllo (thread lettore + scrittore)
 ├─ localabstract:phonestra_<nome>  «A» audio (solo uscita)      ◄──  ├─ Audio (AudioRecord + AAC)
 └─ localabstract:phonestra_<nome>  «V n» video finestra n      ◄──  └─ Video n (VirtualDisplay + MediaCodec)
```

Il primo messaggio di ogni canale lo manda il PC: `segreto (16 byte) · tipo (C/A/V) · id finestra`.
Chi sbaglia il segreto viene chiuso subito.

---

## 1. Trasporto tra componente e PC

### 1.1 Come funziona oggi (codice di Phonestra)

- `src/adb/mod.rs`: un solo collegamento TCP+TLS. Un compito legge tutti i
  messaggi e li smista per id locale. Ogni `Canale::scrivi` manda un `WRTE` e
  **aspetta l'`OKAY`** prima del blocco successivo. In arrivo, a ogni `WRTE` si
  risponde subito `OKAY` e i dati vanno in una coda senza limite. Il CNXN
  annuncia `host::features=shell_v2,cmd,stat_v2` e `MAX_DATI` = 1 MB.
- scrcpy: una sessione per finestra, avviata con `shell:` e poi due canali
  `localabstract:scrcpy_<scid>` (video e comandi).
- aiutante e audio: `exec:CLASSPATH=… app_process …` a ogni richiesta, con
  copia del jar ogni volta (`src/app.rs`).
- custode: `exec:trap '' HUP TERM PIPE; …; cat >/dev/null; …`
  (`src/collegamento.rs`).

### 1.2 I tre modi di parlare con un processo (fatti dal sorgente di adbd)

| Servizio | Cosa crea adbd | Chiusura del canale | Note |
|---|---|---|---|
| `exec:cmd` | processo con **PTY in modo raw** (`cfmakeraw`); stdout e stderr mescolati ✅ | il kernel manda SIGHUP alla sessione quando il lato master del PTY si chiude ✅ | byte intatti; la PTY aggiunge una copia in più e ha buffer piccoli (in Linux `N_TTY_BUF_SIZE` = 4 KB più circa 64 KB di buffer tty 🔶) |
| `shell:cmd` (senza `v2`) | uguale a `exec:` (PTY raw) ✅ | uguale | è come oggi parte scrcpy |
| `shell,v2,raw:cmd` | **socketpair** (niente PTY) e protocollo shell v2: pacchetti `id u8 + lunghezza u32 LE` per stdin, stdout, stderr, codice d'uscita e chiusura dello stdin ✅ | adbd manda **SIGHUP al pid del figlio** e chiude i pipe ✅ | stderr separato e codice d'uscita; serve la funzione `shell_v2`, che già annunciamo |
| `localabstract:nome` | collega direttamente al `LocalServerSocket` del processo ✅ | il socket si chiude (EOF/EPIPE); **nessun segnale** | un processo, tanti flussi; il servizio deve accorgersi da sé che il PC non c'è più |

Fonti: [`daemon/services.cpp`](https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/daemon/services.cpp)
(`ShellService`, `exec:`), [`daemon/shell_service.cpp`](https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/daemon/shell_service.cpp)
(`StartSubprocess`: «If we aren't using the shell protocol we must allocate a
PTY…»; `kill(pid_, SIGHUP)` quando il flusso muore; `setsid()` e
`oom_score_adj = -950` nel figlio; `SIGPIPE` rimesso al default),
[`socket_spec.cpp`](https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/socket_spec.cpp)
(`localabstract`, `localreserved`, `localfilesystem`),
[SERVICES.TXT](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/main/SERVICES.TXT),
[protocol.txt](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/main/protocol.txt).

Altro dal sorgente: `abb_exec:` (Android Binder Bridge) esegue i comandi
`cmd <servizio>` **senza avviare `sh`** ✅ (`services.cpp`: `abb:`/`abb_exec:`).
Per i comandi rapidi del PC che resteranno (per esempio `settings`) costa
meno di `exec:`. Serve la funzione `abb_exec` nel CNXN 🔶.

**Scelta**: avviare con `shell,v2,raw:` e passare i dati su `localabstract:`.
- Rispetto a `exec:` per i dati: niente PTY in mezzo, e adbd legge dal socket
  fino a `max_payload` byte per volta (`local_socket_flush_outgoing` legge in
  ciclo fino a `EAGAIN`) ✅, mentre il PTY ne accumula poco 🔶. Soprattutto,
  **più flussi finiscono nello stesso processo**: il servizio è uno solo per
  tutte le finestre e ha un unico stato (display, volume, ascoltatori).
- Rispetto a una sola uscita `exec:` con i flussi mescolati dentro
  (multiplexing nostro): ogni flusso ADB ha il suo controllo di flusso, e un
  video lento non ferma i comandi a livello ADB. A livello TCP invece la
  coda è una sola (vedi 1.4).
- Il canale d'avvio (`shell,v2`) fa da **cavo di vita**: se cade, adbd manda
  SIGHUP al servizio. Il servizio lo lascia fare (muore) e il custode
  ripristina.

Il **rischio** dei socket astratti è che nello spazio dei nomi astratto
chiunque sul telefono può provare a collegarsi, perché non ci sono permessi
sul file 🔶 (non ho verificato se SELinux lo impedisce alle app). Contromisure:
nome casuale a 128 bit, segreto nel primo messaggio, controllo
`getPeerCredentials().getUid() == 2000`. adbd gira come utente shell (uid
2000) sulle build di produzione ✅ (da noto comportamento di adbd:
`drop_privileges`), quindi ogni collegamento via ADB arriva con uid 2000.

### 1.3 Controllo di flusso, dimensione dei messaggi, delayed ack (fatti)

- `MAX_PAYLOAD` = 1 MiB; il massimo effettivo è il minimo fra quelli
  dichiarati nei due CNXN ✅ ([`adb.h`](https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/adb.h),
  `transport.cpp`: `max_payload = std::min(payload, MAX_PAYLOAD)`).
- **Senza delayed ack**: in ogni direzione di ogni canale c'è un solo
  `WRTE` in volo. Il mittente riprende a leggere dal suo fd solo dopo
  l'`OKAY` ✅ ([`docs/dev/asocket.md`](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/main/docs/dev/asocket.md),
  [`docs/dev/delayed_ack.md`](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/main/docs/dev/delayed_ack.md)).
  Quindi un canale porta al più `max_payload` byte per ogni andata e ritorno
  (RTT). Il Wi-Fi del telefono in risparmio energetico ha RTT tra 5 e
  100+ ms, e il flusso di ogni canale segue quelle oscillazioni 🔶.
- **Con delayed ack** (funzione `delayed_ack`) ✅:
  - chi apre un canale mette in `arg1` dell'`OPEN` quanti byte è disposto a
    ricevere senza conferma. Se la funzione non è stata concordata, `arg1`
    dev'essere 0, altrimenti adbd chiude il canale;
  - l'`OKAY` porta 4 byte (int32 LE) con i byte confermati. Il primo `OKAY`
    di adbd concede `INITIAL_DELAYED_ACK_BYTES` = 32 MiB;
  - chi manda sottrae i byte a ogni `WRTE` e li riaggiunge a ogni `OKAY`, e
    manda finché il saldo è positivo.

  Fonti: `adb.cpp` (`A_OPEN`: «unexpected value of A_OPEN arg1»;
  `send_ready` con il carico di 4 byte), `sockets.cpp`
  (`available_send_bytes`).
- **adbd annuncia sempre `delayed_ack`**; lato PC, `adb` lo annuncia solo con
  `ADB_BURST_MODE=1` ✅ (`transport.cpp`, `supported_features()`). Si attiva
  solo se tutte e due le parti lo annunciano. Oggi Phonestra non lo annuncia,
  quindi è spento. adbd è un modulo aggiornabile (APEX `com.android.adbd`), e
  su Android 14+ ce l'ha quasi certamente 🔶: si verifica leggendo le
  `features=` nel banner CNXN del telefono, che `Adb::wifi` già riceve in
  `dispositivo`.
- Google misura con il delayed ack **circa il 70 % di throughput in più**
  (`adb push` via USB 3) ✅ (`delayed_ack.md`). Via Wi-Fi il guadagno
  riguarda soprattutto la **regolarità** 🔶: il telefono può mandare il
  fotogramma successivo senza aspettare la conferma del precedente, e il PC
  può mandare più tocchi senza aspettare un RTT per ciascuno. Oggi
  `Comandi::dita_insieme` esiste proprio per aggirare questo limite.

**Cosa cambiare nel client** (`src/adb/mod.rs`):
1. CNXN: `features=shell_v2,cmd,stat_v2,delayed_ack,abb_exec`.
2. Se tutte e due le parti hanno `delayed_ack`:
   - `OPEN` con `arg1` = finestra di ricezione **per canale**, piccola e
     voluta: circa 256 KB–1 MB per il video, 64 KB per audio e comandi.
     Non 32 MB: una finestra grande lascia accumulare secondi di video in
     coda, contro la regola «mai accumulare ritardo» (SPECIFICHE §14);
   - gli `OKAY` di risposta portano i 4 byte dei dati **consumati**
     davvero, non solo ricevuti: così la contropressione arriva fino al
     codificatore del telefono;
   - in invio si mantiene un saldo per canale.
3. La coda di ricezione limitata (oggi `unbounded_channel`) va legata alla
   finestra dichiarata.

### 1.4 Una sola connessione TCP: blocco in testa alla coda

Tutti i flussi ADB viaggiano sulla stessa connessione TCP+TLS, in ordine. Il
thread di scrittura di adbd non dà precedenze ✅ (`transport.cpp`: un'unica coda
di `send_packet`). Un `WRTE` video da 1 MB a 5 MB/s occupa il filo per
~200 ms, e audio e comandi arrivati dopo aspettano 🔶. Rimedi:
- dichiarare nel CNXN un **`max_payload` più basso**. 64 KB a 40 Mbit/s fanno
  ~13 ms di attesa al massimo; costa 24 byte di intestazione ogni 64 KB, cioè
  niente 🧪;
- oppure, più avanti, una seconda connessione TCP solo per l'audio.
  Dal Debug wireless si possono aprire più transport, ma ognuno fa la sua
  stretta di mano TLS 🔶. Da valutare solo se la prima misura non basta.

**TLS**: TLS 1.3 con AES-GCM usa le istruzioni AES dell'ARMv8 sul telefono e
AES-NI sul PC. Il costo è trascurabile rispetto alla codifica video 🔶. Il
record TLS massimo è 16 KB, quindi un `WRTE` da 64 KB diventa 4 record.
`TCP_NODELAY` è già attivo da tutte e due le parti ✅ (`set_nodelay(true)` in
`Adb::wifi`; `disable_tcp_nagle` in `daemon/adb_wifi.cpp`).

### 1.5 Quando il collegamento cade

| Causa | Cosa vede il telefono |
|---|---|
| Phonestra chiuso, ucciso (`kill -9`) o PC che si spegne bene | il kernel del PC chiude il TCP, adbd chiude tutti i canali: SIGHUP ai processi `exec:`/`shell`, EOF sui `localabstract` ✅ (provato con il custode, prove §41) |
| Wi-Fi del PC perso, PC spento di colpo, stand-by | **nessun FIN**. adbd non ha keepalive TCP sul Debug wireless: `set_tcp_keepalive` si usa solo lato client, `adb_wifi.cpp` imposta solo `TCP_NODELAY` ✅. Se il telefono sta mandando video, la connessione cade solo quando finiscono le ritrasmissioni TCP (minuti 🔶); se è ferma, può restare aperta a lungo 🔶 |
| Il telefono cambia rete o spegne il Debug wireless | il framework ferma adbd-wifi e i canali si chiudono 🔶 |
| Blocco del telefono con `block_usb_lock` (Samsung) | adbd riparte e tutti i processi figli ricevono SIGHUP ✅ (SPECIFICHE §5.10) |

Da qui la regola: **battito a livello di applicazione** sul canale comandi.
Il PC manda un `PING` ogni secondo e il servizio risponde `PONG`. Il
servizio considera il PC sparito dopo 5 s senza messaggi; il PC considera
perso il telefono dopo 3–5 s senza `PONG`, soglia adatta alle pause del
Wi-Fi in risparmio energetico. Il servizio, quando decide che il PC non c'è,
**termina con `System.exit`** e lascia il ripristino al custode.

**Cosa ripristinare, e chi lo fa** (l'ordine conta):
1. **pausa dei media**: oggi la fa solo il PC alla chiusura ordinata
   (`collegamento.rs`), quindi **con una caduta improvvisa non avviene**, e
   finita la cattura REMOTE_SUBMIX l'audio ripartirebbe dall'altoparlante.
   Deve farla il custode, **prima** del volume;
2. volume multimediale (valore salvato);
3. tempo di spegnimento dello schermo;
4. pannello acceso o spento, secondo le regole di SPECIFICHE §5.9;
5. task dei display virtuali: i display si liberano da soli alla morte del
   processo, perché `VirtualDisplayAdapter` fa `linkToDeath` sul token e in
   `binderDied` distrugge il display ✅ ([VirtualDisplayAdapter.java](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/services/core/java/com/android/server/display/VirtualDisplayAdapter.java)).
   Però le app finiscono sullo schermo principale (Samsung, SPECIFICHE §7.4).
   Il custode conosce gli id dei task e li può togliere dalle recenti
   (`am stack remove` o `IActivityTaskManager.removeTask`) 🔶;
6. blocco del telefono (SPECIFICHE §5.9, punto 4).

I valori restano salvati anche sul PC (`telefoni.toml`), come oggi: è la
riserva per quando anche il custode fallisce.

---

## 2. Ciclo di vita del processo `app_process`

### 2.1 Avvio

- `app_process [opzioni VM] <cartella> [--nice-name=nome] <classe> [argomenti]`.
  `--nice-name` imposta il nome del processo (`setArgv0`) ✅
  ([app_main.cpp](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/cmds/app_process/app_main.cpp)).
  Serve a trovarlo con `pidof phonestra` e a uccidere le istanze rimaste da
  sessioni vecchie, invece di `pkill -f`, che nel progetto ha già fatto danni.
- Senza `--zygote`, app_process avvia `RuntimeInit` e in `onStarted()`
  **avvia il pool di thread Binder** (`ProcessState::startThreadPool()`) ✅:
  le chiamate di ritorno Binder (ascoltatori) funzionano senza fare altro.
- `CLASSPATH` può indicare un `.jar`/`.apk` con dentro `classes.dex`, oppure
  un `.dex` 🔶. Il dex non è precompilato (niente oat), quindi a ogni avvio
  ART verifica le classi e le esegue interpretate o col JIT. Costo tipico di
  un avvio a freddo: 150–400 ms 🧪. Il D8 attuale (`--min-api 34`) va bene.
  Tenere il dex piccolo, senza librerie inutili.
- **Oggi** l'aiutante si copia e si avvia **a ogni richiesta** (elenco app,
  sfondo, appunti sensibili), e ogni volta paga copia, avvio di ART e
  preparazione del contesto finto. Con un servizio di lunga durata la copia
  si fa una volta per collegamento. Il file si cancella appena il processo è
  partito, come fa scrcpy con `unlinkSelf`, perché un dex già caricato non
  serve più su disco 🔶. Evita anche file rimasti se la chiusura fallisce.
- **Istanza unica**: un `LocalServerSocket` con nome fisso di controllo, per
  esempio `phonestra_istanza`, fallisce se il nome è già preso. Il servizio
  nuovo può dire al vecchio di chiudersi, oppure il PC può fare
  `kill $(pidof phonestra)` prima dell'avvio.
- `System.exit` alla fine è obbligatorio: l'SDK avvia thread non-daemon che
  terrebbero vivo il processo ✅ (commento in `Server.main` di scrcpy).
- adbd imposta per i figli `oom_score_adj = -950` ✅, quindi è difficile che
  il low-memory killer li chiuda. Il «phantom process killer» di Android 12+
  riguarda i processi figli delle app, non quelli di adbd 🔶.

### 2.2 Uno o più processi?

| Scelta | Pro | Contro |
|---|---|---|
| **Un servizio per tutto** (proposta) | un avvio di ART; stato condiviso (display, volume, ascoltatori); un solo contesto finto | un crash nativo, per esempio di MediaCodec, ferma tutto (il custode ripristina, il PC riavvia il servizio) |
| Un processo per finestra (oggi, con scrcpy) | isolamento | N avvii di ART (~30–50 MB di RSS ciascuno 🧪), N contesti; lo stato globale (volume, pannello) non ha un proprietario |
| Servizio + **custode** separato | il ripristino avviene anche se il servizio va in crash o è ucciso | un processo in più, piccolo e quasi sempre fermo |

Proposta: **servizio + custode**. Nel passaggio da scrcpy l'audio può restare
per un po' un processo a parte (com'è oggi) e poi confluire nel servizio.

**Il custode alla maniera di scrcpy** ✅ ([CleanUp.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/CleanUp.java)):
- il server lo lancia con `ProcessBuilder("app_process", "/", CleanUp.class…)`;
- il custode fa `Os.setsid()`, cioè entra in una sessione nuova e **non riceve
  il SIGHUP** mandato alla sessione del server;
- legge dallo stdin (un pipe dal server) finché arriva EOF, cioè finché il
  server è vivo, poi ripristina;
- il server gli manda i cambiamenti di stato via via (in scrcpy: 1 byte per il
  pannello).

Per Phonestra il custode riceve righe come `volume 7`, `spegnimento 120000`,
`pannello spento`, `task 123`, `media` (cioè «metti in pausa alla fine»).
Oggi il custode è `sh` con `trap` e `cat`: funziona ✅, ma non sa fare la pausa
dei media né togliere i task, e parla solo con il PC. Non riceve lo stato dal
servizio.

### 2.3 Looper, thread, contesto finto

- **Looper principale**: `Looper.prepareMainLooper()`, oppure la variante di
  scrcpy con `quitAllowed=true` impostato per riflessione su `sMainLooper`
  ✅ ([Server.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/Server.java)).
  Gli ascoltatori che vogliono un `Handler` o un `Executor` (display,
  keyguard, media) si agganciano qui oppure a un `HandlerThread` dedicato.
- **Thread**: uno per leggere i comandi, uno per scrivere su ogni canale (la
  scrittura può bloccarsi quando il Wi-Fi è lento: mai sul thread di
  cattura), uno per il codificatore di ogni finestra, e l'audio come in
  `Audio.java`.
- **Contesto finto** ✅ ([Workarounds.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/Workarounds.java),
  [FakeContext.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/FakeContext.java)):
  - si crea `new ActivityThread()`, si imposta `sCurrentActivityThread` e
    `mSystemThread = true`;
  - su Android 12+ si aggiunge un `ConfigurationController`, altrimenti
    alcuni Samsung vanno in NPE in `DisplayManagerGlobal` (scrcpy #4467,
    già in `Aiuto.java`);
  - `fillAppInfo`: `AppBindData` con `packageName = com.android.shell`, tranne
    sui dispositivi ONYX;
  - `fillAppContext`: `Instrumentation.newApplication(Application.class, FakeContext)`
    in `mInitialApplication`;
  - `FakeContext` estende `ContextWrapper(getSystemContext())`. Rispetto al
    contesto di sistema cambia:
    - `getPackageName`/`getOpPackageName` = `com.android.shell`;
    - `getAttributionSource()` = uid 2000 + `com.android.shell`;
    - `getDeviceId()` = 0;
    - un `ContentResolver` che prende i provider con
      `getContentProviderExternal` (serve per `Settings`);
    - su `clipboard`, `semclipboard` (Samsung) e `activity` sostituisce il
      campo `mContext` del gestore (scrcpy #6224, #6523).

  **Raccomandazione**: copiare questo schema quasi alla lettera, con licenza
  Apache 2.0 e citazione. Oggi `Aiuto.java` ne ha solo una parte:
  `createPackageContext("com.android.shell")` sul contesto di sistema, che
  per lo sfondo ha già dato «package does not belong to uid:2000» (prove).
  Con `FakeContext` l'attribuzione è coerente per tutti i servizi.
- **Caratteri**: la correzione di `caratterePredefinito()` (il typeface
  predefinito mancante in app_process) va fatta una volta sola, all'avvio del
  servizio.

### 2.4 Come il processo si accorge che il PC è sparito

1. SIGHUP dal canale d'avvio `shell,v2`: il processo muore (azione
   predefinita). Va bene, se c'è il custode.
2. EOF o `IOException` sui socket `localabstract`. ART blocca `SIGPIPE`
   (`Runtime::BlockSignals`) ✅, quindi una scrittura su un socket chiuso dà
   `EPIPE` come eccezione e non uccide il processo.
3. Nessun battito per 5 s.

In tutti e tre i casi il servizio esce e il custode ripristina.

---

## 3. Servizi di sistema usati da Phonestra

Permessi della shell letti da
[`packages/Shell/AndroidManifest.xml`](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/packages/Shell/AndroidManifest.xml)
di Android 16 ✅ (552 permessi): vedi §5. Samsung può aggiungerne o toglierne
🔶. Si verificano con `dumpsys package com.android.shell`.

| Funzione | Oggi | API proposta nel servizio | Pubblica? | Permesso (shell ✅) | Stato |
|---|---|---|---|---|---|
| Elenco app del launcher con icone | aiutante a ogni richiesta, base64 | `PackageManager.queryIntentActivities(MAIN/LAUNCHER)` + `loadIcon`, PNG binario, **icone in cache sul PC** per `pacchetto+versionCode` | sì | `QUERY_ALL_PACKAGES` | ✅ già funziona |
| Installazioni e rimozioni | — | `PackageManager.getChangedPackages(seq)` ogni 5–10 s: API pubblica, chiede solo i pacchetti cambiati. In alternativa `LauncherApps.registerCallback` 🔶 | sì | — | 🔶 |
| Notifiche | `dumpsys notification --noredact` ogni 3 s + analisi del testo | `INotificationManager.registerListener(INotificationListener, ComponentName, userId)`: l'ascoltatore «di sistema» usato da SystemUI | nascosta (AIDL); `NotificationListenerService.registerAsSystemService` è `@SystemApi` | **`STATUS_BAR_SERVICE`** (è il controllo di `enforceSystemOrSystemUI`) ✅ | 🔶 da provare, vedi sotto |
| Cancellare una notifica | impossibile | `cancelNotificationsFromListener(token, keys)` con il token dell'ascoltatore | nascosta | come sopra | 🔶 |
| Aprire l'azione o rispondere | — | `PendingIntent.send()` di `contentIntent` o dell'azione; `RemoteInput.addResultsToIntent` per la risposta; `ActivityOptions.setLaunchDisplayId` per il display della finestra | sì | `START_ACTIVITIES_FROM_BACKGROUND` | 🔶 le regole di avvio da background di Android 14+ riguardano chi manda il PendingIntent |
| Media (pausa, stato) | `dumpsys media_session` + `cmd media_session dispatch pause` | `MediaSessionManager.getActiveSessions(null)` + `addOnActiveSessionsChangedListener` + `MediaController.getTransportControls().pause()` | sì (con permesso di sistema) | `MEDIA_CONTENT_CONTROL` ✅ ([MediaSessionManager](https://developer.android.com/reference/android/media/session/MediaSessionManager)) | 🔶 |
| Volume multimediale | `cmd media_session volume --stream 3` | `AudioManager.getStreamVolume/setStreamVolume(STREAM_MUSIC)`; eventi via `VOLUME_CHANGED_ACTION` 🔶 o lettura periodica | sì | `MODIFY_AUDIO_SETTINGS(_PRIVILEGED)` | 🔶 il limite di sicurezza dell'udito (UE) può intervenire se ci sono cuffie |
| Batteria | `dumpsys battery` ogni 30 s | `BatteryManager.getIntProperty(BATTERY_PROPERTY_CAPACITY)` + `isCharging()`; oppure la trasmissione «sticky» `ACTION_BATTERY_CHANGED` 🔶 | sì | `BATTERY_STATS` | 🔶 |
| Wi-Fi (nome della rete) | `cmd wifi status` | `WifiManager.getConnectionInfo().getSSID()`: senza permesso di posizione dà `<unknown ssid>`, ma `NETWORK_SETTINGS` dovrebbe bastare 🔶 | sì | `NETWORK_SETTINGS`, `ACCESS_WIFI_STATE` | 🔶 ripiego: `cmd wifi status` |
| Blocco e sblocco | `dumpsys window \| grep isKeyguardShowing` ogni 3 s | `KeyguardManager.addKeyguardLockedStateListener(executor, l)` (API 33, `@SystemApi`) + `isKeyguardLocked()` | SystemApi | **`SUBSCRIBE_TO_KEYGUARD_LOCKED_STATE`** ✅ ([KeyguardManager](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/app/KeyguardManager.java)) | 🔶 |
| Stato del pannello | messaggi di scrcpy | `DisplayManager.registerDisplayListener` + `Display.getState()`; `PowerManager.isInteractive()` | sì | — | 🔶 |
| Tempo di spegnimento | `settings put system screen_off_timeout` (custode `sh`) | `Settings.System.putInt/getInt` con il `ContentResolver` di `FakeContext` (scrcpy: `wrappers/ContentProvider`, `util/Settings`) | sì | `WRITE_SETTINGS` | ✅ in scrcpy |
| Accendere e spegnere il pannello | `SET_DISPLAY_POWER` di scrcpy | come scrcpy: `SurfaceControl.setDisplayPowerMode(token, mode)`; su Android 14+ i token dei display fisici con `DisplayControl` (classe di `system_server` caricata con `ClassLoaderFactory` + `libandroid_servers`) ✅ ([Device.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/device/Device.java), [DisplayControl.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/wrappers/DisplayControl.java)) | nascosta | `ACCESS_SURFACE_FLINGER`/`DEVICE_POWER` | ✅ in scrcpy. Android 15 ha `DisplayManager.requestDisplayPower` (`cmd display power-off 0`), ma scrcpy la tiene **spenta** (`USE_ANDROID_15_DISPLAY_POWER = false`) ✅; motivo non documentato nel codice, forse gli 0 fotogrammi/s a pannello spento ([scrcpy #5584](https://github.com/Genymobile/scrcpy/issues/5584)) 🔶 |
| Bloccare il telefono alla chiusura | — | `PowerManager.goToSleep` via `IPowerManager` (`DEVICE_POWER`) oppure `KEYCODE_SLEEP` iniettato. Il blocco effettivo dipende da «Blocca subito col tasto di accensione» 🔶 | nascosta | `DEVICE_POWER`, `INJECT_EVENTS` | 🔶 |
| Appunti | scrcpy + aiutante per «sensibile» | `IClipboard.addPrimaryClipChangedListener` + `getPrimaryClip` (firme diverse tra versioni e su Samsung) | nascosta | `READ_CLIPBOARD_IN_BACKGROUND` ✅ | ✅ in scrcpy |
| Schermate protette | `dumpsys window` (flag `FLAG_SECURE`) | resta `dumpsys` per ora 🔶: non c'è un'API per le finestre di altre app | — | `DUMP` | — |

### 3.1 Le notifiche senza `dumpsys` (la scoperta più utile) 🔶

- `NotificationManagerService.registerListener()` chiama
  `enforceSystemOrSystemUI()`, che accetta il chiamante se è di sistema **o se
  ha `STATUS_BAR_SERVICE`** ✅
  ([NotificationManagerService.java](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/services/core/java/com/android/server/notification/NotificationManagerService.java)).
  La shell ha `STATUS_BAR_SERVICE` ✅.
- `ManagedServices.registerSystemService()` registra l'ascoltatore con
  `isSystem = true` e `targetSdk = CUR_DEVELOPMENT` ✅
  ([ManagedServices.java](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/services/core/java/com/android/server/notification/ManagedServices.java)).
  Quando il processo muore, l'ascoltatore viene tolto (morte del binder) 🔶.
- Si ottengono:
  - `onNotificationPosted/Removed` in tempo reale, al posto di un
    `dumpsys` ogni 3 s: meno batteria, meno analisi di testo fragile;
  - `getActiveNotificationsFromListener`;
  - la **cancellazione** con `cancelNotificationsFromListener`. SPECIFICHE
    §8 e §16 oggi la danno per «improbabile».
- **Contenuti sensibili** (Android 15+): gli ascoltatori «non fidati» ricevono
  i codici OTP oscurati (`redactSensitiveNotificationsFromUntrustedListeners`,
  `isUidTrusted`) ✅. La shell ha `RECEIVE_SENSITIVE_NOTIFICATIONS` ✅ e
  probabilmente risulta fidata 🔶: Phonestra vedrebbe gli OTP. Vedi §5.
- Come si fa (🔶): si implementa `android.service.notification.NotificationListenerService`
  e si chiama `registerAsSystemService(context, new ComponentName("com.android.shell", "phonestra.Ascoltatore"), userId)`
  (`@SystemApi`, per riflessione). In alternativa si costruisce il binder
  `INotificationListener.Stub` a mano. Da provare: Samsung potrebbe
  controllare che il componente esista.
- Ripiego: `dumpsys notification --noredact`, che funziona ✅.

---

## 4. API nascoste

### 4.1 Le restrizioni non valgono per `app_process` ✅

- In ART la politica delle API nascoste parte da `kDisabled`
  (`Runtime::hidden_api_policy_`, default dell'opzione `HiddenApiPolicy` in
  `runtime_options.def`). La cambia solo `-Xhidden-api-policy`, che lo zygote
  applica alle app che crea ([runtime.cc](https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/runtime.cc),
  [runtime_options.def](https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/runtime_options.def)).
- `AndroidRuntime.cpp` e `app_main.cpp` non passano opzioni sulle API
  nascoste ✅. Quindi riflessione, `setAccessible`, campi privati e liste
  «blocked» funzionano, come già vediamo con scrcpy e con l'aiutante.
- Le [restrizioni sulle interfacce non-SDK](https://developer.android.com/guide/app-compatibility/restrictions-non-sdk-interfaces)
  riguardano le app. Il rischio vero è un altro: **le firme cambiano**
  tra versioni (Android 14 ha aggiunto `halInputFlags` a
  `AudioRecord.native_setup`; `IClipboard` ha aggiunto `deviceId`) e
  **tra produttori** (Samsung `semclipboard`, Vivo con `AudioRecord`
  modificato, Honor con il pannello).
- Le classi di `system_server` (`com.android.server.*`) non sono nel
  classpath di avvio: servono il `ClassLoader` di `SYSTEMSERVERCLASSPATH` e
  `loadLibrary0` ✅ (DisplayControl di scrcpy).

### 4.2 Come le organizza scrcpy ✅

- `wrappers/ServiceManager`: prende i binder con
  `ServiceManager.getService(nome)` → `I<Servizio>$Stub.asInterface`, un
  oggetto per servizio creato quando serve.
- Un **wrapper per servizio** (`ActivityManager`, `ClipboardManager`,
  `DisplayManager`, `InputManager`, `PowerManager`, `SurfaceControl`,
  `WindowManager`, `StatusBarManager`, `DisplayControl`, `ContentProvider`…).
  Ognuno cerca il `Method` la prima volta, lo tiene in cache, prova le varianti
  di firma e **registra l'errore senza fermare il programma** (per esempio
  `InputManager.injectInputEvent`: in caso di `SecurityException` scrive un
  messaggio chiaro, al massimo ogni 3 s).
- `Workarounds`: correzioni per specifici produttori, scelte con
  `Build.BRAND` e con le versioni, ognuna con il link all'issue che la
  spiega.
- Le differenze di versione vanno a costanti `AndroidVersions.API_xx`. Per
  Phonestra (solo 34+) i rami si riducono a 34/35/36.

### 4.3 Strategia per Phonestra

1. **Guardare cosa c'è, non la versione**: si cerca il metodo con la firma
   voluta, poi le alternative note. `Build.VERSION.SDK_INT` serve solo come
   indizio.
2. **Autotest all'avvio**: il servizio prova ogni capacità (ascoltatore
   notifiche, keyguard, media, pannello, appunti…) e manda al PC `CIAO` con
   l'elenco `capacità: {nome: ok | errore}`. Il PC spegne le funzioni che
   mancano o usa il ripiego (`dumpsys`, `cmd`) e lo scrive nel log.
3. **Stub solo per compilare** (come oggi in `telefono/aiuto/stub`), con le
   chiamate nascoste fatte per riflessione dentro un unico wrapper per
   servizio. Scegliere le API pubbliche o `@SystemApi` ogni volta che la
   shell ha il permesso (`MediaSessionManager`, `KeyguardManager`,
   `AudioManager`, `PackageManager`), che cambiano molto meno delle AIDL.
4. **Riusare i wrapper di scrcpy** (Apache 2.0) con la loro storia di
   correzioni Samsung, invece di riscoprirle: citarli e includere la licenza
   (SPECIFICHE §21).
5. **Raccolta delle firme**: uno strumento di diagnosi
   (`phonestra-prova firme`) che, via servizio, elenca i metodi di
   `IClipboard`, `IInputManager`, `IDisplayManager` e `INotificationManager`
   del telefono. Serve per capire subito un modello nuovo.

---

## 5. Sicurezza

### 5.1 Cosa può fare la shell (Android 16, AOSP) ✅

Fra i permessi rilevanti che la shell ha:
- input e schermo: `INJECT_EVENTS`, `READ_FRAME_BUFFER`,
  `CAPTURE_VIDEO_OUTPUT`, `ACCESS_SURFACE_FLINGER`, `ADD_TRUSTED_DISPLAY`,
  `ADD_ALWAYS_UNLOCKED_DISPLAY`, `INTERNAL_SYSTEM_WINDOW`,
  `CREATE_VIRTUAL_DEVICE`, `MANAGE_DISPLAYS`;
- audio: `CAPTURE_AUDIO_OUTPUT`, `CAPTURE_MEDIA_OUTPUT`, `RECORD_AUDIO`,
  `MODIFY_AUDIO_ROUTING`, `MODIFY_AUDIO_SETTINGS(_PRIVILEGED)`;
- attività e barra di stato: `MANAGE_ACTIVITY_TASKS`, `REAL_GET_TASKS`,
  `START_ACTIVITIES_FROM_BACKGROUND`, `STATUS_BAR`, `STATUS_BAR_SERVICE`,
  `EXPAND_STATUS_BAR`;
- notifiche: `MANAGE_NOTIFICATIONS`, `MANAGE_NOTIFICATION_LISTENERS`,
  `RECEIVE_SENSITIVE_NOTIFICATIONS`, `CONTROL_KEYGUARD_SECURE_NOTIFICATIONS`;
- media e appunti: `MEDIA_CONTENT_CONTROL`, `READ_CLIPBOARD_IN_BACKGROUND`;
- impostazioni e blocco: `WRITE_SECURE_SETTINGS`, `WRITE_SETTINGS`,
  `DEVICE_POWER`, `CONTROL_KEYGUARD`, `DISABLE_KEYGUARD`, `LOCK_DEVICE`,
  `SUBSCRIBE_TO_KEYGUARD_LOCKED_STATE`;
- sistema: `INTERACT_ACROSS_USERS_FULL`, `QUERY_ALL_PACKAGES`,
  `MANAGE_APP_OPS_MODES`, `NETWORK_SETTINGS`, `READ_LOGS`, `DUMP`,
  `PACKAGE_USAGE_STATS`.

**Non** ha `CAPTURE_SECURE_VIDEO_OUTPUT` né `ACCESS_NOTIFICATIONS` ✅. Le
finestre `FLAG_SECURE` restano nere nei nostri display virtuali, ed è giusto
così (SPECIFICHE §7.6).

### 5.2 Regole per il componente

1. **Niente comando generico**: il protocollo espone solo operazioni precise.
   Il PC ha già la shell via ADB; il servizio non deve diventare una seconda
   porta d'ingresso, più facile da raggiungere (socket astratto).
2. **Solo chi è autorizzato** si collega: nome casuale, segreto, uid 2000
   (§1.2). Nessuna porta TCP aperta sul telefono.
3. **Non resta niente**: niente processo dopo la sessione (battito + custode),
   niente file (dex cancellato subito dopo l'avvio), niente impostazioni
   cambiate oltre a quelle elencate, e sempre ripristinate.
4. **Non si toccano le protezioni**: nessuna modifica a Protezione avanzata,
   Blocco automatico, app ops e permessi delle app (decisione già presa,
   SPECIFICHE §5.10). `WRITE_SECURE_SETTINGS` si usa solo per
   `adb_allowed_connection_time` e `adb_wifi_enabled`, come oggi.
5. **Dati sensibili**:
   - gli appunti con `IS_SENSITIVE` non passano (già fatto) e vanno
     controllati **nel servizio**, prima di spedire: il testo di una password
     non deve mai uscire dal telefono;
   - notifiche e appunti non finiscono nei log: `PHONESTRA_DEBUG` scrive solo
     lunghezze e pacchetti;
   - **OTP nelle notifiche**: Android 15 li nasconde agli ascoltatori non
     fidati e durante la condivisione dello schermo. I nostri display virtuali
     non sono una MediaProjection, quindi quella protezione probabilmente non
     scatta 🔶. Proposta: Phonestra rispetta lo stesso principio e mostra le
     notifiche con contenuto sensibile (`Ranking.hasSensitiveContent()` 🔶)
     solo come «Nuovo messaggio da <app>», a meno che l'utente non scelga
     diversamente;
   - l'iniezione di eventi va solo ai display di Phonestra e allo schermo
     principale del drawer, mai «alla cieca»;
   - schermate protette: nessun tentativo di aggirarle (display sicuri,
     `CAPTURE_BLACKOUT_CONTENT`…).
6. **Integrità del jar**: il PC scrive il file con permessi `0644` in
   `/data/local/tmp`, cartella della shell, e lo avvia subito. Un'app non può
   scriverci 🔶, quindi il rischio di sostituzione è basso. Per sicurezza si
   usa un nome casuale per ogni avvio, come oggi.

---

## 6. Rischi

| Rischio | Probabilità | Effetto | Mitigazione |
|---|---|---|---|
| Il PC sparisce senza chiudere il TCP e il servizio continua (pannello spento, volume al massimo, cattura audio attiva) | media | alta | battito di 5 s + custode con `setsid` |
| Un crash nativo (MediaCodec) ferma tutte le finestre | bassa-media | media | custode; il PC riavvia il servizio e ricrea i display (SPECIFICHE §5.7) |
| Firme nascoste diverse su One UI (appunti, input, notifiche) | alta nel tempo | media | wrapper con varianti, autotest, ripieghi `dumpsys`/`cmd` |
| Blocco in testa alla coda TCP (video grande davanti all'audio) | media 🔶 | audio a scatti | `max_payload` più piccolo, delayed ack con finestre piccole, e più avanti una connessione solo per l'audio |
| Il `delayed_ack` rompe qualcosa nel client (conteggi sbagliati → canale fermo) | media (codice nuovo) | alta | test unitari sul conteggio; bandiera per spegnerlo; prova con `adb` di sistema e `ADB_BURST_MODE=1` come riferimento |
| Il `registerListener` delle notifiche è rifiutato da Samsung | media | bassa | ripiego `dumpsys` |
| Qualcun altro sul telefono si collega al socket astratto | bassa | alta | segreto + controllo dell'uid |
| Istanze vecchie ancora vive (dopo un crash del PC) | media | media | `--nice-name`, nome unico di istanza, `pidof` all'avvio |
| Costo di avvio di ART a ogni richiesta (oggi) | certo | bassa-media | servizio di lunga durata |

---

## Domande aperte

1. L'adbd del S23+ e del S26 annuncia `delayed_ack` (e `abb_exec`) nel banner
   CNXN? Da che versione del modulo adbd?
2. `registerListener` / `registerAsSystemService` funziona dalla shell su One
   UI 8.5? La shell risulta «fidata» per le notifiche sensibili?
   `cancelNotificationsFromListener` cancella davvero?
3. `PendingIntent.send()` di una notifica, mandato dalla shell, apre l'attività
   sul display virtuale scelto (`setLaunchDisplayId`), oppure le regole di
   avvio da background di Android 14+ lo bloccano?
4. Quale `max_payload` dà il miglior compromesso tra latenza dell'audio e
   throughput del video via Wi-Fi? Basta un solo TCP?
5. Il blocco del telefono alla chiusura: `goToSleep` basta, o dipende da
   «Blocca subito col tasto di accensione»?
6. Perché scrcpy non usa `requestDisplayPower` di Android 15? Vale anche per
   noi con i display virtuali?
7. SELinux permette a un'app qualsiasi di collegarsi a un socket astratto
   della shell? (Con segreto e controllo dell'uid la risposta conta meno.)
8. Dopo la morte del servizio, i task dei display virtuali finiscono sullo
   schermo principale: il custode li toglie in tempo, prima che l'utente li
   veda?
9. Quanto pesa (RSS, CPU a riposo) un servizio ART sempre acceso, rispetto ai
   processi di oggi?

## Prove da fare sul telefono

Per ogni prova: `phonestra-prova`. Chiudere a fine prova server `adb` e
sessioni, e rimettere tempo di spegnimento e volume (CLAUDE.md).

1. **Banner CNXN**: stampare `Adb::dispositivo` e cercare `features=` con
   `delayed_ack`, `abb_exec`, `shell_v2`. Un minuto.
2. **Delayed ack**: prototipo nel client (CNXN + `OPEN arg1` + `OKAY` a 4 byte).
   Misurare throughput e pause massime di `exec:cat /dev/zero | head -c 50M`
   con e senza, poi un video in finestra: fotogrammi/s, ritardo,
   pause > 100 ms (banco `phonestra-prova banco`).
3. **`exec:` contro `shell,v2,raw:` contro `localabstract:`**: stesso flusso
   (per esempio 5 MB/s di dati finti da un piccolo processo Java), stesse
   misure. Conferma o smentisce il limite del PTY.
4. **`max_payload` 1 MB / 256 KB / 64 KB / 16 KB**: pacchetti audio in ritardo
   (> 40 ms) mentre scorre un video 1080p.
5. **Caduta senza FIN**: con Phonestra collegato, staccare il Wi-Fi del PC
   (o `iptables -j DROP` verso il telefono). Misurare quanto tempo il
   telefono tiene il canale, oggi e poi con il battito; verificare che il
   custode ripristini volume, spegnimento, pannello e che **metta in pausa i
   media**.
6. **Custode Java con `setsid`**: `kill -9` del servizio → ripristino entro 1 s;
   `kill -HUP` → idem.
7. **Tempi di avvio**: `time` di `app_process` fino al primo messaggio `CIAO`
   (a freddo e a caldo), con dex piccolo e grande.
8. **Ascoltatore notifiche**: registrazione, arrivo in tempo reale,
   cancellazione di una notifica di prova (mandata con
   `cmd notification post`), OTP di prova.
9. **Keyguard**: `addKeyguardLockedStateListener`: eventi di blocco e sblocco
   senza `dumpsys`.
10. **MediaSessionManager**: elenco sessioni, pausa, eventi di cambio sessione
    (YouTube, Facebook).
11. **Volume e batteria** via API: `setStreamVolume` da 0 a 15 e ritorno;
    capacità e carica.
12. **Wi-Fi**: `getConnectionInfo().getSSID()` con i permessi della shell.
13. **Socket astratto**: provare da un'app qualsiasi (per esempio Termux, se
    c'è) a collegarsi a `phonestra_…`: rifiutato da SELinux o dal controllo
    dell'uid?
14. **Memoria**: RSS del servizio con 0, 1 e 3 finestre, confrontato con N
    processi scrcpy.
