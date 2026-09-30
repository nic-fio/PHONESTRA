# Componente nostro: lo scheletro (fase 1, 28 set 2026)

Il servizio di lunga durata che sostituirà scrcpy un pezzo alla volta
(`user-decisions.md`, «Via da scrcpy»; piano in `study/README.md`, specifica
principale `study/system.md`). Questa è **solo l'infrastruttura**: processo,
canali, segreto, battito, custode, adattatori delle API nascoste con autotest.
Audio, video e input arriveranno come nuovi tipi di canale e nuovi messaggi.
**Stato (fase 3, §15)**: Phonestra usa solo lui per audio, video, input e
appunti; scrcpy è stato tolto il 28 set 2026.

Legenda come negli studi: ✅ verificato (sul PC, su codice o documentazione),
🔶 ipotesi da verificare sul telefono. **Nessuna parte è ancora stata provata
sul telefono.**

## 1. Pezzi e file

| Dove | File | Cosa fa |
|---|---|---|
| telefono | `android/helper/src/phonestra/Servizio.java` | il servizio: segreto, socket, canali, battito, guardiano, codici d'uscita |
| telefono | `Protocollo.java` | formato dei messaggi del canale comandi e del preambolo |
| telefono | `Custode.java` | avvia il custode (`sh` con `setsid`) e gli manda l'elenco delle azioni di ripristino |
| telefono | `Autotest.java` | prova all'avvio quali API nascoste ci sono, senza usarle |
| telefono | `Nascoste.java` | adattatori delle API nascoste (spostati da `Sistema.java`): servizi, varianti di firma, ripieghi |
| telefono | `Contesto.java` | contesto di sistema e della shell, preparato una volta sola (era `Aiuto.contesto()`) |
| PC | `src/componente.rs` | `Componente`: avvio, pronto, canale comandi, `CIAO`, battito, altri canali, chiusura |
| PC | `src/adb/shell.rs` | servizio `shell,v2,raw:` nel nostro client ADB |
| PC | `src/bin/prova.rs` | `phonestra-prova servizio [secondi] [--sparisci]` |

Il servizio è un comando dell'aiutante (`phonestra.Aiuto servizio`): stesso
jar, stessa compilazione (`android/helper/build.sh`).

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
pezzi, una fascia di 16 ciascuno: audio nessun messaggio proprio (canale
`audio`), video 0x40–0x4f (§12), input 0x50–0x5f (§13); un test Rust
(`input_nostro`) controlla che nessun tipo sia usato due volte. Estendibile così: un tipo nuovo che il servizio
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
| 3 | nessun messaggio dal PC per 5 s (PC sparito: adbd sul Wi-Fi non se ne accorge, `study/system.md` §1.5) |
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
  (`study/system.md` §1.5): pausa dei media 100, volume 200, tempo di
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

## 7. Sicurezza (study/system.md §5)

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

**Scelta dell'avvio: `shell,v2,raw:` e non `exec:`** (`study/system.md`
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
- compilazione del jar (`android/helper/build.sh`), `cargo build`, `cargo test`
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

## 11. Audio (canale `audio`, 28 set 2026)

Primo pezzo sopra lo scheletro: la ricetta scelta con le misure (prove
§42–43, `api-android.md` §1): **loopback** (AudioPolicy con
`ROUTE_FLAG_LOOP_BACK`, gli usi di `Audio.USI`; il telefono intanto tace),
**AAC-LC 192 kbit/s** (PCM come riserva), orari dal conteggio dei campioni,
lettura a priorità −19, lettura/codifica/spedizione su thread separati.
Cattura, lettura e codifica sono **le classi dello strumento di misura**
(`Audio.java`, rese visibili nel pacchetto), non una copia.

| Dove | File | Cosa fa |
|---|---|---|
| telefono | `android/helper/src/phonestra/CanaleAudio.java` | gestore del tipo `audio` (registrato in `Servizio.TIPI`) |
| PC | `src/audio_nostro.rs` | apertura del canale, pacchetti, riproduzione (`avdec_aac`), copie per chi registra |
| PC | `src/bin/prova/audio_componente.rs` | `phonestra-prova audio-componente` |

**Tipo del canale**: `audio` o `audio:aac` (AAC), `audio:pcm` (PCM s16le,
48 kHz, stereo). Nessun messaggio nuovo sul canale comandi. Un solo canale
audio alla volta: uno nuovo ferma il vecchio e aspetta (al massimo 3 s) che
abbia tolto la sua politica prima di registrare la propria. Formato
sconosciuto: una riga `errore` e il canale si chiude.

**Pacchetti** (servizio → PC, stesso formato dello strumento):
`orario u64 BE · lunghezza u32 BE · dati`. Bit 61 dell'orario = testo UTF-8
`tipo chiave=valore …`; bit 62 = configurazione del codec; altrimenti dati con
orario in µs = campioni letti × 10⁶ / 48000 (per difetto).
1. Primo pacchetto, sempre un testo: `inizio formato=aac|pcm frequenza=48000
   canali=2 [bitrate=192000] sorgente=loopback buffer_ms=… registrazione=istanza|statica`,
   oppure `errore …` (cattura non partita; poi il canale si chiude).
2. AAC: il pacchetto di configurazione (AudioSpecificConfig, 2 byte `11 90`
   per AAC-LC 48 kHz stereo) prima di qualsiasi dato: lo garantisce
   `MediaCodec` (`BUFFER_FLAG_CODEC_CONFIG` esce per primo).
3. Dati (AAC: un frame di 1024 campioni; PCM: 1024 campioni) e testi
   `lettura`, `misura` (uno al secondo, come lo strumento: `persi`, `zeri`,
   `deriva_ms`, `nice`…), `avviso`, `errore`.
Il PC non manda niente: chiudere il canale ferma la cattura.

**Thread sul telefono**: `audio-lettura` (−19, `Audio.Lettura`),
`audio-codifica` (`Audio.codifica`), `audio-spedizione` (scrive sul socket;
coda di ~5 s che scarta i più vecchi, contati in `persi`),
`audio-sentinella` (legge dal socket solo per accorgersi della chiusura anche
quando la spedizione non ha niente da scrivere). Ogni thread cattura i propri
errori, anche gli `Error` delle API nascoste: un problema dell'audio chiude il
canale con una riga `errore`, mai il servizio.

**Politica audio, chi la toglie**:
- chiusura del canale (dal PC, o canale nuovo): `CanaleAudio` ferma il
  registratore e chiama `unregisterAudioPolicy` (`Audio.Cattura.chiudi`);
- fine del servizio con `System.exit` (FINE, battito mancato, canale comandi
  chiuso): gancio di chiusura (`audio-fine`), prima del `Runtime.halt` di 2 s;
- processo morto di colpo (`kill -9`, crash nativo): **Android**. ✅ Codice
  AOSP (`AudioService.registerAudioPolicy`, ramo main): la politica è un
  `AudioPolicyProxy` legato con `linkToDeath` al binder di callback della
  politica, che vive nel nostro processo; `binderDied()` chiama `release()`,
  che la toglie. 🔶 Da vedere sul telefono con `--uccidi`.
- Il custode **non** ha un'azione per l'audio: nessun comando di shell toglie
  la politica di un altro processo, e non serve. Il loop-back non cambia
  impostazioni del telefono: tolta la politica, il telefono torna a suonare.
- Controllo nelle prove: `dumpsys audio`, sezione «Audio policies»: ogni
  politica stampa una riga `android.media.audiopolicy.AudioPolicyConfig:` e,
  per il nostro mix, `* route flags=0x2` (formato ✅ da AOSP,
  `AudioPolicyConfig.toLogFriendlyString`); `audio_nostro::conta_politiche`.

**Lato PC** (`src/audio_nostro.rs`):
- `Flusso::apri(&componente, Formato::Aac)` apre il canale e aspetta
  `inizio` (10 s); `prossimo()` dà `Configurazione`, `Testo`, `Dati`;
- `Riproduzione`: `appsrc` con caps `audio/mpeg, mpegversion=4,
  stream-format=raw, codec_data=<configurazione>` → `avdec_aac` →
  `audioconvert ! audioresample ! autoaudiosink` (PCM: caps raw, senza
  decodificatore). Orari: `Orari` (regolari, riallineo oltre 60 ms) e
  `Margine` (80 ms, +40 ms a ogni ritardo fino a 300, riallineo oltre 200 ms):
  **stessa logica** di `audio::riproduci`, copiata (non condivisa) perché
  `audio.rs` sparirà con scrcpy; le durate vengono da `Durate` (conteggio dei
  campioni: 21 333/21 334 µs, nessun errore accumulato);
- `riproduci(&componente)`: **la funzione da chiamare al posto di
  `audio::riproduci(&adb)`** (AAC; `PHONESTRA_AUDIO_CODEC=pcm` per il PCM).
  Vuole il `Componente` già avviato; finisce se il canale si chiude (errore)
  e, annullata, chiude il canale;
- copie per chi registra: `audio_nostro::ascolta()` (orario regolare, frame
  AAC grezzo), `caps_registrazione()` (caps con `codec_data` dell'audio in
  corso), `durata_pacchetto()`;
- `gstreamer1.0-libav` (per `avdec_aac`) è già nell'AppImage
  (`packaging/collect.sh`: `libav`), come `isomp4`.

**Per collegarlo a Phonestra** (da fare dopo le prove sul telefono):
1. `collegamento.rs`: avviare il `Componente` e al posto di
   `audio::riproduci(&adb)` lanciare `audio_nostro::riproduci(&componente)`.
   `apri_canale` vuole `&Componente`: il componente va tenuto in un `Arc`
   (o il compito dell'audio deve possederlo) e chiuso con `chiudi()` alla fine
   del collegamento. Il volume al massimo e il tempo di spegnimento restano
   al custode di oggi finché non passano al custode del servizio.
2. Registrazione (`finestra.rs`, `inizia_registrazione`): togliere le caps
   Opus dalla descrizione dell'`appsrc name=audio`, e subito dopo il
   `parse::launch` impostarle con `audio.set_caps(audio_nostro::caps_registrazione().as_ref())`
   (se `None`, l'audio non è in corso: registrare senza audio o con le caps
   Opus di oggi); `crate::audio::ascolta()` → `crate::audio_nostro::ascolta()`;
   la durata del buffer da 20 ms a `durata_pacchetto(Formato::Aac, _)`
   (21,333 ms). `mp4mux` accetta l'AAC grezzo con `codec_data`: ✅ provato
   sul PC (`registrazione_mp4_con_aac`). Non fatta ora perché passerebbe la
   registrazione all'audio nuovo mentre la riproduzione usa ancora scrcpy.

### 11.1 Prove sul telefono (audio)

Telefono sbloccato, Phonestra chiuso, qualcosa che suona (un reel parlato).
1. `phonestra-prova audio-componente 60` — AAC. Attesi: riga `inizio …
   formato=aac … registrazione=…`, configurazione `[11, 90]`, misure con
   `nice -19` e `persi 0`; riassunto con 0 orari irregolari, 0 zeri, 0 tagli;
   politiche «prima N, durante N+1 (1 loop-back), dopo la chiusura del canale
   N, alla fine N»; tutti `ok`, «prova riuscita». Ascoltare
   `phonestra-prova.aac`. Durante la prova il telefono tace; dopo suona.
2. `phonestra-prova audio-componente 60 aac --ascolta` — come sopra, e
   l'audio dal vivo dalle casse del PC con la pipeline vera: ascoltare se ci
   sono interruzioni; «ascolto: 0 pacchetti in ritardo» atteso sul Wi-Fi buono.
3. `phonestra-prova audio-componente 30 pcm` — riserva PCM, `phonestra-prova.wav`.
4. `phonestra-prova audio-componente 20 --uccidi` — `kill -9` a metà: attesi
   servizio «Uscito(137)», politiche alla fine come prima (tolta da Android),
   nessun processo né jar; il telefono torna a suonare.
5. Se la riga «ATTENZIONE: la politica non si vede in dumpsys» compare,
   guardare a mano `phonestra-prova shell 'dumpsys audio' | grep -A12 'Audio policies'`
   durante una prova lunga.

✅ Sul PC: compilazione del jar; test del formato dei pacchetti a pezzi,
durate esatte, orari, margine (stessi numeri della logica di scrcpy),
controllo degli orari, ADTS, conteggio delle politiche; **AAC vero** (codificato
da `avenc_aac` a 48 kHz stereo) decodificato da `avdec_aac` con le caps dalla
configurazione, spinto nella pipeline di riproduzione e scritto in MP4 da
`mp4mux`. 🔶 Sul telefono: tutto il canale (nessuna parte provata), in
particolare il gancio di chiusura, la sentinella (fine del file quando il PC
chiude il `localabstract`) e la sostituzione di un canale con un altro.

## 12. Video (fase 2, 28 set 2026)

Il pezzo che sostituirà scrcpy per le finestre delle app e per lo schermo del
telefono nel drawer. **Stesse funzioni di oggi** (`user-decisions.md`):
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
  Misurato in §43: ~40 ms, ma con una pagina animata. **Prima prova del modulo
  (S23+, Orologio fermo): nessun fotogramma chiave entro 1 s**, e solo 28
  fotogrammi in 15 s: `c2.qti.avc.encoder` ignora `repeat-previous-frame-after`,
  quindi a schermo fermo non esce niente e la richiesta aspetta il prossimo
  cambiamento (una finestra ferma resterebbe nera). Correzione: se il
  fotogramma chiave non esce entro 80 ms, lo schermo si ridisegna staccando e
  riattaccando la Surface del codificatore (`VirtualDisplay.setSurface(null)` e
  di nuovo la sua); se ancora niente, una seconda volta dopo altri 160 ms.
  Vale anche per lo specchio. La prova segna le richieste «a schermo fermo».
- **Chiusura del canale video**: nella prima prova il PC non vedeva chiudersi
  `video:<id>` dopo `VIDEO_CHIUDI`. Errore del telefono, non della prova: il
  thread del canale è fermo in `read` sullo stesso socket, e in Linux `close`
  di un descrittore con una lettura in corso in un altro thread non chiude
  davvero il socket (niente fine del flusso ad adbd). Ora prima
  `shutdownOutput`/`shutdownInput` (fine del flusso e lettura sbloccata), poi
  `close`.
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
  `requestDisplayPower`, che scrcpy ha visto bloccare l'input (study/video.md §4.5).
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
let servizio = Condiviso::avvia(c);          // smista risposte ed eventi (componente.rs, §14)
let SessioneNostra { codec, display, video: flusso, mut comandi, mut eventi, .. } =
    SessioneNostra::avvia(&servizio, &Opzioni { .. }).await?;   // stesse Opzioni di sessione.rs
comandi.avvia_app("com.android.chrome").await?;             // o informazioni_app
leggi_pacchetto(&mut flusso).await?;                        // come oggi
comandi.ridimensiona(l, a).await?; comandi.ricomincia_video().await?;
video_nostro::pannello(&servizio, false)?;
while let Some(e) = eventi.recv().await { /* Evento::Orientamento, Protetta, Spostata… */ }
comandi.chiudi(true).await?;                                // true = via dalle recenti
servizio.chiudi().await?;                                   // FINE del servizio
```

Lo smistamento (prima dentro `video_nostro`, nel `Video`) sta ora in
`componente.rs` per tutti i pezzi: vedi §14.

**Cosa restava per collegarlo a Phonestra** (fatto, §14): un `Video` per collegamento in
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

✅ Sul telefono (S23+, Android 16, `video-componente app`): primo fotogramma
in 590 ms, ridimensionamento in 83 ms, pannello, eventi, schermata protetta,
pulizia, ffprobe. Da riprovare dopo le due correzioni sopra: fotogrammi chiave
a schermo fermo (ridisegno forzato: ipotesi che `setSurface` faccia comporre
subito un fotogramma) e chiusura del canale.

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

## 13. Input (modulo 3, 28 set 2026)

Tocchi, rotellina, tasti, testo, «indietro» e appunti sul canale `comandi`.
Scopo: **le stesse funzioni di oggi con scrcpy** (`sessione::Comandi`,
`appunti.rs`), niente di più (`user-decisions.md`). Codice nostro, scrcpy
solo come documentazione (`study/input.md` §2–§4, §6). **Non ancora
collegato all'interfaccia**: scrcpy resta in uso. **Niente è stato provato sul
telefono.**

| Dove | File | Cosa fa |
|---|---|---|
| telefono | `Input.java` | messaggi 0x50–0x5f, coda del thread «input», iniezione, dita, scalatura, appunti |
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

### Messaggi (fascia 0x50–0x5f, big-endian)

| Tipo | Nome | Contenuto |
|---|---|---|
| 0x50 | `TOCCHI` | `display i32 · larghezza u16 · altezza u16 · n u8 · n × (dito i64 · azione u8 · x i32 · y i32 · pressione f32)` |
| 0x51 | `ROTELLINA` | `display i32 · x i32 · y i32 · larghezza u16 · altezza u16 · orizzontale f32 · verticale f32` |
| 0x52 | `TASTO` | `display i32 · azione u8 · codice u32 · ripetizione u32 · meta u32` |
| 0x53 | `TESTO` | `display i32 · testo UTF-8` |
| 0x54 | `INDIETRO` | `display i32 · azione u8` |
| 0x55 | `APPUNTI_SCRIVI` | `display i32 · incolla u8 · testo UTF-8`; risposta vuota se l'id non è 0 |
| 0x56 | `APPUNTI_LEGGI` | domanda vuota; risposta `stato u8 · testo` |
| 0x57 | `APPUNTI_ASCOLTA` | `attivo u8`; risposta vuota se l'id non è 0 |
| 0x58 | `APPUNTI_CAMBIATI` | servizio → PC, spontaneo: `stato u8 · testo` |
| 0x5c | `CONTEGGI` | domanda di diagnosi; risposta `chiave=valore` (iniettati, falliti, scartati, avvisi_appunti, ultimo_errore) |
| 0x5d | `PROVA` | domanda: comando di prova in testo (`InputProva.java`) |

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
  chiama `Input.dimensioneVideo` a ogni misura mandata al PC e
  `Input.dimentica(display)` alla chiusura di uno schermo.
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

### Cosa restava per collegarlo a Phonestra

Finestre e drawer: fatto (§14). Appunti: ancora da fare.

- In `finestra.rs`: al posto di `sessione::Comandi` un `InputNostro` con
  `componente.mittente()` e l'id dello schermo della finestra (dal modulo
  video). I metodi hanno lo stesso nome e significato. Restano fuori
  dall'input (modulo video o comandi): `avvia_app`, `ridimensiona`,
  `pannello`, `ricomincia_video`, `chiudi`.
- Il modulo video chiama già `Input.dimensioneVideo` quando manda al PC una
  nuova misura del flusso (`SessioneVideo.sostituisci`) e `Input.dimentica`
  quando chiude uno schermo (`SessioneVideo.chiudi`): da lì in poi vale il
  confronto esatto della misura, come scrcpy.
- In `appunti.rs`: al posto della sessione scrcpy `clipboard_autosync` e di
  `app::appunti_sensibili`, `ascolta_appunti(true)` all'apertura del
  collegamento e gli `APPUNTI_CAMBIATI` letti da `Componente::ricevi`
  (`Appunti::da_avviso`): `Testo` → stessi controlli di oggi (lunghezza,
  rimbalzo), `Sensibili`/`Sconosciuti` → non passa.

## 14. Fase 2: video e input di finestre e drawer (28 set 2026)

Dopo l'audio (§11, prove §47), anche **video e input** delle finestre delle
app e dello schermo del telefono nel drawer passano dal componente nostro.
Stesse funzioni di oggi (`user-decisions.md`), niente di nuovo
nell'interfaccia. **Non ancora provato sul telefono**: lista delle prove in
`phase2-tests-todo.md`.

### Un servizio per collegamento

- `Collegamento` avvia **un solo** `Componente` per collegamento (compito
  `gira_motore`), lo avvolge in un `componente::Condiviso` (si clona) e lo
  pubblica come `collegamento::Motore::Nostro` (`Collegamento::motore()`, un
  `watch`). Audio (`audio_nostro::riproduci`), finestre e drawer usano quello.
- **Chiusura in ordine**: alla chiusura di Phonestra prima le sessioni (che
  chiedono al servizio di togliere le app dalle recenti), poi `FINE` al
  servizio (il custode sul telefono ripristina, pannello compreso), poi il
  custode del tempo di spegnimento.
- **Servizio caduto** col telefono ancora collegato: il motore torna `None`,
  le finestre vedono la sessione cadere e aspettano; il servizio riparte dopo
  2 s. Dopo 3 cadute nello stesso collegamento: scrcpy fino al prossimo
  collegamento.
- **Riserva scrcpy** (`Motore::Scrcpy`): se il servizio non parte, oppure con
  `PHONESTRA_COMPONENTE=scrcpy`, tutto (video, input, audio) passa da scrcpy
  come prima; `sessione.rs` resta. `PHONESTRA_COMPONENTE_AUDIO=scrcpy` forza
  solo l'audio di scrcpy (prove di confronto).

### Smistamento (`componente.rs`)

`Condiviso` tiene il `Componente` in un compito (`smista`) e smista i
messaggi del canale comandi (`Smistamento`, provato con test senza rete):

| Messaggio | Va a |
|---|---|
| risposta (bandiera `RISPOSTA`) | chi ha fatto la domanda con quell'id (`domanda`, `apri_sessione`); `ERRORE` → errore della domanda; risposte non attese: solo gli `ERRORE` nel log |
| `VIDEO_EVENTO` (`id=<sessione>`) | la sessione aperta con `apri_sessione`; `evento=fine` la chiude; eventi arrivati prima della risposta di apertura tenuti 2 s e consegnati con la risposta |
| altri spontanei | chi si è iscritto al tipo (`iscrivi(tipo)`: servirà agli appunti, `APPUNTI_CAMBIATI`) |

Domande senza risposta entro 5 s: errore. Tocchi e tasti non passano dal
compito: `Condiviso::mittente()` li mette direttamente in coda al canale
comandi (id 0, nessuna risposta). I canali (`audio`, `video:<id>`) si aprono
con `Condiviso::apritore()` (`componente::Apritore`), senza passare dal
compito. `video_nostro::Video` (lo smistamento solo video) è stato tolto.

### Finestre e drawer (`finestra.rs`)

Una sessione sceglie il motore all'avvio (`aspetta_telefono`: collegamento e
motore usabile); `Telefono` (enum) ha gli stessi metodi con scrcpy
(`sessione::Comandi`) o col componente (`InputNostro` + `ComandiVideo`). Il
resto del giro della sessione (mouse, tastiera, zoom, pressione lunga,
registrazione, ridimensionamento, ricreazione del display, colonna delle app
solo verticali, pannello «in mano») è lo stesso codice per tutti e due.

| Oggi con scrcpy | Col componente nostro |
|---|---|
| `Sessione::avvia` per finestra | `SessioneNostra::avvia(&servizio, &opzioni)` |
| numero del display dal messaggio «New display» | dalla risposta di apertura, subito (specchio: resta −1 come oggi) |
| `cmd window set-ignore-orientation-request` dal PC | lo fa il telefono all'apertura |
| `am start … APPLICATION_DETAILS_SETTINGS` dal PC | `ComandiVideo::informazioni_app` |
| `chiedi_orientamento` (dumpsys a 2,5 s, 4 s e quando la finestra si allarga) | `Evento::Orientamento` (all'inizio e quando cambia) |
| `Collegamento::protetti` (dumpsys nel giro dei 3 s) | `Evento::Protetta`; il giro dei 3 s non chiede più `COMANDO_FINESTRE` |
| `togli_dalle_recenti(adb, display)` | `ComandiVideo::chiudi(true)` |
| canali chiusi a fine sessione (il server muore) | `ComandiVideo::chiudi(false)` + canale video chiuso |
| «rispegni»: il server che muore riaccende il pannello | non serve: il pannello è uno per telefono (resta solo per scrcpy) |
| server scrcpy morto: canale video chiuso | anche `Evento::Fine` (il telefono chiude la sessione): si riapre come dopo una caduta |

`Evento::Spostata` e `Evento::Rimosso` non fanno niente (oggi non c'è
l'equivalente: «App aperta sul telefono – riportala qui» è nelle SPECIFICATION
ma non ancora nel programma); si vedono con `PHONESTRA_DEBUG=1`.

### Cosa resta

- **Appunti** (prossimo passo): oggi ancora la sessione scrcpy
  `clipboard_autosync` più `app::appunti_sensibili`. Da fare: all'avvio del
  servizio `APPUNTI_ASCOLTA 1` e `Condiviso::iscrivi(APPUNTI_CAMBIATI)` in
  `appunti::ascolta` (vedi §13, «Cosa resta»). L'**incolla** delle finestre
  passa già dal componente (`InputNostro::incolla`); il rimbalzo verso il PC
  lo ferma ancora `Collegamento::e_un_rimbalzo`.
- Il pannello resta spento durante una ricreazione del display (con scrcpy si
  riaccendeva un attimo): differenza voluta, invisibile se non migliore.
- Dopo le prove sul telefono: togliere scrcpy da AppImage e licenza (passo 4
  del piano), `sessione.rs` e la riserva.

## 15. Fase 3: scrcpy tolto (28 set 2026)

Dopo le prove della fase 2 (prove-collegamento §44–50: audio, video, input e
appunti dal componente nostro, senza difetti) l'utente ha deciso di togliere
scrcpy **del tutto** (`user-decisions.md`, «Via da scrcpy», passo 4).

- **Tolti**: `telefono/scrcpy-server-v4.1` (e il suo `include_bytes!`),
  `telefono/LICENZA-scrcpy.txt` (anche dall'AppImage, `collect.sh`),
  `src/sessione.rs` (sessione, `Comandi`, `lancia`, `apri_canale`,
  `togli_dalle_recenti`, messaggio «New display»), `src/audio.rs` (audio Opus),
  `collegamento::Motore` e le variabili `PHONESTRA_COMPONENTE`,
  `PHONESTRA_COMPONENTE_VIDEO`, `PHONESTRA_COMPONENTE_AUDIO`, il ripiego
  automatico su scrcpy, il «rispegni» del pannello, `Collegamento::protetti` e
  i `dumpsys` del PC per schermate protette e orientamento
  (`notifiche::COMANDO_FINESTRE`, `COMANDO_ORIENTAMENTI` e le loro letture),
  l'ascolto degli appunti con `clipboard_autosync`, `app::appunti_sensibili` e
  il comando `appunti-sensibili` dell'aiutante, i comandi di `phonestra-prova`
  che usavano scrcpy (`tocchi`, `video`, `audio`, `banco`, `appunti`).
- **Spostati**: `Opzioni`, `Pacchetto`, `leggi_pacchetto`, `intestazione`,
  `nome_codec` in `video_nostro::flusso` (il formato dei pacchetti è quello di
  `SessioneVideo.java`); `diagnosi` in `lib.rs`.
- **Collegamento**: `Collegamento::componente()` pubblica il `Condiviso`
  (prima `Motore::Nostro`). Se il componente non parte o si ferma
  `CADUTE_MASSIME` (3) volte nello stesso collegamento, `gira_componente`
  smette di riprovare e pubblica il motivo in `Collegamento::guasto()`: il
  drawer mostra la pillola rossa «Phonestra non parte sul telefono» e il velo
  sul telefono disegnato con la spiegazione e «Riconnetti ora»; le finestre
  mostrano lo stesso velo. «Riconnetti ora» (`riconnetti_ora`) fa ripartire i
  tentativi; al prossimo collegamento si riprova comunque da capo.
- **Registrazione**: l'audio va nel file solo se è AAC del componente; senza
  audio in corso (o col PCM di prova) la registrazione resta senza audio.
- Nessun comportamento cambiato col componente funzionante: le righe «con
  scrcpy» delle sezioni precedenti restano come storia.
