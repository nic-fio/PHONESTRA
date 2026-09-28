# API di Android per il componente nostro

Il componente per il telefono (`telefono/aiuto`) sostituisce scrcpy un pezzo
alla volta (decisione del 28 set 2026, `decisioni-utente.md`). Qui, **prima di
scrivere il codice di ogni pezzo**, le API scelte, i permessi della shell che
servono e le differenze tra Android 14, 15 e 16. Solo Android 14+ (API 34+).

Il componente gira come la shell (uid 2000, `app_process`), non è un'app
installata: niente finestre di consenso, ma solo i permessi che la shell ha.

## 1. Audio (bozza, da rimisurare)

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

## 2. Video e finestre delle app (fase 0: strumento di misura)

> Studio completo in `studio/video.md`. Qui le API usate dallo **strumento di
> misura** (`telefono/aiuto/src/phonestra/VideoProva.java`, `Sistema.java`,
> `Pulizia.java`), scritto da zero: scrcpy solo come documentazione. Le scelte
> definitive del componente si fanno dopo le misure.

**Come si lancia** (dal PC, telefono già preparato): `phonestra-prova
video-prova <prova> [opzioni]`. L'uscita arriva riga per riga; i PNG salvati in
`/data/local/tmp` (righe `png: …`) vengono copiati in `phonestra-prova.<nome>.png`
e cancellati dal telefono. Ogni prova chiude sempre schermi virtuali, task,
codificatori e ascoltatori (anche dopo un errore) e un guardiano la termina
comunque dopo un tempo massimo.

| Prova (video.md) | Comando |
|---|---|
| 1 permessi della shell | `video-prova permessi` |
| 2 schermo virtuale minimo | `video-prova schermo [--app P] [--misura 1120x1992] [--dpi 448]` |
| 3 gesto «indietro» | `video-prova schermo --aperto 60` e poi `… --sempre-sbloccato --aperto 60` |
| 5 fotogramma chiave | `video-prova chiave [--codec avc,hevc] [--secondi 20] [--app P --url U] [--nome CODIFICATORE]` |
| 8 istanze | `video-prova istanze [--codec hevc] [--misura 1120x1992] [--massimo 8] [--app P1,P2…]` |
| 10 FLAG_SECURE | `video-prova protetto --app com.x8bit.bitwarden` e poi senza `--app` (Orologio) |
| 11 eventi dei task | `video-prova task [--secondi 30] [--app P1,P2…] [--senza-schermo]` |
| 14 codificatori | `video-prova codificatori` |

API usate (**[P]** pubblica, **[N]** nascosta, per riflessione con ripieghi):

- **Contesto**: `ActivityThread` finto (come per l'audio) e
  `createPackageContext("com.android.shell")`: Android 16 controlla che il
  pacchetto dichiarato sia quello dell'uid.
- **Schermo virtuale** [P]: `Context.getSystemService(DISPLAY_SERVICE)` →
  `DisplayManager.createVirtualDisplay(nome, l, a, dpi, surface, flag)` (ripiego:
  costruttore nascosto `DisplayManager(Context)`). Flag proposti in video.md
  §1.1 (`PUBLIC | PRESENTATION | OWN_CONTENT_ONLY | SUPPORTS_TOUCH |
  ROTATES_WITH_CONTENT | DESTROY_CONTENT_ON_REMOVAL | TRUSTED | OWN_DISPLAY_GROUP |
  OWN_FOCUS | TOUCH_FEEDBACK_DISABLED`), `ALWAYS_UNLOCKED` solo con
  `--sempre-sbloccato`. Flag effettivi letti con `Display.getFlags()` (nomi dalle
  costanti `Display.FLAG_*` del telefono stesso) e dalle righe di `dumpsys display`.
  `VirtualDisplay.setSurface()` [P] per passare da un codificatore all'altro
  senza ricreare lo schermo.
- **Immagini**: `ImageReader.newInstance(l, a, RGBA_8888, 3)` [P] come Surface
  dello schermo, `acquireLatestImage()`, PNG con `Bitmap.compress`.
- **Avvio app**: `PackageManager.getLaunchIntentForPackage` [P] (o
  `ACTION_VIEW` + `setPackage` per un indirizzo), `ActivityOptions.makeBasic()
  .setLaunchDisplayId(id)` [P], `IActivityManager.startActivityAsUser` a 11
  parametri [N] (`ActivityManager.getService()`, utente `-2` = corrente);
  ripiego `am start --display`. La riga stampata dice quale via ha funzionato e
  il codice restituito (0 avviata, 2 portata davanti, negativo = errore).
- **Chiusura dei task**: `IActivityTaskManager.getAllRootTaskInfosOnDisplay(id)`
  + `removeTask(taskId)` [N] (tolgono anche dalle recenti), **prima** di
  `VirtualDisplay.release()`; ripiego `am stack list` / `am stack remove`.
- **Codificatore** [P]: `MediaCodecList(REGULAR_CODECS)`, primo codificatore con
  `isHardwareAccelerated()` e non `isAlias()`; misura arrotondata a
  `getWidthAlignment/HeightAlignment`; formato come scrcpy (8 Mbit/s, 60 fps
  dichiarati, `COLOR_FormatSurface`, `i-frame-interval` 10 s,
  `repeat-previous-frame-after` 100 ms, `priority` 0, `color-range` limitato);
  `createInputSurface()`; uscita letta su un thread suo.
  Fotogramma chiave: `setParameters(PARAMETER_KEY_REQUEST_SYNC_FRAME)` e primo
  buffer con `BUFFER_FLAG_KEY_FRAME`; `prepend-sps-pps-to-idr-frames=1`
  (`KEY_PREPEND_HEADER_TO_SYNC_FRAMES`) provato a parte: se il codificatore non lo
  accetta fallisce `configure` e la prova lo dice. La prova controlla anche nei
  byte dell'IDR se SPS (H.264) o VPS/SPS (H.265) ci sono davvero.
- **Limiti dichiarati** [P]: `CodecCapabilities.getMaxSupportedInstances()`,
  `VideoCapabilities.getSupportedPerformancePoints()` (e `covers()` della misura
  a 60 fps), `areSizeAndRateSupported()`, `EncoderCapabilities
  .isBitrateModeSupported()`.
- **Schermate protette** [N]: `IWindowManager.captureDisplay(displayId,
  CaptureArgs, ScreenCaptureListener)`. I tipi del 2° e 3° parametro si leggono
  dalla firma trovata (Android 14–15 `android.window.ScreenCapture$…`, 16 qpr2
  `ScreenCaptureInternal$…`); `CaptureArgs` dal suo `Builder` (ripiego `null`);
  ascoltatore da `createSyncCaptureListener()` + `getBuffer()` o, in ripiego, dal
  costruttore con `ObjIntConsumer`; attesa massima 5 s. Poi
  `ScreenshotHardwareBuffer.containsSecureLayers()` e `asBitmap()`. Per
  confronto: quota di pixel neri nel PNG dell'ImageReader e numero di finestre
  con `SECURE` in `dumpsys window windows` sullo schermo.
- **Eventi dei task** [N]: sottoclasse di `android.app.TaskStackListener` (classe
  nascosta che implementa già tutti i metodi di `ITaskStackListener.Stub` con
  corpi vuoti: i metodi aggiunti dai produttori non causano
  `AbstractMethodError`), registrata con
  `IActivityTaskManager.registerTaskStackListener` e tolta con
  `unregisterTaskStackListener`. Eventi ascoltati: creazione, rimozione, davanti/
  dietro, cambio di schermo (`onTaskDisplayChanged`), fuoco, orientamento chiesto
  (task e attività), rotazione, avvio su secondo schermo fallito/deviato,
  «indietro» sulla radice. All'avvio la prova confronta le nostre firme con
  quelle del telefono e segnala quelle che non esistono (eventi che non
  arriverebbero).
- **Permessi** [P]: `PackageManager.checkPermission(permesso, "com.android.shell")`.
- **Stato per il gesto «indietro»**: `KeyguardManager.isKeyguardLocked()` ogni
  5 s e `dumpsys activity service com.android.systemui/.SystemUIService`
  (`SysUiState state:`) prima, a schermo aperto e dopo la chiusura.

**Firme controllate senza telefono** (28 set): le API pubbliche compilando
contro `android.jar` dell'SDK 34; quelle nascoste con `dexdump` sul
`framework.jar` delle immagini AOSP dell'emulatore (API 34 e API 37):
`startActivityAsUser` a 11 parametri, `getAllRootTaskInfosOnDisplay(int)`,
`removeTask(int)`, `register/unregisterTaskStackListener`, `DisplayManager(Context)`
ci sono in entrambe; `IWindowManager.captureDisplay` usa `ScreenCapture$…` in 34 e
`ScreenCaptureInternal$…` in 37, entrambe con `createSyncCaptureListener()` e
`CaptureArgs$Builder()`; in 37 l'ascoltatore si costruisce con `ObjIntConsumer`
(in 34 con `Consumer`) e `CaptureArgs` ha `setSecureContentPolicy`.
`TaskStackListener`: in 37 **non c'è più** `onTaskRequestedOrientationChanged`
(resta `onActivityRequestedOrientationChanged`). **Da verificare sul telefono**:
le varianti Samsung (One UI) di queste firme e il comportamento reale.

## 3. Tocchi, tasti, appunti, comandi (da studiare)
