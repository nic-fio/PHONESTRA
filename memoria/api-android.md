# API di Android per il componente nostro

Il componente per il telefono (`telefono/aiuto`) sostituisce scrcpy un pezzo
alla volta (decisione del 28 set 2026, `decisioni-utente.md`). Qui, **prima di
scrivere il codice di ogni pezzo**, le API scelte, i permessi della shell che
servono e le differenze tra Android 14, 15 e 16. Solo Android 14+ (API 34+).

Il componente gira come la shell (uid 2000, `app_process`), non è un'app
installata: niente finestre di consenso, ma solo i permessi che la shell ha.

## 1. Audio

> **Scelta dopo le misure del 28 set (prove-collegamento §42)**: sorgente
> **loopback** (AudioPolicy con `ROUTE_FLAG_LOOP_BACK`), **PCM** non compresso,
> orari dal conteggio dei campioni, thread a priorità −19. File «perfetto»
> all'ascolto, 0 zeri e 0 tagli in 60 s. Il testo sotto è la bozza iniziale
> (REMOTE_SUBMIX + AAC), superata; il §1.1 descrive lo strumento di misura.


> Lo studio del 28 set (`studio/audio.md`) mostra che anche REMOTE_SUBMIX passa
> dal *remote submix*, senza orologio vero: la sorgente si sceglie solo dopo il
> confronto in PCM delle tre sorgenti (REMOTE_SUBMIX, LOOP_BACK,
> LOOP_BACK_RENDER). Sintesi e ordine di lavoro in `studio/README.md`. La bozza
> `telefono/aiuto/src/phonestra/Audio.java` serve da base per quelle misure.

- **Sorgente**: `AudioRecord` con `MediaRecorder.AudioSource.REMOTE_SUBMIX`
  (l'uscita intera del telefono, dopo il mixer). Richiede
  `CAPTURE_AUDIO_OUTPUT`, che la shell ha. Mentre la cattura è attiva Android
  non manda l'audio agli altoparlanti (è quello che vogliamo).
  Scartata la cattura per singola app (`AudioPlaybackCaptureConfiguration`):
  nelle prove del 28 set lasciava vuoti di 50–120 ms nei lettori di Facebook e
  Chrome (§41–42 delle prove).
- **Contesto**: `AudioRecord.Builder.setContext()` (API 31) con un contesto del
  pacchetto `com.android.shell`, così l'attribuzione del permesso corrisponde
  all'uid della shell.
- **Formato**: PCM 16 bit, 48 kHz, stereo; letto a blocchi di 1024 campioni.
- **Compressione**: `MediaCodec` AAC-LC (`audio/mp4a-latm`), 192 kbit/s: la
  stessa scelta del registratore dello schermo Samsung, che nelle prove ha dato
  audio senza interruzioni. Opus (quello di scrcpy) è un codificatore software.
- **Orari**: dal numero di campioni letti, non dall'orologio: regolari per
  costruzione (gli orari irregolari di scrcpy causavano micro-interruzioni).
- **Trasporto**: sull'uscita del processo (`exec:`), pacchetti con 8 byte di
  orario e bandiere (bit 62 = configurazione) e 4 byte di lunghezza, come quelli
  che Phonestra già legge. Lettura e spedizione su due thread: il Wi-Fi lento
  non ferma la cattura.

### 1.1 Strumento di misura (fase 0, 28 set 2026)

`Audio.java` è diventato lo strumento per le prove A1, A2, A4, A5, A6 di
`studio/audio.md`: comando dell'aiutante
`audio sorgente=submix|loopback|render formato=pcm|aac priorita=si|no voce=si|no`,
dal PC `phonestra-prova audio-nostro <secondi> [sorgente] [formato]
[senza-priorita] [voce]`. Legenda come negli studi: [V] verificato su codice o
documentazione, [I] ipotesi da verificare sul telefono (la versione scritta il
28 set **non è ancora stata provata**).

- **`submix`**: come sopra (`AudioRecord`, `REMOTE_SUBMIX`, `setContext` col
  contesto della shell, buffer di 0,5 s).
- **`loopback` / `render`**: API di sistema nascoste di
  `android.media.audiopolicy`, tutte per riflessione (nessuno stub) [I per le
  firme: prese dal codice AOSP e dall'uso che ne fa scrcpy, riscritte]:
  - `AudioMixingRule.Builder()` → `setTargetMixRole(MIX_ROLE_PLAYERS)` (prima
    delle regole, che il ruolo rende valide o no) → `addMixRule(RULE_MATCH_ATTRIBUTE_USAGE,
    AudioAttributes)` una volta per uso → `build()`;
  - usi catturati: `UNKNOWN, MEDIA, GAME, ASSISTANT, ASSISTANCE_ACCESSIBILITY,
    ASSISTANCE_NAVIGATION_GUIDANCE, ASSISTANCE_SONIFICATION`, cioè quelli che il
    motore AOSP manda al submix anche con `REMOTE_SUBMIX` (strategie MEDIA e
    ACCESSIBILITY): le tre sorgenti si confrontano sugli stessi suoni. Suonerie,
    sveglie, notifiche e chiamate restano sul telefono. Con `voce=si` si
    aggiunge `VOICE_COMMUNICATION` e si chiama
    `voiceCommunicationCaptureAllowed(true)` **prima** di `build()` (prova A7);
  - `AudioMix.Builder(regola)` → `setFormat(PCM 16 bit, 48 kHz, stereo)` →
    `setRouteFlags(ROUTE_FLAG_LOOP_BACK)` (il telefono tace) oppure
    `ROUTE_FLAG_LOOP_BACK_RENDER` (il telefono continua a suonare) → `build()`;
  - `AudioPolicy.Builder(contesto della shell)` → `addMix(mix)` → `build()`;
  - registrazione: prima `AudioManager.registerAudioPolicy(politica)` (metodo
    di sistema pubblico, sull'`AudioManager` di `getSystemService("audio")`),
    se fallisce `AudioManager.registerAudioPolicyStatic` (privato, statico); la
    riga `inizio` dice quale ha funzionato (`registrazione=istanza|statica`).
    0 = riuscita. Permesso: `MODIFY_AUDIO_ROUTING` (shell, da Android 13) [V];
  - `AudioPolicy.createAudioRecordSink(mix)` → l'`AudioRecord` da leggere. Il
    suo buffer lo decide Android (circa il minimo, non i nostri 0,5 s) [I]: la
    riga `inizio` lo riporta (`buffer_ms`), perché con un buffer piccolo il
    rischio di perdere dati è più alto;
  - alla fine `unregisterAudioPolicy` (o `unregisterAudioPolicyAsyncStatic`);
    se il processo muore, il sistema toglie comunque la politica (binder morto) [I].
  - Nota [I]: AOSP marca la cattura del mix col tag `SUBMIX_FIXED_VOLUME`:
    potrebbe voler dire che la copia è presa a volume fisso, prima del volume
    del telefono. È quello che la prova A4 (volume 1/15 contro 15/15) misura.
- **Formato**: `pcm` = i campioni così come letti (little-endian), `aac` =
  AAC-LC 192 kbit/s come la bozza. In AAC la codifica ha ora un thread suo:
  la lettura non aspetta più il codificatore.
- **Thread di lettura**: `android.os.Process.setThreadPriority(THREAD_PRIORITY_URGENT_AUDIO)`
  (−19) salvo `priorita=no`; il valore effettivo si legge dal campo 19 («nice»)
  di `/proc/self/task/<tid>/stat` (`Process.myTid()`). Se Android rifiuta la
  priorità arriva una riga `avviso`.
- **Misure** (pacchetto con il **bit 61** alzato, contenuto testo UTF-8,
  `tipo chiave=valore …`; tipi `inizio`, `lettura`, `misura`, `avviso`,
  `errore`). Una `misura` al secondo di audio letto, valori cumulativi:
  `campioni`, `letture` (chiamate a `read`), `brevi` (read con meno byte del
  chiesto), `lettura_max_ms` (read più lunga nell'ultimo secondo), `zeri`,
  `zeri_ms`, `zeri_max_ms` (sequenze ≥ 48 campioni a 0 su entrambi i canali,
  dopo un livello medio di ~10 ms sopra −40 dBFS e finite dal ritorno del
  suono), `persi` (pacchetti scartati perché la coda di ~5 s era piena),
  `posizione`, `deriva_ms`, `attesa_ms`, `nice`.
- **Deriva**: `AudioRecord.getTimestamp(ts, AudioTimestamp.TIMEBASE_MONOTONIC)`
  (API 24) dà la coppia (`framePosition`, `nanoTime`). `deriva_ms` = tempo
  secondo i campioni catturati meno tempo di `CLOCK_MONOTONIC`, dal primo
  orario valido (orologio audio più lento o più veloce, salti = dati persi o
  inventati); `attesa_ms` = campioni catturati (stimati adesso) meno campioni
  letti da noi: se cresce senza fermarsi o supera `buffer_ms`, si perdono dati.
  Senza orario: `orario=assente`.
- **Errori**: l'uscita d'errore del processo non arriva al PC (`2>/dev/null`),
  quindi ogni eccezione arriva anche come riga `errore …`.
- **Codificatori** (prova A1, e 14 del video): comando dell'aiutante
  `codificatori`, dal PC `phonestra-prova codificatori`.
  `new MediaCodecList(REGULAR_CODECS).getCodecInfos()`, solo i codificatori
  (`isEncoder`), per ogni tipo audio/video: nome, `isHardwareAccelerated` /
  `isSoftwareOnly`, `isVendor`, `isAlias` / `getCanonicalName` (API 29), e da
  `getCapabilitiesForType`: istanze, profili (`profileLevels`; per AAC col nome:
  2 LC, 5 HE, 29 HEv2, 23 LD, 39 ELD), modi di bitrate
  (`EncoderCapabilities.isBitrateModeSupported`), per l'audio bitrate, canali e
  frequenze, per il video allineamento, larghezze, altezze, fps, bitrate. Tutte
  API pubbliche [V].
- **Analisi sul PC** (`src/misura_audio.rs`), sul PCM ricevuto: le stesse
  sequenze di zeri (livello prima = RMS dei 10 ms precedenti) e i **tagli
  netti**: livello a finestre di 1 ms che passa da sopra −30 dB a sotto −50 dB
  in 2 ms e resta sotto −50 dB per almeno 30 ms. Livelli in dBFS come RMS
  riferito a 32768.

## 2. Video e finestre delle app (da studiare)

## 3. Tocchi, tasti, appunti, comandi (da studiare)
