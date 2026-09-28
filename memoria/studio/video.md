# Studio: video, schermi e app per il componente nostro (Android 14–16)

Studio per la sezione «2. Video e finestre delle app» di `memoria/api-android.md`.
Punto di vista: un processo avviato con `app_process` come la shell (uid 2000,
pacchetto `com.android.shell`), Android 14, 15, 16 (API 34–36).

Legenda: **[V]** = verificato nelle fonti citate (codice AOSP o scrcpy letto in
questo studio); **[S]** = segnalato da terzi (issue, articoli), non verificato da
noi; **[I]** = ipotesi mia, da provare sul telefono.

Fonti principali (lette per questo studio):
- scrcpy 4.1 (`versionName "4.1"`, commit `19c1261`, 5 set 2026):
  <https://github.com/Genymobile/scrcpy/tree/master/server/src/main/java/com/genymobile/scrcpy>
- AOSP, rami `android14-release`, `android15-release`, `android16-qpr2-release`
  su <https://android.googlesource.com/platform/frameworks/base/> (gli stessi file
  si leggono su <https://cs.android.com>).
- Documentazione: <https://developer.android.com/about/versions/16/behavior-changes-16>,
  <https://developer.android.com/reference/android/media/MediaCodec>,
  <https://developer.android.com/reference/android/media/MediaFormat>.

---

## Riepilogo e raccomandazioni

1. **Schermo virtuale**: `DisplayManager.createVirtualDisplay(VirtualDisplayConfig)` (API
   pubblica) con i flag `PUBLIC | PRESENTATION | OWN_CONTENT_ONLY | SUPPORTS_TOUCH |
   ROTATES_WITH_CONTENT | DESTROY_CONTENT_ON_REMOVAL | TRUSTED | OWN_DISPLAY_GROUP |
   OWN_FOCUS | TOUCH_FEEDBACK_DISABLED`; i permessi (`ADD_TRUSTED_DISPLAY`,
   `ADD_ALWAYS_UNLOCKED_DISPLAY`, `INTERNAL_SYSTEM_WINDOW`) la shell li ha in 14 e 16 [V].
2. **`ALWAYS_UNLOCKED` da provare senza**: su Galaxy S23+/S26 con Android 16 è indicato
   come causa del gesto «indietro» che smette di funzionare fino al riavvio (scrcpy #6007) [S].
3. **Schermo principale**: `DisplayManager.createVirtualDisplay(nome, l, a, idDaSpecchiare,
   surface)` (nascosta, `@SystemApi`, `CAPTURE_VIDEO_OUTPUT` che la shell ha) [V].
4. **Avvio app**: `ActivityOptions.setLaunchDisplayId()` (pubblica) + `IActivityTaskManager`
   /`IActivityManager.startActivityAsUser` come scrcpy; eventi dei task con
   `ITaskStackListener` (sostituisce il `dumpsys` per «solo verticale» e «app spostata sul telefono»).
5. **Codifica**: `MediaCodec` da Surface come scrcpy, ma con **fotogramma chiave a comando**
   (`PARAMETER_KEY_REQUEST_SYNC_FRAME`) e **bitrate al volo** (`PARAMETER_KEY_VIDEO_BITRATE`)
   invece di ricreare il codificatore; ricreare solo al cambio di misura.
6. **Pausa delle finestre nascoste**: `PARAMETER_KEY_SUSPEND` ferma la codifica senza chiudere
   nulla (batteria e codificatori liberi) [V API; effetto I].
7. **Pannello spento**: `SurfaceControl.setDisplayPowerMode()` con i token di
   `DisplayControl` (in `services.jar`), come scrcpy; la nuova `requestDisplayPower` di 15/16
   ha cambiato firma ed è stata scartata da scrcpy [V].
8. **FLAG_SECURE**: nero per costruzione (la shell non ha `CAPTURE_SECURE_VIDEO_OUTPUT`) [V];
   per accorgersene: `ITaskStackListener` + `IWindowManager.captureDisplay()` →
   `containsSecureLayers()` [I], con il `dumpsys` attuale come riserva.
9. **Android 16 e app solo verticali**: sugli schermi con larghezza minima ≥ 600 dp le app con
   targetSdk 36 **ignorano** il blocco dell'orientamento [V doc]: la regola «finestra a misura
   fissa» va decisa sul valore *chiesto* dall'app, non sulle bande nere.
10. Tutte le API nascoste vanno chiamate per riflessione con ripieghi per firma (come scrcpy):
    Samsung One UI 8.5 ha già aggiunto metodi astratti a interfacce AIDL (#6925) [S].

---

## 1. Schermi virtuali e cattura dello schermo principale

### 1.1 Come si crea uno schermo virtuale per le app

**Come fa scrcpy** [V] (`video/NewDisplayCapture.java`, `startNew()`,
<https://github.com/Genymobile/scrcpy/blob/master/server/src/main/java/com/genymobile/scrcpy/video/NewDisplayCapture.java>):
costruisce per riflessione un `android.hardware.display.DisplayManager(Context)` con il suo
`FakeContext` (pacchetto `com.android.shell`) e chiama la **pubblica**
`createVirtualDisplay(name, w, h, dpi, surface, flags)`. Flag usati su API 34+:

| Flag | Valore | Stato nell'API | Cosa fa | Permesso richiesto | Shell |
|---|---|---|---|---|---|
| `PUBLIC` | 1<<0 | pubblico | altre app possono mostrarsi lì | — (ma senza `OWN_CONTENT_ONLY` diventa `AUTO_MIRROR`) | — |
| `PRESENTATION` | 1<<1 | pubblico | consente finestre `Presentation` | — | — |
| `SECURE` | 1<<2 | pubblico | mostra le finestre FLAG_SECURE | `CAPTURE_SECURE_VIDEO_OUTPUT` | **no** |
| `OWN_CONTENT_ONLY` | 1<<3 | pubblico | niente specchio: solo il contenuto suo | — | — |
| `AUTO_MIRROR` | 1<<4 | pubblico | specchia un altro schermo | `CAPTURE_VIDEO_OUTPUT` | sì |
| `SUPPORTS_TOUCH` | 1<<6 | @hide | accetta tocchi | — | — |
| `ROTATES_WITH_CONTENT` | 1<<7 | @hide | ruota con l'app | — | — |
| `DESTROY_CONTENT_ON_REMOVAL` | 1<<8 | @hide | alla chiusura le attività muoiono invece di passare al telefono | — | — |
| `SHOULD_SHOW_SYSTEM_DECORATIONS` | 1<<9 | @TestApi | barre di sistema, launcher | solo se `TRUSTED` | — |
| `TRUSTED` | 1<<10 | @SystemApi | schermo «fidato» | `ADD_TRUSTED_DISPLAY` | **sì** |
| `OWN_DISPLAY_GROUP` | 1<<11 | @hide | gruppo di schermi proprio (acceso/spento a sé) | `ADD_TRUSTED_DISPLAY` | **sì** |
| `ALWAYS_UNLOCKED` | 1<<12 | @hide | non segue la schermata di blocco | `ADD_ALWAYS_UNLOCKED_DISPLAY` | **sì** |
| `TOUCH_FEEDBACK_DISABLED` | 1<<13 | @hide | niente vibrazione/suono ai tocchi | — | — |
| `OWN_FOCUS` | 1<<14 | @TestApi | fuoco proprio (non ruba quello del telefono) | ignorato se non `TRUSTED` | — |
| `DEVICE_DISPLAY_GROUP` | 1<<15 | @hide | gruppo di un VirtualDevice | senza VirtualDevice non fa nulla | — |
| `STEAL_TOP_FOCUS_DISABLED` | 1<<16 | @SystemApi | non diventa lo schermo «in primo piano» | richiede `OWN_FOCUS` | — |

Fonti dei valori e degli attributi: `DisplayManager.java`
([A14](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android14-release/core/java/android/hardware/display/DisplayManager.java),
[A16 qpr2](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android16-qpr2-release/core/java/android/hardware/display/DisplayManager.java)) [V].
I valori sono **identici da 14 a 16** [V].

**Controlli dei permessi** in `DisplayManagerService.createVirtualDisplayInternal()` [V]
([A14, righe ~1395–1530](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android14-release/services/core/java/com/android/server/display/DisplayManagerService.java),
[A16 qpr2, righe ~1882–2085](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android16-qpr2-release/services/core/java/com/android/server/display/DisplayManagerService.java)):
- `PUBLIC` senza `OWN_CONTENT_ONLY` → diventa `AUTO_MIRROR` → serve `CAPTURE_VIDEO_OUTPUT`.
  Con `OWN_CONTENT_ONLY` l'`AUTO_MIRROR` viene tolto: per uno schermo «app» **non serve**
  `CAPTURE_VIDEO_OUTPUT`.
- `SECURE` → `CAPTURE_SECURE_VIDEO_OUTPUT`: **la shell non ce l'ha** (manifest di Shell,
  [A14](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android14-release/packages/Shell/AndroidManifest.xml)
  e A16 qpr2: nessuna riga `CAPTURE_SECURE_VIDEO_OUTPUT`) [V]. Da qui le schermate protette nere.
- `TRUSTED`, `OWN_DISPLAY_GROUP` → `ADD_TRUSTED_DISPLAY`; `ALWAYS_UNLOCKED` →
  `ADD_ALWAYS_UNLOCKED_DISPLAY`. Nel manifest di Shell 14 e 16 ci sono entrambi (sotto il
  commento «CtsVirtualDevicesTestCases»), più `CAPTURE_VIDEO_OUTPUT`, `READ_FRAME_BUFFER`,
  `ACCESS_SURFACE_FLINGER`, `INTERNAL_SYSTEM_WINDOW`, `DEVICE_POWER`,
  `MANAGE_ACTIVITY_TASKS`, `REMOVE_TASKS`, `START_ACTIVITIES_FROM_BACKGROUND`,
  `CREATE_VIRTUAL_DEVICE`; in 16 anche `MANAGE_DISPLAYS` [V].
- **Novità Android 16** [V, assente in A14 e A15]: `ALWAYS_UNLOCKED` viene **ignorato**
  se lo schermo non ha `OWN_DISPLAY_GROUP` (o `DEVICE_DISPLAY_GROUP`); `OWN_FOCUS` e
  `SHOULD_SHOW_SYSTEM_DECORATIONS` ignorati senza `TRUSTED`; `STEAL_TOP_FOCUS_DISABLED`
  ignorato senza `OWN_FOCUS`. In A16 c'è anche `validatePackageName(ownerUid, packageName)`:
  il pacchetto dichiarato **deve** essere quello dell'uid (quindi `com.android.shell`, come
  fa già il nostro aiutante con `createPackageContext("com.android.shell")`).

**Perché serve `TRUSTED` (e `PUBLIC`)** [V] (`ActivityTaskSupervisor.isCallerAllowedToLaunchOnDisplay`,
[A15](https://github.com/aosp-mirror/platform_frameworks_base/blob/android15-release/services/core/java/com/android/server/wm/ActivityTaskSupervisor.java)):
chi ha `INTERNAL_SYSTEM_WINDOW` (la shell) può avviare dove vuole; ma poi è **l'app** ad
avviare le sue attività successive o altre app (browser, condivisione). Su uno schermo non
fidato si possono lanciare solo attività con `allowEmbedded`; su uno schermo privato (non
`PUBLIC`) solo il proprietario o un uid già presente. Senza `TRUSTED|PUBLIC` i link e le
condivisioni aperti dall'app finirebbero bloccati o sul telefono.

**Raccomandazione.** Usare l'API pubblica `DisplayManager.createVirtualDisplay(VirtualDisplayConfig,
Handler, VirtualDisplay.Callback)` (in A16 è pubblica; `VirtualDisplayConfig.Builder` ha anche
`setRequestedRefreshRate()` pubblico) con un `Context` del pacchetto shell. Vantaggi:
- `setRequestedRefreshRate(60)` [V, API pubblica in A15/A16]: gli schermi virtuali altrimenti
  vanno alla frequenza del pannello (120 Hz sugli S23+/S26) → meno lavoro per GPU e
  codificatore. Da confrontare con `KEY_MAX_FPS_TO_ENCODER` (§3).
- il `VirtualDisplay.Callback` (`onPaused/onResumed/onStopped`) avvisa se il sistema smette
  di disegnare.

Flag proposti: `PUBLIC | PRESENTATION | OWN_CONTENT_ONLY | SUPPORTS_TOUCH |
ROTATES_WITH_CONTENT | DESTROY_CONTENT_ON_REMOVAL | TRUSTED | OWN_DISPLAY_GROUP | OWN_FOCUS |
TOUCH_FEEDBACK_DISABLED`, **senza** `SHOULD_SHOW_SYSTEM_DECORATIONS` (oggi
`vd_system_decorations=false`) e con `ALWAYS_UNLOCKED` **da decidere dopo la prova** (§6):
- con `ALWAYS_UNLOCKED`: le app sul PC restano visibili anche col telefono bloccato (utile col
  pannello spento?) ma su Samsung A16 è sospettato del difetto del gesto «indietro» [S];
- senza: quando il telefono si blocca lo schermo virtuale mostra il blocco/nero [I] — ma
  Phonestra oggi lavora a telefono sbloccato, e al blocco chiude comunque il debug (§10 prove).
- `DEVICE_DISPLAY_GROUP` (che scrcpy aggiunge) senza VirtualDevice produce solo un messaggio nel
  registro [V, A14 riga ~1649]: inutile.

**Densità e misure.** La densità è un parametro della creazione e di `VirtualDisplay.resize(l,
a, dpi)` (pubblica). Misure multiple di 8 (o dell'allineamento del codificatore,
`VideoCapabilities.getWidthAlignment/HeightAlignment`): scrcpy le allinea [V,
`VideoConstraints`], e noi abbiamo visto che scarta in silenzio i clic con misure diverse
(prove §18) — nel componente nostro l'allineamento va fatto **prima** di creare lo schermo e
restituito al PC.

### 1.2 Cattura dello schermo principale (drawer)

scrcpy [V] (`video/ScreenCapture.java`, `wrappers/DisplayManager.java`) prova, nell'ordine:
1. `DisplayManager.createVirtualDisplay(String, int, int, int displayIdToMirror, Surface)`:
   metodo **statico nascosto** (`@SystemApi`, `@RequiresPermission(CAPTURE_VIDEO_OUTPUT)`),
   che crea uno schermo `AUTO_MIRROR` con `setDisplayIdToMirror()` e densità 1
   ([A14 riga ~1567](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android14-release/core/java/android/hardware/display/DisplayManager.java)) [V].
   Presente in 14–16; è la via usata oggi.
2. ripiego `SurfaceControl.createDisplay/setDisplaySurface/setDisplayProjection/setDisplayLayerStack`:
   su 14+ `createDisplay` non è più in `SurfaceControl` (in A16 c'è
   `DisplayControl.createVirtualDisplay(String, boolean)` in `services.jar`) [V]: per noi
   (solo 14+) il ripiego **non serve**; se il punto 1 fallisse, meglio un errore chiaro.

Punti da sapere:
- La misura del video la decide la misura dello schermo di specchio (scrcpy passa la misura
  ridotta a `max_size`, senza filtro OpenGL quando non ci sono rotazioni/ritagli) [V].
- Rotazione del telefono: scrcpy ascolta i cambi con `IWindowManager.registerDisplayWindowListener`
  perché `DisplayListener` su 14 e su alcune versioni di 15 non manda eventi (commento in
  `display/DisplayMonitor.java`, issue [#5908](https://github.com/Genymobile/scrcpy/issues/5908)) [V/S].
  Al cambio ricrea codificatore e specchio.
- Il drawer mostra il telefono: può restare H.264 a misura contenuta e 30 fotogrammi/s
  (`KEY_MAX_FPS_TO_ENCODER`) per risparmiare il codificatore per le finestre delle app [I].

### 1.3 Alternative scartate o da tenere in serbo
- **MediaProjection**: chiede il consenso sullo schermo e un'app: fuori dal nostro modello.
- **VirtualDeviceManager** (`CREATE_VIRTUAL_DEVICE` è della shell [V]): è la via «ufficiale»
  per lo streaming di app (Chromebook, Phone Link), con schermi e input virtuali propri; ma
  richiede un'associazione `CompanionDeviceManager` con profilo `APP_STREAMING` [I: non
  verificato che la shell possa crearla senza app]. Da tenere come ricerca futura, non ora.

---

## 2. Avvio delle app, task, chiusura, tastiera, decorazioni

### 2.1 Avvio su uno schermo preciso
scrcpy [V] (`device/Device.startApp`, `wrappers/ActivityManager.startActivity`):
- `PackageManager.getLaunchIntentForPackage()` (o `getLeanbackLaunchIntentForPackage`),
  `FLAG_ACTIVITY_NEW_TASK`;
- `ActivityOptions.makeBasic().setLaunchDisplayId(id)` (**pubblica**, API 26);
- `IActivityManager.startActivityAsUser(null, "com.android.shell", intent, null, null, null,
  0, 0, null, options, USER_CURRENT=-2)` per riflessione (via `ActivityManagerNative.getDefault()`);
- con il prefisso `+` prima `forceStopPackage`.

Proposta: stessa via, ma (a) `IActivityTaskManager.startActivity` è l'equivalente più diretto
[V, presente in `IActivityTaskManager.aidl` A15]; (b) opzionale
`ActivityOptions.setLaunchWindowingMode(FULLSCREEN)` per evitare che One UI apra in finestra
libera (DeX/«desktop») [I]; (c) `FLAG_ACTIVITY_MULTIPLE_TASK` **no**: se l'app è già aperta sul
telefono Android la **sposta** sullo schermo virtuale (comportamento che vogliamo).

### 2.2 Seguire i task: `ITaskStackListener`
`IActivityTaskManager.registerTaskStackListener()` (permesso `MANAGE_ACTIVITY_TASKS`, che la
shell ha) dà eventi utili [V, `ITaskStackListener.aidl`
[A15](https://github.com/aosp-mirror/platform_frameworks_base/blob/android15-release/core/java/android/app/ITaskStackListener.aidl)]:
- `onTaskDisplayChanged(taskId, newDisplayId)` → «App aperta sul telefono – riportala qui»
  (oggi la ricaviamo in altro modo); riportarla: `moveRootTaskToDisplay(taskId, displayId)`
  [V, `IActivityTaskManager`];
- `onTaskRequestedOrientationChanged(taskId, orientamento)` e
  `onActivityRequestedOrientationChanged` → **app solo verticale** senza `dumpsys` ogni tot
  (il `dumpsys` costava ~100 ms e disturbava l'audio, prove §27);
- `onTaskRemoved`, `onTaskMovedToFront`, `onTaskFocusChanged`, `onTaskCreated`.
Chiusura: `IActivityTaskManager.removeTask(taskId)` (`REMOVE_TASKS`, della shell) [V];
`getTasks()`/`getAllRootTaskInfosOnDisplay(displayId)` per sapere cosa c'è su uno schermo.
Rischio: l'interfaccia cresce da una versione all'altra (A14→A15: aggiunti metodi) → la classe
`Stub` va estesa e `onTransact` deve **ignorare i metodi sconosciuti** come fa scrcpy per
`IDisplayWindowListener` (§5.3).

### 2.3 Tastiera virtuale (IME)
`IWindowManager.setDisplayImePolicy(displayId, policy)` (A12+, @hide) con
`LOCAL=0`, `FALLBACK_DISPLAY=1` (predefinita per gli schermi virtuali: la tastiera compare
sul telefono), `HIDE=2` [V, scrcpy `wrappers/WindowManager.java`, `IWindowManager.aidl` A16 riga ~779].
Phonestra usa la tastiera del PC: proposta **`HIDE`**, così la tastiera Samsung non si apre sul
pannello del telefono (spento) né consuma [I: verificare che i campi di testo ricevano
comunque i tasti iniettati — dovrebbe, perché i `KeyEvent` vanno alla finestra con il fuoco].
Nota: con la tastiera Samsung `LOCAL` può non funzionare (scrcpy
[#5997](https://github.com/Genymobile/scrcpy/issues/5997)) [S]. scrcpy ripristina la
politica alla fine con il processo di pulizia (§4.5).

### 2.4 Decorazioni di sistema
Senza `SHOULD_SHOW_SYSTEM_DECORATIONS` niente barra di Samsung né launcher (prove §9) [V
nostro]. Con le decorazioni Samsung A16 entra in modalità DeX/desktop e mostra due barre
([#6591](https://github.com/Genymobile/scrcpy/issues/6591)) [S]; su Motorola A16 il flag
mancante non basta più ([#6684](https://github.com/Genymobile/scrcpy/issues/6684)) [S]. Da
tenere spento.

---

## 3. Codifica (MediaCodec da Surface)

### 3.1 Cosa fa scrcpy [V] (`video/SurfaceEncoder.java`)
- `MediaCodec.createEncoderByType(mime)` (o per nome), formato: `KEY_BIT_RATE`,
  `KEY_FRAME_RATE=60` (obbligatorio ma non vincolante), `COLOR_FormatSurface`,
  `KEY_COLOR_RANGE=LIMITED`, `KEY_I_FRAME_INTERVAL=10` s,
  `KEY_REPEAT_PREVIOUS_FRAME_AFTER=100 000` µs, `KEY_PRIORITY=0` (tempo reale),
  `KEY_LATENCY=1`, `max-fps-to-encoder` se richiesto.
- Ciclo: `configure` → `createInputSurface` → collega lo schermo (`VirtualDisplay.setSurface`
  se esiste già) → `start` → legge le uscite (`dequeueOutputBuffer(-1)`) finché EOS.
- **Ogni «reset»** (cambio misura/rotazione, **RESET_VIDEO del PC**, ridimensionamento) =
  `signalEndOfInputStream()` → `stop` → `reset` → nuovo `configure` + nuova Surface
  (`CaptureControl.reset`). Quindi il nostro «ricomincia video» oggi **ricrea il
  codificatore**: da qui, verosimilmente, la ripartenza di 1–2 s vista nelle prove §27 [I].
- Errori: fino a 3 tentativi, poi riduzione della misura (2560, 1920, 1600…) se
  `downsize_on_error`.
- Thread del video con `Looper.prepare()` (senza, stallo su Meizu,
  [#4143](https://github.com/Genymobile/scrcpy/issues/4143)) [S].
- Non usa `PARAMETER_KEY_REQUEST_SYNC_FRAME` né `PARAMETER_KEY_VIDEO_BITRATE` [V: nessuna
  occorrenza nel codice].

### 3.2 Parametri: cosa dicono le fonti
Da `MediaFormat.java` A16 qpr2 e dalla documentazione [V]:

| Chiave | Stringa | API | Note |
|---|---|---|---|
| `KEY_REPEAT_PREVIOUS_FRAME_AFTER` | `repeat-previous-frame-after` | 19 | solo input da Surface: ripete il fotogramma se non arriva niente; dà il primo fotogramma anche a schermo fermo. Su alcuni codificatori Codec2 viene **ignorato** («no c2 equivalents», Motorola A16, [#6500](https://github.com/Genymobile/scrcpy/issues/6500)) [S] |
| `KEY_MAX_FPS_TO_ENCODER` | `max-fps-to-encoder` | 29 (prima privata) | scarta i fotogrammi in eccesso **prima** del codificatore |
| `KEY_MAX_PTS_GAP_TO_ENCODER` | `max-pts-gap-to-encoder` | 29 | limita i salti di orario tra fotogrammi |
| `KEY_CREATE_INPUT_SURFACE_SUSPENDED` | `create-input-buffers-suspended` | 29 | crea sospeso, poi riprende con `PARAMETER_KEY_SUSPEND` |
| `KEY_LATENCY` | `latency` | 26 | solo codificatori: fotogrammi di ritardo ammessi (1) |
| `KEY_LOW_LATENCY` | `low-latency` | 30 | **solo decodificatori**: inutile qui |
| `KEY_PRIORITY` | `priority` | 23 | 0 = tempo reale |
| `KEY_OPERATING_RATE` | `operating-rate` | 23 | pianificazione delle risorse |
| `KEY_PREPEND_HEADER_TO_SYNC_FRAMES` | `prepend-sps-pps-to-idr-frames` | 29 | SPS/PPS davanti a ogni IDR; se non supportato **fallisce `configure`** |
| `KEY_INTRA_REFRESH_PERIOD` | `intra-refresh-period` | 24 | rinfresco progressivo invece di fotogrammi chiave (se `FEATURE_IntraRefresh`) |
| `KEY_MAX_B_FRAMES` | `max-bframes` | 29 | 0 = niente B (latenza) |
| `KEY_ALLOW_FRAME_DROP` | `allow-frame-drop` | 31 | per decodificatori su Surface |

`MediaCodec.setParameters(Bundle)` a codifica in corso (API 19) [V doc]:
`PARAMETER_KEY_REQUEST_SYNC_FRAME` (fotogramma chiave «appena possibile»),
`PARAMETER_KEY_VIDEO_BITRATE` (bitrate al volo), `PARAMETER_KEY_SUSPEND` (+ `_SUSPEND_TIME`
API 29: sospende l'input da Surface), `PARAMETER_KEY_OFFSET_TIME` (API 29).

### 3.3 Proposte per il nostro codificatore
1. **Fotogramma chiave a comando** = `setParameters(REQUEST_SYNC_FRAME)`, senza ricreare nulla;
   con `KEY_PREPEND_HEADER_TO_SYNC_FRAMES=1` (con ripiego se `configure` fallisce) il PC può
   ripartire da qualunque IDR. Ricreare il codificatore solo se non arriva un IDR entro ~300 ms [I].
2. **Qualità adattiva (§14 delle specifiche)**: prima `PARAMETER_KEY_VIDEO_BITRATE` (al volo,
   nessuna interruzione), poi fotogrammi/s (richiede nuovo `configure` con
   `KEY_MAX_FPS_TO_ENCODER`, oppure `resize`/refresh dello schermo virtuale [I]), poi misura.
   Da provare se Qualcomm/Exynos rispettano il cambio di bitrate al volo in VBR; eventualmente
   `KEY_BITRATE_MODE=CBR` [I].
3. **Finestra ridotta a icona o coperta** → `PARAMETER_KEY_SUSPEND=1`; alla riapertura
   `SUSPEND=0` + `REQUEST_SYNC_FRAME` [I sull'effetto concreto: batteria, calore].
4. **Cambio di misura**: il codificatore non cambia risoluzione da solo → nuovo `configure` e
   nuova Surface. Per accorciare il buco: preparare il nuovo codificatore **prima**, poi
   `VirtualDisplay.resize()` + `setSurface(nuova)` e solo dopo chiudere il vecchio [I; costa
   un'istanza in più per un attimo]. In alternativa `MediaCodec.createPersistentInputSurface()`
   + `setInputSurface()` (API 23) [I: non chiaro se la misura dei buffer segue il `configure`].
5. **Limiti hardware**: `CodecCapabilities.getMaxSupportedInstances()` (API 23) e
   `VideoCapabilities.getSupportedPerformancePoints()` (API 29) per stimare quante finestre
   insieme (§7.6: «da misurare») prima di arrivare all'errore `0xfffffff4` / risorse esaurite
   ([#5798](https://github.com/Genymobile/scrcpy/issues/5798), rom1v: «limite di quanto il
   codificatore hardware può codificare in parallelo») [S].
6. **Codec**: H.265 predefinito come da specifiche; elenco con `MediaCodecList(REGULAR_CODECS)`
   scartando i software (`isSoftwareOnly()`, API 29). AV1: da elencare sul telefono (sul
   Snapdragon 8 Gen 2 dell'S23+ credo non ci sia un codificatore AV1 hardware [I]).
7. Orari (pts): quelli della Surface (tempo di composizione) → irregolari per natura; al PC
   servono per la registrazione MP4, per la visione si mostra l'ultimo arrivato (specifiche
   «mai accumulare ritardo»).

### 3.4 Problemi noti dei codificatori (Samsung/Qualcomm/Exynos)
- Allineamento: scrcpy legge `getWidthAlignment/HeightAlignment` [V]; da noi misure non
  multiple di 8 hanno dato problemi (prove §18) [V nostro].
- A 3840×1496 l'S23+ scendeva a 12–20 fps; 2560×1000 a 24; 1120×1992 a 60 (prove §18, §20)
  [V nostro] → tetto 2560 confermato.
- `REPEAT_PREVIOUS_FRAME_AFTER` ignorato su alcuni Codec2 [S]: se accade, il primo fotogramma di
  un'app ferma non arriva; rimedio: `REQUEST_SYNC_FRAME` all'avvio e dopo ogni reset [I].
- Exynos: non ho trovato nelle issue recenti di scrcpy problemi specifici di Exynos 2400/2600
  con input da Surface [S: assenza di prove, non prova di assenza]. Da provare sull'S26
  (verificare quale SoC monta l'esemplare di prova: `getprop ro.soc.model`).

---

## 4. Rotazione, dimensioni, densità, solo verticali, FLAG_SECURE, pannello

### 4.1 Rotazione e schermo virtuale
- Con `ROTATES_WITH_CONTENT` lo schermo virtuale può ruotare quando l'app lo chiede; scrcpy
  allora ruota l'immagine con un filtro OpenGL (`OpenGLRunner`) perché il buffer resta nella
  forma originale [V, `NewDisplayCapture.prepare/start`]. Se `displayRotation == 0` il filtro
  non si usa [V]. Nelle nostre prove invece le app solo verticali venivano **messe a bande**
  (`areBoundsLetterboxed=true`, prove §27) [V nostro].
- Proposta: niente OpenGL nel primo pezzo; se lo schermo virtuale ruota (evento da
  `IDisplayWindowListener.onDisplayConfigurationChanged`), rifare lo schermo con lati scambiati
  o avvisare il PC [I].

### 4.2 App solo verticali e Android 16
- **Fatto** [V doc, <https://developer.android.com/about/versions/16/behavior-changes-16>]:
  con targetSdk 36, sugli schermi con **larghezza minima ≥ 600 dp**, Android 16 ignora
  `screenOrientation`, `resizeableActivity`, `min/maxAspectRatio`, `setRequestedOrientation()`;
  eccezioni: giochi (`appCategory`), scelta dell'utente nelle impostazioni del rapporto
  d'aspetto, schermi < sw600dp; rinuncia temporanea con
  `PROPERTY_COMPAT_ALLOW_RESTRICTED_RESIZABILITY` (non più dal targetSdk 37).
- Conseguenza [I]: una finestra di Phonestra a 1 punto = 1 dp, larga e alta più di 600 punti,
  crea uno schermo sw ≥ 600 dp: un'app con targetSdk 36 «solo verticale» verrà **allungata**
  invece che messa a bande. La decisione dell'utente (§29: niente ridimensionamento per le app
  solo verticali) resta valida, ma il riconoscimento non può basarsi sulle bande: usare
  l'orientamento **chiesto** (`onTaskRequestedOrientationChanged`) e, se serve, tenere lo
  schermo sotto i 600 dp di lato corto per queste app.
- A16 aggiunge anche `VirtualDisplayConfig.Builder.setIgnoreActivitySizeRestrictions()`
  (`@SystemApi`, flag di funzione, richiede `TRUSTED`) [V]: è l'**opposto** di quello che ci
  serve; lasciarlo spento.

### 4.3 Densità
Densità fissata alla creazione e a ogni `resize(l, a, dpi)` [V]. Oggi: densità del telefono,
pixel in proporzione, lato massimo 2560 (prove §18) [V nostro]. Il cambio di densità a schermo
vivo rilancia la configurazione delle app (possibile ricreazione dell'attività) [I]: meglio
tenerla fissa per tutta la vita della finestra, come oggi.

### 4.4 Schermate protette (FLAG_SECURE)
- **Fatto** [V]: senza `CAPTURE_SECURE_VIDEO_OUTPUT` non si può creare uno schermo `SECURE`;
  le finestre `FLAG_SECURE` sono composte nere nella nostra Surface. Non si aggira (ed è la
  decisione del progetto).
- Come accorgersene:
  1. **Oggi**: `dumpsys window windows`, `FLAG_SECURE 0x2000` con `mHasSurface=true` [V nostro]
     — funziona ma è lento e dipende dal formato del testo.
  2. **Proposta** [I]: `IWindowManager.captureDisplay(displayId, CaptureArgs, listener)`
     (richiede `READ_FRAME_BUFFER`, che la shell ha
     [V, `WindowManagerService.captureDisplay` A14 riga ~9429]) restituisce un
     `ScreenshotHardwareBuffer` con **`containsSecureLayers()`** [V che il metodo esiste in
     `android.window.ScreenCapture` A14]. Da provare se è vero anche quando i livelli protetti
     vengono anneriti (non catturati). Chiamarlo solo a eventi (`onTaskMovedToFront`, cambio
     di fuoco) e non a intervalli. Attenzione: in A16 qpr2 il parametro è diventato
     `ScreenCaptureInternal.CaptureArgs` [V, `IWindowManager.aidl` riga ~1109] → firma diversa
     tra versioni.
  3. Riserva: fotogramma tutto nero per > 1 s mentre l'app è in primo piano [I].
- Android 15 «protezione durante la condivisione dello schermo» (notifiche e campi sensibili
  nascosti) riguarda le sessioni **MediaProjection** [S,
  <https://developer.android.com/about/versions/15/behavior-changes-all>]; noi non ne usiamo
  → verosimilmente non ci tocca [I, da verificare con un OTP].

### 4.5 Spegnere il pannello lasciando il telefono attivo
Come fa scrcpy [V] (`device/Device.setDisplayPower`, `wrappers/DisplayControl.java`,
`wrappers/SurfaceControl.java`):
- da API 34, se `SurfaceControl.getPhysicalDisplayIds` non esiste (in A16 infatti non c'è
  [V]), carica `com.android.server.display.DisplayControl` da `SYSTEMSERVERCLASSPATH` con
  `ClassLoaderFactory.createClassLoader(...)` e `Runtime.loadLibrary0(..., "android_servers")`;
  `DisplayControl.getPhysicalDisplayIds()` + `getPhysicalDisplayToken(id)` [V in A16 qpr2];
- per **ogni** schermo fisico `SurfaceControl.setDisplayPowerMode(token, 0=OFF / 2=NORMAL)`
  [V: ancora presente in `SurfaceControl.java` A16 qpr2, riga ~2139];
- eccezione Honor su A14 ([#4823](https://github.com/Genymobile/scrcpy/issues/4823)) [S];
- **non usa** `DisplayManagerGlobal.requestDisplayPower()` (A15, `MANAGE_DISPLAYS`): la firma
  è `(int, boolean)` in AOSP 15 [V] ma `(int, int state)` su vivo A15 e in AOSP 16 [V], e dove
  funzionava bloccava l'input ([#5530](https://github.com/Genymobile/scrcpy/issues/5530)) [S].
- Spegnere così agisce solo su SurfaceFlinger/HWC: `PowerManager` crede lo schermo acceso, per
  questo il **touch del telefono resta attivo** (specifiche §limiti) [I, spiegazione coerente].
  Qualunque cambio di stato (tasto di accensione, notifica che accende lo schermo) riporta il
  pannello acceso: va riapplicato (scrcpy lo fa dopo il tasto POWER, `scheduleDisplayPowerOff`,
  200 ms) [V].
- **Ripristino a prova di caduta**: scrcpy lancia un secondo processo (`CleanUp`) che resta vivo
  se il principale muore e ripristina pannello, `stay_on`, tempo di spegnimento, politica IME [V].
  Consigliato anche per noi (oggi «il componente che termina lo riaccende»: non copre il
  `kill` o la morte improvvisa).
- Samsung: con il telefono **in sospensione** i clic sugli schermi virtuali smettono di
  funzionare ([#5561](https://github.com/Genymobile/scrcpy/issues/5561), rom1v) [S] — il nostro
  modello (telefono sveglio, pannello spento) lo evita; e One UI 8 può chiudere il Wi-Fi a
  schermo spento ([#6454](https://github.com/Genymobile/scrcpy/issues/6454)) [S].

---

## 5. Come lo risolve scrcpy 4.x: API nascoste e accorgimenti

### 5.1 API nascoste usate (per riflessione) e perché
| Classe/metodo | Perché | Stato 14–16 |
|---|---|---|
| `ActivityThread` finto (`sCurrentActivityThread`, `mSystemThread`, `getSystemContext`) | ottenere un `Context` di sistema in `app_process` | [V] in uso su 14–16 |
| `ConfigurationController` in `ActivityThread` | Samsung: `DisplayManagerGlobal.getDisplayInfoLocked` chiede la configurazione ([#4467](https://github.com/Genymobile/scrcpy/issues/4467)) | già nel nostro aiutante |
| `AppBindData` / `mInitialApplication` | alcune API chiedono l'app corrente (non su ONYX) | — |
| `FakeContext` con pacchetto `com.android.shell`, `AttributionSource(SHELL_UID)` | attribuzione dei permessi | necessario in A16 (`validatePackageName`) |
| `DisplayManager(Context)` costruttore nascosto | chiamare la `createVirtualDisplay` pubblica con il nostro contesto | ok |
| `DisplayManager.createVirtualDisplay(name,w,h,displayIdToMirror,surface)` statico `@SystemApi` | specchio dello schermo principale | ok |
| `DisplayManagerGlobal.getDisplayInfo(id)` e campi `logicalWidth/Height`, `rotation`, `logicalDensityDpi`, `layerStack`, `flags`, `uniqueId` | stato degli schermi (ripiego: parsing di `dumpsys display`) | `uniqueId` può mancare ([#6461](https://github.com/Genymobile/scrcpy/issues/6461)) [S] |
| `IWindowManager.registerDisplayWindowListener` | cambi di misura/rotazione (DisplayListener rotto su 14 e su alcune 15) | ok |
| `IWindowManager.setDisplayImePolicy/getDisplayImePolicy`, `freezeDisplayRotation(id, rot, caller)` | IME, rotazione (firma con `String` da 14 QPR3/15) | firme che cambiano [V] |
| `IActivityManager.startActivityAsUser`, `forceStopPackage` | avvio app | ok |
| `SurfaceControl.setDisplayPowerMode`, `DisplayControl.getPhysicalDisplayIds/Token` | pannello | `DisplayControl` sta in `services.jar` |
| `InputManager.injectInputEvent`, `setDisplayId` | input (fuori da questo studio) | — |

Nota sulle restrizioni alle API nascoste: un processo avviato con `app_process` dalla shell non
passa dallo zygote e scrcpy (come il nostro aiutante) usa la riflessione su 14–16 senza
problemi [V nell'uso; I sul meccanismo preciso].

### 5.2 Differenze per versione gestite da scrcpy [V]
- API 34: flag `OWN_FOCUS`, `DEVICE_DISPLAY_GROUP`; `DisplayControl` per gli schermi fisici;
  `DisplayMonitor` passa a `IDisplayWindowListener`; Honor A14.
- API 35: `requestDisplayPower` (disattivato); `freezeDisplayRotation` con parametro `caller`.
- API 36 / One UI 8.5: nuovi metodi astratti in `IDisplayWindowListener`
  (`onDisplayAnimationsDisabledChanged`) → `AbstractMethodError` e disconnessione nelle versioni
  vecchie ([#6925](https://github.com/Genymobile/scrcpy/issues/6925)) [S]; scrcpy 4.1 la evita
  sovrascrivendo `onTransact` e rispondendo `writeNoException()` ai metodi sconosciuti
  (`wrappers/DisplayWindowListener.java`) [V].

### 5.3 Cosa riprendere e cosa no
- **Riprendere**: `FakeContext`/`ActivityThread`, il trucco di `onTransact` per ogni `Stub`
  AIDL che implementiamo, il processo di pulizia separato, il ciclo «reset → ricrea» del
  codificatore (solo per i cambi di misura), l'allineamento delle misure, il debouncer del
  ridimensionamento (`DisplayResizeDebouncer`).
- **Non serve** (solo 14+): i rami per API < 34, il ripiego `SurfaceControl.createDisplay`,
  il ramo Oculus, `DEVICE_DISPLAY_GROUP`, fotocamera, OpenGL (finché non ruotiamo).
- **Fare meglio**: fotogramma chiave e bitrate al volo; eventi dei task invece di `dumpsys`;
  un solo processo con più schermi virtuali (oggi un server scrcpy per finestra) [I: da
  valutare, riduce memoria e avvio ma un errore fa cadere tutte le finestre].
- Licenza: prendere **idee** e nomi di API (fatti), scrivere codice nostro; dove si copia una
  porzione, citarla (Apache 2.0).

---

## 6. Rischi

1. **API nascoste che cambiano firma**: già successo per `requestDisplayPower`,
   `freezeDisplayRotation`, `captureDisplay` (`ScreenCaptureInternal` in A16 qpr2),
   `IDisplayWindowListener` (Samsung). Mitigazione: riflessione con ricerca per nome e
   ripieghi, test all'avvio che registra quali varianti sono state trovate, `onTransact`
   tollerante.
2. **Differenze OEM (Samsung)**:
   - gesto «indietro» bloccato fino al riavvio dopo blocco/sblocco con uno schermo virtuale
     `ALWAYS_UNLOCKED` (S23+ e S26 Ultra, One UI 8/8.5; diagnosi: `SysUiState` con
     `navbar_gone` e `backGestureDisabled=true`, forse legato alla barra di navigazione che
     Samsung crea per lo schermo virtuale) [S, [#6007](https://github.com/Genymobile/scrcpy/issues/6007)].
     **È sui nostri telefoni di prova** e Phonestra blocca il telefono all'uscita:
     rischio concreto;
   - DeX/desktop sugli schermi con decorazioni; IME Samsung e politica `LOCAL`;
   - `DisplayManagerGlobal` che chiede la configurazione (già risolto).
3. **Android 16 e grandi schermi**: app solo verticali allungate su sw ≥ 600 dp (§4.2).
4. **Codificatori**: istanze limitate (più finestre → errori), parametri ignorati da alcuni
   Codec2, cambi di bitrate al volo non rispettati [I].
5. **Pannello**: spegnimento non noto a `PowerManager` → si riaccende da solo in certi casi,
   touch attivo; se il componente muore senza pulizia il pannello resta spento (serve il
   processo di pulizia).
6. **Permessi della shell modificati dall'OEM** o dalla «Protezione avanzata» [I]: verificarli
   a ogni avvio (vedi prove).

---

## Domande aperte

1. Senza `ALWAYS_UNLOCKED` (e senza decorazioni) il difetto del gesto «indietro» sparisce sui
   nostri Samsung? E cosa si vede sul PC quando il telefono si blocca?
2. `containsSecureLayers()` è vero anche quando le finestre protette vengono solo annerite?
3. `REQUEST_SYNC_FRAME` sui codificatori Qualcomm (S23+) ed Exynos/Qualcomm (S26): quanto
   tarda l'IDR? `PREPEND_HEADER_TO_SYNC_FRAMES` è accettato?
4. `PARAMETER_KEY_VIDEO_BITRATE` al volo è rispettato in VBR o serve CBR?
5. Quante istanze H.265/H.264 hardware in parallelo reggono S23+ e S26 alla misura tipica
   (es. 1120×1992)? Cosa dicono `getMaxSupportedInstances()` e i `PerformancePoint`?
6. `setRequestedRefreshRate(60)` riduce davvero il lavoro rispetto a `max-fps-to-encoder`?
7. Con `OWN_FOCUS` + `STEAL_TOP_FOCUS_DISABLED` i tasti iniettati arrivano ancora all'app sullo
   schermo virtuale, senza togliere il fuoco al telefono?
8. `IME HIDE`: i campi di testo ricevono i tasti e il testo «incollato» come oggi?
9. Un processo unico per tutte le finestre o uno per finestra?
10. Quale SoC ha l'S26 di prova (Exynos 2600 o Snapdragon) e quali codificatori elenca (AV1?)?
11. Android 16: con una finestra grande (sw ≥ 600 dp) Facebook resta verticale o si allunga
    (dipende dal targetSdk dell'app)?

## Prove da fare sul telefono

Brevi, con `phonestra-prova` o con un piccolo jar di prova; a fine prova chiudere schermi
virtuali, riaccendere il pannello, ripristinare le impostazioni.

1. **Permessi della shell**: `dumpsys package com.android.shell | grep -E
   "ADD_TRUSTED_DISPLAY|ADD_ALWAYS_UNLOCKED|CAPTURE_VIDEO_OUTPUT|READ_FRAME_BUFFER|MANAGE_ACTIVITY_TASKS|REMOVE_TASKS"`
   → tutti `granted=true` (S23+, S26).
2. **Schermo virtuale minimo**: jar che crea lo schermo con i flag proposti (senza
   `ALWAYS_UNLOCKED`) su un `ImageReader`, avvia l'Orologio, salva un PNG, chiude. Controllare
   in `dumpsys display` i flag effettivi (A16 può toglierne alcuni).
3. **Gesto «indietro» (#6007)**: con e senza `ALWAYS_UNLOCKED`: schermo virtuale aperto →
   blocca → sblocca → controlla `dumpsys activity service com.android.systemui/.SystemUIService
   | grep -A4 'SysUiState state:'` (nessun `navbar_gone`/`backGestureDisabled=true`) e prova il
   gesto. **Attenzione**: se resta bloccato serve un riavvio del telefono: chiedere prima
   all'utente.
4. **Link da un'app**: dall'app sullo schermo virtuale aprire un link e una condivisione:
   restano sullo schermo virtuale?
5. **Fotogramma chiave**: codificatore H.265 attivo, `REQUEST_SYNC_FRAME` ogni 2 s: misurare il
   ritardo tra richiesta e primo IDR; poi con `PREPEND_HEADER_TO_SYNC_FRAMES=1`.
6. **Bitrate al volo**: 8 → 2 → 8 Mbit/s su YouTube: la dimensione dei pacchetti segue entro 1 s?
7. **Sospensione**: `PARAMETER_KEY_SUSPEND` 10 s su YouTube, poi ripresa + IDR: nessun errore,
   CPU/temperatura (`dumpsys thermalservice`) più bassa.
8. **Istanze**: aprire 1, 2, 3, 4… schermi+codificatori a 1120×1992 finché fallisce; confrontare
   con `getMaxSupportedInstances()`.
9. **Refresh**: stesso banco di §20 (testufo) con `setRequestedRefreshRate(60)` e senza.
10. **FLAG_SECURE**: Bitwarden sullo schermo virtuale, `IWindowManager.captureDisplay()` →
    stampa `containsSecureLayers()`; poi un'app normale (deve dare falso).
11. **Eventi dei task**: `registerTaskStackListener`, aprire Facebook (solo verticale) e YouTube:
    arrivano `onTaskRequestedOrientationChanged` con `SCREEN_ORIENTATION_PORTRAIT`? Aprire la
    stessa app sul telefono: arriva `onTaskDisplayChanged`?
12. **Pannello**: `setDisplayPowerMode(OFF)` via `DisplayControl` su A16/One UI 8.5, notifica di
    prova, tasto di accensione, morte del processo con `kill -9` → il processo di pulizia
    riaccende?
13. **IME HIDE**: campo di testo in un'app sullo schermo virtuale, tasti dal PC e «incolla»:
    funzionano e la tastiera Samsung non compare sul telefono.
14. **Elenco codificatori**: nomi, profili, allineamenti, AV1 sì/no, su S23+ e S26.
