# Studio: l'audio del telefono nel componente nostro (Android 14–16, processo shell)

*28 set 2026. Studio preparatorio, nessun file del repository modificato.
Legenda: **[V]** = verificato leggendo codice sorgente o documentazione ufficiale
(link accanto); **[M]** = misurato da noi (prove-collegamento §§ citati o dati del
28 set); **[I]** = ipotesi da verificare sul telefono.*

## Riepilogo e raccomandazioni

1. Le due catture che la shell ha a disposizione, «output» (`REMOTE_SUBMIX`) e
   «playback» (`AudioPolicy` con loop-back), passano **tutte e due dallo stesso
   dispositivo virtuale "remote submix"**. Questo non ha un orologio hardware:
   chi scrive (il mixer di Android) viene frenato da un "freno a sleep" (MonoPipe)
   che ne varia la velocità da 0,57× a 2× [V]. **Ipotesi principale** [I]: è questa
   la causa comune dei vuoti nei lettori (playback), del calo a 24 fotogrammi/s
   (output, prove §17 banco) e delle micro-interruzioni.
2. Il registratore dello schermo Samsung (audio perfetto) fa suonare anche il
   telefono: quasi certamente usa la variante **LOOP_BACK_RENDER**, in cui il
   mixer segue l'orologio **hardware** dell'altoparlante [I]. È la prima strada da provare.
3. Gli orari a raffica di scrcpy **sono spiegati dal codice** [V]: con Opus scrcpy
   sovrascrive l'orario di ogni pacchetto con l'ora di uscita dal codificatore
   (`System.nanoTime()` − 20 ms), e con le altre fonti usa `AudioTimestamp.nanoTime`
   senza correggerlo con `framePosition`. Contare i campioni, come fa già
   `Audio.java`, è la scelta giusta.
4. Il codificatore Opus di Android (`c2.android.opus.encoder`, software) accumula
   correttamente i blocchi di 1024 campioni in pacchetti da 960 [V]: **non è lui**
   a creare i vuoti. I "buchi dentro l'audio compresso" si formano **prima**, nella
   cattura (zeri messi dal remote submix, o mixer senza dati) [I].
5. Raccomandazione: **primo passo del nostro audio = PCM non compresso** (1,536
   Mbit/s, latenza di codifica zero), con un contatore di "silenzi digitali" sul
   telefono. Serve a capire dove nascono i vuoti; AAC-LC resta come passo
   successivo (o PCM come scelta definitiva se il Wi-Fi regge).
6. Confrontare sul telefono, con lo stesso programma di misura, **tre sorgenti**:
   REMOTE_SUBMIX, LOOP_BACK, LOOP_BACK_RENDER; misurando underrun dei lettori
   (`dumpsys media.audio_flinger`), fotogrammi/s e zeri nel PCM.
7. Attenzione: `memoria/api-android.md` §1 ha scelto REMOTE_SUBMIX, ma le prove
   del 26 set (prove-collegamento, «banco») misuravano **24/s invece di 60/s** con
   quella sorgente [M]. La scelta va rimisurata con il nostro componente prima di
   considerarla chiusa.
8. Se si usa AudioPolicy: catturare **tutti gli usi** (non solo MEDIA: scrcpy 4.1
   perde l'audio dei giochi, corretto solo nel ramo `dev`) e chiamare
   `voiceCommunicationCaptureAllowed(true)` **prima** di `build()` (scrcpy lo
   chiama dopo, senza effetto) [V].
9. Priorità del thread: `THREAD_PRIORITY_URGENT_AUDIO` sul thread che legge,
   `AudioRecord` con 0,5 s di buffer (già così), lettura bloccante a 1024
   campioni [V/I].
10. Microfono e fotocamera come webcam del PC sono **tecnicamente possibili** dalla
    shell (ha `CAMERA`, `BACKGROUND_CAMERA`, `RECORD_AUDIO`,
    `RECORD_BACKGROUND_AUDIO`) [V], ma SPECIFICHE §12 dice che l'utente li ha
    **tolti il 27 set**: qui solo per completezza.

---

## 1. Sorgenti di cattura disponibili alla shell

### 1.1 Permessi della shell (uid 2000) su 14, 15, 16 [V]

Dal manifesto di `com.android.shell`
([android14](https://github.com/aosp-mirror/platform_frameworks_base/blob/android14-release/packages/Shell/AndroidManifest.xml),
[android15](https://github.com/aosp-mirror/platform_frameworks_base/blob/android15-release/packages/Shell/AndroidManifest.xml),
[main](https://github.com/aosp-mirror/platform_frameworks_base/blob/main/packages/Shell/AndroidManifest.xml)),
identici per l'audio nelle tre versioni:
`CAPTURE_AUDIO_OUTPUT`, `CAPTURE_MEDIA_OUTPUT`,
`CAPTURE_VOICE_COMMUNICATION_OUTPUT`, `MODIFY_AUDIO_ROUTING`,
`MODIFY_AUDIO_SETTINGS(_PRIVILEGED)`, `MANAGE_AUDIO_POLICY`, `RECORD_AUDIO`,
`RECORD_BACKGROUND_AUDIO`, `CALL_AUDIO_INTERCEPTION`, `QUERY_AUDIO_STATE`,
più `CAMERA`, `BACKGROUND_CAMERA`, `SYSTEM_CAMERA`.
(Il mirror GitHub di AOSP non è aggiornato ad Android 16 QPR; Samsung può
togliere permessi: da verificare con `dumpsys package com.android.shell`.)

### 1.2 `REMOTE_SUBMIX` («output» in scrcpy)

- `AudioRecord` con `MediaRecorder.AudioSource.REMOTE_SUBMIX`, richiede
  `CAPTURE_AUDIO_OUTPUT` [V] ([AudioSource](https://developer.android.com/reference/android/media/MediaRecorder.AudioSource#REMOTE_SUBMIX)).
- Quando la cattura parte, la policy rende disponibile il dispositivo d'uscita
  `REMOTE_SUBMIX` con indirizzo `"0"` e il motore instrada lì le strategie
  **MEDIA, DTMF, ACCESSIBILITY, REROUTING** [V]
  ([Engine.cpp](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/services/audiopolicy/enginedefault/src/Engine.cpp),
  ramo `STRATEGY_MEDIA`: `devices2.add(remoteSubmix)`).
- Effetti sul telefono [V dal codice del motore AOSP; I per Samsung, che ha un
  motore di policy suo]:
  - **media** (video, musica, giochi): solo nel submix → altoparlante muto
    (confermato dalle prove [M]);
  - **suonerie e sveglie** (`STRATEGY_SONIFICATION`): *«no sonification on remote
    submix»* → escono dall'altoparlante e **non** arrivano al PC;
  - **notifiche** (`SONIFICATION_RESPECTFUL`): tolto il submix fuori chiamata →
    altoparlante;
  - **chiamate** telefoniche e VoIP che usano `setMode(IN_COMMUNICATION)`: la
    strategia PHONE prende il sopravvento e in chiamata anche i media possono
    uscire dall'auricolare/altoparlante: è l'issue scrcpy
    [#4087](https://github.com/Genymobile/scrcpy/issues/4087) (col microfono
    del gioco attivo l'audio smette di arrivare al PC; spiegazione di yume-chan
    nello stesso thread).
- **Ignora il rifiuto di cattura** delle app (`allowAudioPlaybackCapture=false`)
  perché prende il mix finale [V: documentazione scrcpy
  [audio.md](https://github.com/Genymobile/scrcpy/blob/master/doc/audio.md)] —
  che le app protette arrivino davvero al PC è ancora *da verificare* (SPECIFICHE §10).
- Il volume del telefono **si applica** (il mix è dopo il volume) [I, coerente con
  §41: Facebook a volume 0].
- Effetto misurato [M, prove «banco», 26 set]: YouTube in display virtuale a
  **24/s in 4 prove su 5** invece di 60/s. E il 28 set: vuoti dei lettori
  59 → 1, ma micro-interruzioni dentro l'audio.

### 1.3 `AudioPolicy` con loop-back («playback» in scrcpy)

- **Non** è `AudioPlaybackCaptureConfiguration` (che richiede un `MediaProjection`
  e quindi la finestra di consenso): scrcpy registra direttamente una
  `AudioPolicy` con un `AudioMix` di ruolo `MIX_ROLE_PLAYERS` e
  `ROUTE_FLAG_LOOP_BACK`, poi `createAudioRecordSink()`. Serve
  `MODIFY_AUDIO_ROUTING`, dato alla shell da Android 13 [V]
  ([AudioPlaybackCapture.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/audio/AudioPlaybackCapture.java),
  issue [#4380](https://github.com/Genymobile/scrcpy/issues/4380), PR
  [#5102](https://github.com/Genymobile/scrcpy/pull/5102)).
- Sotto, la policy crea **un altro dispositivo remote submix** con indirizzo =
  id della registrazione, e i lettori che corrispondono alla regola vengono
  spostati su quell'uscita [V]
  ([AudioPolicyManager.cpp](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/services/audiopolicy/managerdefault/AudioPolicyManager.cpp),
  `registerPolicyMixes`, `getOutputForAttr`). Quindi **stesso HAL** di «output».
- Le app possono rifiutare la cattura (`allowAudioPlaybackCapture`,
  `setAllowedCapturePolicy`; le app con targetSdk ≤ 28 sono escluse per
  predefinito) [V] ([Capture audio playback](https://developer.android.com/media/platform/av-capture)).
  Il sistema può catturare `ALLOW_CAPTURE_BY_SYSTEM`; la shell **non** è sistema
  in questo senso [I].
- Pro rispetto a REMOTE_SUBMIX (yume-chan in #4380): si può lasciare suonare il
  telefono (`ROUTE_FLAG_LOOP_BACK_RENDER`), non dipende dalle cuffie collegate,
  non dipende dal volume, si può catturare **per singola app** (regola per UID).
- Difetti di scrcpy 4.1 [V, dal codice]:
  - cattura **solo `USAGE_MEDIA`**: l'audio dei giochi (`USAGE_GAME`,
    Unity/Unreal/FMOD) e `USAGE_UNKNOWN` non viene catturato **e resta
    sull'altoparlante del telefono**. Corretto nel ramo `dev` dal commit
    [e71cdb8](https://github.com/Genymobile/scrcpy/commit/e71cdb8782f524d8ec4deaa1b1e4930c728acab0)
    (PR #6975, issue [#7038](https://github.com/Genymobile/scrcpy/issues/7038)),
    che aggiunge 13 usi;
  - `voiceCommunicationCaptureAllowed(true)` è chiamato sul *builder* **dopo**
    `build()`: non ha effetto. rom1v stesso nota in #4380 che la voce delle
    chiamate non viene catturata. Con la chiamata nell'ordine giusto e il
    permesso `CAPTURE_VOICE_COMMUNICATION_OUTPUT` (che la shell ha) la voce VoIP
    dovrebbe essere catturabile [I: sidharthv96 in #4380 riferisce che «capturing
    in call audio works (at least in WhatsApp calls)»]. Rilevante per il limite
    «le chiamate restano sul telefono» di SPECIFICHE §10/§12.
- Effetto misurato [M]: 60/s nei video, ma **vuoti di 50–120 ms, ~1,5 al secondo**
  nei reel di Facebook e in Chrome (tagli netti −21 → −45 dB in 1 ms).

### 1.4 Perché nel remote submix i lettori possono andare in underrun

Codice dell'HAL remote submix, sia la versione AIDL di Android 14+
([StreamRemoteSubmix.cpp](https://cs.android.com/android/platform/superproject/main/+/main:hardware/interfaces/audio/aidl/default/r_submix/StreamRemoteSubmix.cpp),
[SubmixRoute.h](https://cs.android.com/android/platform/superproject/main/+/main:hardware/interfaces/audio/aidl/default/r_submix/SubmixRoute.h))
sia quella vecchia
([audio_hw.cpp](https://cs.android.com/android/platform/superproject/main/+/main:hardware/libhardware/modules/audio_remote_submix/audio_hw.cpp)) [V]:

- tra uscita (mixer) e ingresso (cattura) c'è un tubo `MonoPipe` di
  **4 × 1024 = 4096 campioni (~85 ms)**;
- **chi legge** è cadenzato dall'orologio di sistema: `inRead` aspetta al massimo
  la durata del blocco, poi **riempie di zeri** quello che manca («*in any case,
  it is emulated that data for the entire buffer was available*»: `memset(buffer, 0, …)`).
  Nella versione vecchia: 3 tentativi da 5 ms, poi zeri;
- **chi scrive** (il thread mixer di AudioFlinger) quando c'è un lettore attivo è
  bloccante (`shouldBlockWrite()`), e il blocco è simulato con uno *sleep*
  regolato dal riempimento del tubo
  ([MonoPipe.cpp](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/media/libnbaio/MonoPipe.cpp)):
  tubo quasi vuoto → dorme **metà** del tempo del blocco (scrive a 2×); sopra il
  punto di lavoro → dorme 1,15×, 1,35×, 1,75×.

Conseguenze [I, da verificare con le prove in fondo]:

- il mixer, e quindi il consumo dei buffer degli `AudioTrack` delle app, **non
  procede a velocità costante**: a tratti consuma al doppio della velocità reale.
  Un lettore che ha dimensionato il suo buffer per l'uscita normale (in
  particolare i lettori su uscita *deep buffer*, con periodi lunghi e riempimenti
  "pigri" per risparmiare batteria) può svuotarsi → **underrun → vuoto netto**
  nel mix (i 50–120 ms del playback);
- i lettori video sincronizzano l'immagine sulla posizione dell'audio
  (`AudioTrack.getTimestamp`): un orologio audio che accelera e rallenta porta a
  **scartare fotogrammi** (il 24/s di «output»);
- se il mixer ritarda (CPU occupata, app che parte), il lettore della cattura
  trova il tubo vuoto e riceve **zeri**: micro-interruzioni **già nel PCM
  catturato**, quindi anche nel file compresso (quello che si sente con VLC).

Perché le due modalità si comportano diversamente pur usando lo stesso HAL non è
spiegato dal codice letto: differenze nel dimensionamento delle uscite e nel
ricrearsi degli `AudioTrack` quando vengono spostati di uscita (invalidazione e
`restoreTrack`) [I]. Va misurato, non dedotto.

### 1.5 La variante con orologio hardware: `ROUTE_FLAG_LOOP_BACK_RENDER`

Con `LOOP_BACK_RENDER` il lettore resta sull'uscita vera (altoparlante) e una
copia del suo flusso va al submix (uscite secondarie, "tee patch" in
AudioFlinger) [V: esistenza del flag e uso in scrcpy `--audio-dup`; I: dettagli].
Il ritmo del mixer lo detta l'**orologio hardware**: nessun underrun indotto dal
tubo. È molto probabilmente ciò che fa il registratore dello schermo Samsung,
che suona anche dal telefono e ha dato audio perfetto [I].

Problema: **il telefono suona** (e con il volume al massimo, deciso per
Facebook). Idee da provare [I]:
- la copia verso il submix è presa prima del volume? Se sì, si può tenere il
  volume multimediale a 1/15 (Facebook avvia l'audio con volume > 0, §41) e
  il telefono sussurra appena. Non soddisfa «il telefono non suona»: va proposto
  all'utente solo se le misure lo giustificano;
- instradare la parte "render" su un'uscita muta con
  `AudioManager.setPreferredDeviceForStrategy` (la shell ha
  `MODIFY_AUDIO_ROUTING`): non ho trovato un dispositivo "muto" standard;
- il *master mute* (`AudioSystem.setMasterMute`) viene applicato anche ai thread
  del submix salvo HAL che lo gestiscano da sé
  ([Threads.cpp](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/services/audioflinger/Threads.cpp),
  `setMasterMute_l`): probabilmente zittirebbe anche la cattura [I].

## 2. `AudioRecord` dalla shell

- **Contesto / attribuzione**: da API 31 `AudioRecord.Builder.setContext()`;
  scrcpy passa un `FakeContext` con pacchetto `com.android.shell` e uid 2000
  (così `AttributionSource` corrisponde al chiamante e il controllo dei
  permessi passa) [V]
  ([AudioDirectCapture.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/audio/AudioDirectCapture.java)).
  Il nostro `Audio.java` fa lo stesso per riflessione. Su Android 11 serviva
  un'attività della shell in primo piano; da 12 non più (doc scrcpy) [V]. Nessuna
  restrizione nuova nota su 14–16 per un processo non-app [I].
- **Formato**: PCM 16 bit, 48 kHz, stereo; il remote submix lavora a 48 kHz
  16 bit stereo per predefinito (`kDefaultSampleRateHz = 48000`) [V]: niente
  ricampionamento.
- **Dimensione del blocco**: scrcpy legge al massimo 1024 campioni perché «il
  sistema cattura a blocchi di 1024» (commento in
  [AudioConfig.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/audio/AudioConfig.java)),
  coerente con il periodo del tubo (4096/4) [V]. Leggere 1024 = 21,33 ms.
- **Buffer interno**: scrcpy `8 × getMinBufferSize()`; `Audio.java` `max(4×min,
  0,5 s)`. Il buffer grande non aumenta la latenza se si legge appena i dati
  arrivano; riduce il rischio di **overrun** (dati persi se il nostro thread si
  ferma) [V: commento scrcpy «This buffer size does not impact latency»].
- **Lettura bloccante** (`READ_BLOCKING`, predefinita per gli array): la più
  semplice e corretta per un thread dedicato. La non bloccante serve solo con un
  unico thread che fa altro; non è il nostro caso [V: documentazione
  [AudioRecord.read](https://developer.android.com/reference/android/media/AudioRecord#read(byte[],%20int,%20int,%20int))].
- **Orari**: `AudioRecord.getTimestamp(ts, TIMEBASE_MONOTONIC)` dà una coppia
  (`framePosition`, `nanoTime`): il campione numero `framePosition` è stato
  catturato all'istante `nanoTime` [V]
  ([AudioTimestamp](https://developer.android.com/reference/android/media/AudioTimestamp)).
  L'orario giusto dell'inizio di un blocco che parte dal campione N è
  `nanoTime + (N − framePosition) / 48000`. **scrcpy usa `nanoTime` così com'è**
  come orario del blocco appena letto, ignorando `framePosition`
  ([AudioRecordReader.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/audio/AudioRecordReader.java)):
  l'orario si aggiorna a scatti → pts irregolari [V].
  Raccomandazione: orari **dal conteggio dei campioni** (già così in
  `Audio.java`); leggere `getTimestamp` ogni ~1 s solo per **misurare la deriva**
  tra orologio audio e `CLOCK_MONOTONIC` e per scoprire salti (campioni persi),
  mandandola al PC come diagnostica.
- **Overrun**: in Java non c'è un contatore; si vede come salto di
  `framePosition` rispetto ai campioni letti, e in `dumpsys media.audio_flinger`
  (thread di registrazione, colonna overrun) [V/I].
- **Priorità**: `Process.setThreadPriority(THREAD_PRIORITY_URGENT_AUDIO)` (−19)
  sul thread di lettura. Linux richiede `RLIMIT_NICE` per i valori negativi;
  `init` di Android lo imposta a 40 per tutti i processi, ereditato da `adbd` e
  quindi dalla shell [I: da verificare leggendo `/proc/<pid>/task/<tid>/stat`
  dopo la chiamata]. `SCHED_FIFO` non è disponibile alla shell [I].
- **Silenziamento dalla policy**: con più catture contemporanee Android può
  consegnare **silenzio** a una di esse (regole di condivisione dell'ingresso,
  [Sharing audio input](https://developer.android.com/media/platform/sharing-audio-input));
  `AudioManager.AudioRecordingCallback` / `AudioRecordingConfiguration.isClientSilenced()`
  lo segnala [V]. Da registrare nel componente come diagnostica.

## 3. Compressione

### 3.1 Codificatori disponibili [V per AOSP, I per Samsung]

- AOSP fornisce codificatori software Codec2: `c2.android.aac.encoder` (FDK
  AAC), `c2.android.opus.encoder` (libopus), `c2.android.flac.encoder`,
  `c2.android.amrnb/amrwb.encoder`
  ([C2SoftAacEnc.cpp](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/media/codec2/components/aac/C2SoftAacEnc.cpp),
  [C2SoftOpusEnc.cpp](https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/media/codec2/components/opus/C2SoftOpusEnc.cpp)).
- Sui Samsung con Snapdragon di solito non c'è un codificatore AAC hardware
  esposto a `MediaCodec`; possono esserci codificatori `c2.sec.*` [I]. Da
  elencare sul telefono (prova A1). Un codificatore software a 48 kHz stereo
  costa pochissima CPU su un S23+: non è un problema di prestazioni.
- `C2SoftAacEnc` supporta anche **AAC-LD / AAC-ELD** (bassa latenza) [V]
  (`getAOTFromProfile`: `AOT_ER_AAC_LD`, `AOT_ER_AAC_ELD`), decodificabili sul PC
  da libfdk-aac / FFmpeg [I per GStreamer].

### 3.2 Opus di Android e scrcpy: cosa succede davvero [V]

- `C2SoftOpusEnc` usa trame da **20 ms (960 campioni)**, complessità 10,
  `OPUS_APPLICATION_AUDIO`, DTX spento. I blocchi di 1024 campioni sono
  accumulati (`mFilledLen`) e codificati a trame intere: **nessun campione
  perso o duplicato**. Un blocco da 1024 produce 1 pacchetto, ogni 15 blocchi 2.
- Gli orari d'uscita sono **ricalcolati dal numero di campioni** (`mAnchorTimeStamp
  + mProcessedSamples/…`): regolari, ma ignorano gli orari d'ingresso.
- scrcpy, per aggirare questo, **sovrascrive** l'orario di ogni pacchetto Opus
  con `System.nanoTime()/1000 − durata` **nel momento in cui il pacchetto esce
  dal codificatore** (`fixTimestamp`, `recreatePts` in
  [AudioEncoder.java](https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/audio/AudioEncoder.java)).
  Il codificatore lavora a scatti (callback asincrone, 2 pacchetti per volta
  ogni 15 blocchi, scheduling) → **pts a raffica (1–3 ms / 30–40 ms)**:
  è esattamente ciò che abbiamo misurato [M]. rom1v lo ammette in
  [#6929](https://github.com/Genymobile/scrcpy/issues/6929): «the Android opus
  encoder mux timestamps have to be "overwritten". You might get better results
  with aac».
- Il buffer d'ingresso del codificatore Opus è `max-input-size = 3840` byte
  (960 campioni) nei log di [#5482](https://github.com/Genymobile/scrcpy/issues/5482),
  mentre scrcpy chiede letture fino a 4096 byte: con i buffer Codec2 reali più
  capienti funziona, ma è un punto fragile [I].
- Conclusione: le micro-interruzioni **dentro** il file Opus non vengono dalla
  codifica (che conserva ogni campione) ma da ciò che entra: zeri del remote
  submix o underrun del mixer (§1.4) [I forte].

### 3.3 Confronto per Phonestra

| Formato | Banda | Latenza aggiunta (circa) | Note |
|---|---|---|---|
| PCM 16 bit 48 kHz stereo | 1,536 Mbit/s (192 kB/s) | 0 (solo il blocco di 21 ms) | Nessun codificatore; perfetto per diagnosi; il Wi-Fi misurato (288 Mbit/s, §31) lo regge |
| AAC-LC 192 kbit/s | 0,19 Mbit/s | ~40–60 ms (trama 1024 + ritardo del codificatore/decodificatore) | Quello del registratore Samsung; orari d'ingresso conservati |
| AAC-ELD | ~0,1–0,2 Mbit/s | ~15–25 ms | Da provare sul PC (decodifica) |
| Opus 128 kbit/s | 0,13 Mbit/s | ~26 ms (20 + 6,5 lookahead) | Orari ricalcolati dal codificatore |

[I per le latenze: valori tipici dei formati, non misurati sul telefono.]

**Raccomandazione**: il componente parte in **PCM**, con un'opzione AAC-LC. Il PCM
elimina un'intera classe di dubbi (codificatore, orari, riuso dei buffer) e
rende l'analisi dei vuoti banale: una sequenza di campioni esattamente a zero in
mezzo al suono = zeri messi dal submix. Se in uso reale il PCM va bene, si può
tenere; la registrazione MP4 "senza ricodifica" (SPECIFICHE §13) richiederebbe
allora di codificare sul PC.

Nel `Audio.java` attuale: un unico thread legge e poi aspetta il codificatore
(`dequeueInputBuffer(-1)`): se il codificatore tarda, la lettura si ferma. Con
0,5 s di buffer non dovrebbe perdere dati, ma conviene separare lettura e
codifica (o passare a PCM) e misurare [I].

## 4. Come lo fa scrcpy 4.x e cosa si sa dalle issue

Codice: [server/…/audio/](https://github.com/Genymobile/scrcpy/tree/master/server/src/main/java/com/genymobile/scrcpy/audio) [V]

- `AudioDirectCapture`: `AudioRecord` con la sorgente scelta (`REMOTE_SUBMIX`
  per «output»), `setContext(FakeContext)`, buffer 8× il minimo; su Vivo il
  Builder fallisce e c'è un ripiego (#3805).
- `AudioPlaybackCapture`: `AudioPolicy` + `AudioMix` `LOOP_BACK` o
  `LOOP_BACK_RENDER` (`--audio-dup`), solo `USAGE_MEDIA` in 4.1.
- `AudioRecordReader`: legge ≤ 1024 campioni; pts = `AudioTimestamp.nanoTime` o
  stima; forza la monotonia.
- `AudioEncoder`: `MediaCodec` asincrono con due thread (in/out); per Opus e FLAC
  ricrea i pts dall'orologio all'uscita.
- `AudioRawRecorder`: PCM senza codifica (`--audio-codec=raw`).

Issue rilevanti:
- [#3793](https://github.com/Genymobile/scrcpy/issues/3793) «robotic and glitchy
  sound» e [#4055](https://github.com/Genymobile/scrcpy/issues/4055) «crackling»:
  **lato PC** (SDL chiede campioni ogni 10 ms invece di 5); il file registrato
  era perfetto. Risolto con `--audio-output-buffer`, poi predefinito a 10 ms
  (#6775). Non riguarda noi (GStreamer/PipeWire), ma insegna il metodo di rom1v:
  **prima si ascolta il file registrato** per separare telefono e PC.
- [#5925](https://github.com/Genymobile/scrcpy/issues/5925): tagli e intonazione
  variabile con PC sotto carico: lato PC; file registrato perfetto.
- [#6015](https://github.com/Genymobile/scrcpy/issues/6015): audio a scatti con
  display virtuale, **anche nel file MP4** → lato telefono. Senza risposta.
  Simile al nostro caso (Phonestra usa display virtuali).
- [#6907](https://github.com/Genymobile/scrcpy/issues/6907) (Galaxy S25+,
  Android 16): glitch con audio **e** video nello stesso processo; spariscono
  con due processi separati (uno solo video, uno solo audio `output` AAC).
  Senza risposta. Suggerisce contesa tra video e audio **nello stesso processo**
  (thread, GC, priorità) [I] → nel nostro componente l'audio va su thread
  prioritario, meglio ancora in un processo separato.
- [#6929](https://github.com/Genymobile/scrcpy/issues/6929): file registrati con
  audio «rotto» in VLC/DaVinci; rom1v: orari Opus sovrascritti, «meglio aac».
- [#4087](https://github.com/Genymobile/scrcpy/issues/4087),
  [#5281](https://github.com/Genymobile/scrcpy/issues/5281): con il microfono
  in uso (chat vocale) l'audio non arriva più al PC con «output»; con
  «playback» sì, ma non la voce degli altri.
- [#5781](https://github.com/Genymobile/scrcpy/issues/5781),
  [#7038](https://github.com/Genymobile/scrcpy/issues/7038): giochi muti con
  «playback» → risolto nel ramo `dev` catturando tutti gli usi.
- Non ho trovato issue su scrcpy che descrivano i **vuoti dei lettori** con
  «playback» né il **calo dei fotogrammi** con «output»: sono da considerare
  scoperte nostre, non problemi noti e risolti.

## 5. Microfono e fotocamera del telefono per il PC

**Nota**: SPECIFICHE §12 — «Tolto (27 set 2026, decisione dell'utente) […] Non va
riproposto senza una richiesta dell'utente». Qui solo la fattibilità tecnica.

- **Microfono** [V]: la shell ha `RECORD_AUDIO` e `RECORD_BACKGROUND_AUDIO`;
  scrcpy cattura `MIC`, `VOICE_COMMUNICATION` (con cancellazione dell'eco del
  telefono), `UNPROCESSED`, ecc. da Android 11/12 senza trucchi.
  Limiti: regole di condivisione dell'ingresso (se un'app in primo piano usa il
  microfono con sorgente "sensibile", noi riceviamo **silenzio**
  [V, Sharing audio input]); interruttore privacy «Accesso al microfono» →
  silenzio; indicatore verde della privacy visibile [I].
- **Fotocamera** [V]: la shell ha `CAMERA`, `BACKGROUND_CAMERA`,
  `SYSTEM_CAMERA`; scrcpy la usa con camera2 da Android 12 (`--video-source=camera`,
  [camera.md](https://github.com/Genymobile/scrcpy/blob/master/doc/camera.md)).
  Limiti [I]: un'app in primo piano che apre la fotocamera ci toglie l'accesso
  (priorità del CameraService); interruttore privacy «Accesso alla fotocamera»;
  risoluzioni dichiarate non sempre reali (lo dice la doc di scrcpy).
- **Sul PC**: sorgenti PipeWire (audio) e, per il video, nodo PipeWire
  `Video/Source` (visto dai browser e dalle app via portale fotocamera) o
  `v4l2loopback` (modulo del kernel, serve root: contrasta con l'AppImage senza
  installazione) [I].

## 6. Rischi, differenze tra produttori, cose da non dimenticare

- **Samsung ha un motore di policy e HAL suoi**: il comportamento di §1.2
  (cosa va al submix, cosa all'altoparlante) è quello di AOSP; su One UI può
  differire (Separate app sound, Dolby Atmos, Adapt Sound agiscono sull'uscita
  vera, non sul submix) [I].
- Quale implementazione del remote submix gira sul S23+ (AIDL di Android 14+ o
  la vecchia di libhardware) cambia i dettagli (zeri dopo la scadenza o dopo
  3×5 ms), non il principio [V/I].
- **Volume**: con REMOTE_SUBMIX l'audio catturato segue il volume multimediale
  (per questo va al massimo); con AudioPolicy no [I da yume-chan #4380]. Se si
  passa ad AudioPolicy, la regola «volume al massimo» resta utile solo per
  Facebook (che guarda il volume).
- **Suonerie e sveglie** con REMOTE_SUBMIX suonano dal telefono e non arrivano al
  PC [V AOSP].
- **Più catture insieme**: registratore Samsung, Assistente o chiamata attivi
  possono farci ricevere silenzio [V].
- **Chiamate**: con `setMode(IN_COMMUNICATION)` l'instradamento cambia per tutti i
  media (#4087).
- **Cuffie/Bluetooth collegati**: con REMOTE_SUBMIX la policy preferisce il
  submix per i media; con AudioPolicy la regola vince sul dispositivo [I].
- **Contesa di CPU tra video e audio** nello stesso processo (#6907) [I].
- **App che vietano la cattura** (solo AudioPolicy): non arrivano al PC e, con
  LOOP_BACK, **restano sull'altoparlante del telefono** [I da verificare].
- Android 15/16: nessun cambiamento documentato che tocchi `REMOTE_SUBMIX` o
  `AudioPolicy` per la shell; scrcpy 4.x funziona su 16 (#6907, #6929) [V per il
  funzionamento, I per l'assenza di restrizioni future].

## Domande aperte

1. Il registratore Samsung usa davvero `LOOP_BACK_RENDER` (o un percorso
   proprietario)? Si vede con `dumpsys media.audio_policy` durante una
   registrazione (mix registrati, flag di instradamento).
2. La copia di `LOOP_BACK_RENDER` è presa prima o dopo il volume? Se prima, è
   accettabile per l'utente un telefono a volume 1/15 durante il collegamento?
   (Va chiesto all'utente solo dopo la misura.)
3. I vuoti di «playback» sono underrun dei lettori (visibili in AudioFlinger) o
   zeri del submix? E le micro-interruzioni di «output»?
4. Il calo a 24/s con «output» si ripete con il nostro componente (PCM, thread
   prioritario), o dipendeva da scrcpy?
5. Con AudioPolicy e `voiceCommunicationCaptureAllowed` nell'ordine giusto la
   voce di WhatsApp arriva al PC? (Riguarda il rinvio di SPECIFICHE §12.)
6. Le app che vietano la cattura arrivano al PC con REMOTE_SUBMIX? (SPECIFICHE
   §10, «da verificare».)
7. La shell riesce davvero a mettersi a −19 (`THREAD_PRIORITY_URGENT_AUDIO`)?
8. Il PCM a 1,5 Mbit/s su ADB Wi-Fi regge nei primi 20 s dopo l'apertura di
   un'app, quando il telefono è sotto sforzo (§31)?

## Prove da fare sul telefono

Tutte con `phonestra-prova` quando basta; `adb` di sistema solo per `dumpsys`;
a fine prova chiudere il server adb, ripristinare volume e spegnimento.

- **A1 – Inventario**: elencare i codificatori audio (`MediaCodecList`: nome,
  hardware/software, alias), `dumpsys package com.android.shell | grep
  permission` (permessi audio/fotocamera concessi), quale HAL remote submix è
  caricato (`dumpsys media.audio_flinger`, sezione moduli; `service list | grep
  audio`).
- **A2 – Tre sorgenti a confronto, in PCM**: componente con sorgente
  selezionabile (REMOTE_SUBMIX / LOOP_BACK / LOOP_BACK_RENDER, tutti gli usi),
  PCM, orari dal conteggio. Per ciascuna: reel di Facebook 60 s, video in
  Chrome 60 s, YouTube 60 s. Misure:
  1. **zeri digitali** nel PCM ricevuto (sequenze ≥ 48 campioni esattamente a 0
     in mezzo al suono) e cadute di livello > 20 dB in 1 ms (lo stesso metodo del
     28 set);
  2. **underrun dei lettori**: `dumpsys media.audio_flinger` prima e dopo (colonna
     underrun/`UndFrmCnt` delle tracce, overrun del thread di registrazione);
  3. **fotogrammi/s** del video (banco di prova «banco»);
  4. deriva `AudioTimestamp` vs campioni letti (salti = dati persi).
- **A3 – Samsung**: avviare il registratore dello schermo con «suoni
  multimediali» e leggere `dumpsys media.audio_policy` (mix dinamici,
  `ROUTE_FLAG`) e `dumpsys media.audio_flinger` (uscite duplicate/tee).
- **A4 – Volume e LOOP_BACK_RENDER**: con volume 1/15 e 15/15, confrontare il
  livello del PCM catturato: se uguale, la copia è prima del volume.
- **A5 – AAC contro PCM**: la sorgente migliore di A2, codificata AAC-LC 192
  kbit/s: stessi contenuti, stesse misure; più il file riascoltato con VLC.
- **A6 – Priorità**: dopo `setThreadPriority(-19)` leggere il nice del thread in
  `/proc/<pid>/task/<tid>/stat`; ripetere A2 con il telefono sotto carico
  (apertura di app pesanti) con e senza priorità.
- **A7 – Casi limite**: notifica, suoneria, sveglia (dove suonano, arrivano al
  PC?); chiamata WhatsApp (voce al PC con AudioPolicy + voce VoIP permessa?);
  un gioco (USAGE_GAME); un'app che vieta la cattura; cuffie Bluetooth
  collegate; registratore Samsung attivo in contemporanea (silenzio?).
- **A8 – Contesa video/audio**: A2 con l'audio nel processo del video e in un
  processo separato (#6907).

## Fonti principali

- scrcpy, codice audio: <https://github.com/Genymobile/scrcpy/tree/master/server/src/main/java/com/genymobile/scrcpy/audio>
- scrcpy, documentazione: <https://github.com/Genymobile/scrcpy/blob/master/doc/audio.md>, <https://github.com/Genymobile/scrcpy/blob/master/doc/camera.md>
- scrcpy, issue: #3793, #4055, #4087, #4380, #5102, #5281, #5482, #5781, #5925, #6015, #6907, #6929, #7038; commit e71cdb8
- AOSP remote submix (AIDL): <https://cs.android.com/android/platform/superproject/main/+/main:hardware/interfaces/audio/aidl/default/r_submix/>
- AOSP remote submix (vecchio): <https://cs.android.com/android/platform/superproject/main/+/main:hardware/libhardware/modules/audio_remote_submix/audio_hw.cpp>
- MonoPipe: <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/media/libnbaio/MonoPipe.cpp>
- Motore di policy: <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/services/audiopolicy/enginedefault/src/Engine.cpp>
- AudioPolicyManager: <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/services/audiopolicy/managerdefault/AudioPolicyManager.cpp>
- Codificatori Codec2: <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/media/codec2/components/>
- Manifesto della shell: <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/packages/Shell/AndroidManifest.xml>
- Android Developers: [Capture audio playback](https://developer.android.com/media/platform/av-capture), [Sharing audio input](https://developer.android.com/media/platform/sharing-audio-input), [AudioTimestamp](https://developer.android.com/reference/android/media/AudioTimestamp), [AudioRecord](https://developer.android.com/reference/android/media/AudioRecord), [MediaRecorder.AudioSource](https://developer.android.com/reference/android/media/MediaRecorder.AudioSource)
