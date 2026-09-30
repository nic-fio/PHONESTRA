# Studio: INPUT dal PC al telefono (Android 14–16), per il componente di Phonestra

*28 set 2026. Fonti lette: codice di scrcpy (`master` al commit
[`19c1261`](https://github.com/Genymobile/scrcpy/tree/19c1261d2e2cbf2b5e6a71a8b64cc1dd3ede06ac),
5 set 2026, stessa linea di 4.1), sorgenti AOSP di Android 14 e 15 (mirror
[`aosp-mirror`](https://github.com/aosp-mirror/platform_frameworks_base), rami
`android14-release` / `android15-release`), Android 16 (ramo `lineage-23.0` di
[LineageOS](https://github.com/LineageOS/android_frameworks_base), basato su AOSP 16:
android.googlesource.com rispondeva spesso 503), issue di scrcpy. Nel testo:
**[V]** = verificato leggendo il codice/la documentazione citata; **[I]** = ipotesi o
dedotto, da provare sul telefono. Nulla è stato ancora provato sul S23+.*

## Riepilogo e raccomandazioni

1. **Tocchi, clic, rotellina, tasti speciali: si rifà quello che fa scrcpy**, con
   `InputManager.injectInputEvent` (la shell ha `INJECT_EVENTS` su 14, 15, 16 [V]) e
   `InputEvent.setDisplayId(id)` per mandare l'evento allo schermo virtuale giusto [V].
   Modalità **ASYNC** per tutto; `WAIT_FOR_FINISH` solo prima di leggere gli appunti dopo un «copia».
2. I tasti arrivano all'app di **ogni** finestra solo perché gli schermi virtuali hanno
   `OWN_FOCUS`, che Android accetta solo con `TRUSTED` (la shell ha `ADD_TRUSTED_DISPLAY`
   anche su 16 [V]). Da conservare nel pezzo «video/finestre».
3. Clic sinistro = **dito** (non mouse): trascinare scorre invece di selezionare; clic
   destro = pressione lunga del dito (scelta attuale, giusta). Mouse vero solo per hover e rotellina.
4. **Testo**: tenere il testo composto dal PC; per ASCII tasti della mappa virtuale; per
   le accentate **provare prima** la scomposizione in tasto morto (come `KeyComposition`
   di scrcpy) e tenere «appunti + incolla» come ripiego (sovrascrive gli appunti e può
   far comparire l'avviso «X ha incollato dagli appunti»).
5. **Tastiera UHID** (`/dev/uhid`, la shell è nel gruppo `uhid` e SELinux lo consente [V]):
   è la strada giusta per giochi, scorciatoie, tasti tenuti premuti e layout italiano
   completo. Due novità utili: il **codice paese HID 14 = italiano** fa scegliere il layout
   da solo (Android 14+ [V], serve il kernel [I]), e **associare la tastiera allo schermo
   virtuale** (`addUniqueIdAssociationByPort`, 15+) dovrebbe mandare i tasti alla finestra giusta [I].
6. Da non fare subito: mouse UHID (puntatore «catturato», va contro le finestre del PC),
   IME proprio (andrebbe installata un'app), disattivare il touch del telefono (la shell
   non ha `DISABLE_INPUT_DEVICE` [V]).
7. **Appunti**: `ClipboardManager` con contesto finto `com.android.shell` (+ il trucco
   `mContext` anche per il servizio Samsung `semclipboard`), Looper principale in
   esecuzione per il listener; la shell legge in background grazie a
   `READ_CLIPBOARD_IN_BACKGROUND` [V]. Leggere prima la **descrizione** (non mostra l'avviso
   e dice se è sensibile), poi il testo.
8. **Controller da gioco UHID**: fattibile (scrcpy lo fa: finto Xbox 360); va letto il
   controller sul PC. Priorità bassa.
9. Ordine consigliato: (a) iniezione SDK completa = parità con scrcpy; (b) appunti;
   (c) prove su accentate; (d) UHID tastiera come opzione «tastiera fisica»; (e) gamepad.
10. Rischi principali: Samsung (appunti `semclipboard`, tastiera a schermo che ignora
    l'impostazione su One UI 8, UHID a pannello spento), AltGr con UHID, fuoco dei tasti
    con più finestre.

---

## 1. Iniezione di eventi

### 1.1 API e permesso

- **Chiamata**: `android.hardware.input.InputManager#injectInputEvent(InputEvent, int mode)`
  (nascosta, `@hide`). Da Android 14 `InputManager` delega a
  `InputManagerGlobal.getInstance().injectInputEvent(...)`; esiste anche la variante con
  `targetUid` (limita l'evento alle finestre di un uid) [V:
  [InputManagerGlobal 14](https://github.com/aosp-mirror/platform_frameworks_base/blob/android14-release/core/java/android/hardware/input/InputManagerGlobal.java),
  [16](https://github.com/LineageOS/android_frameworks_base/blob/lineage-23.0/core/java/android/hardware/input/InputManagerGlobal.java)].
  `InputManager.getInstance()` esiste ancora in 14–16 [V]; scrcpy però ottiene il servizio
  con `FakeContext.get().getSystemService(INPUT_SERVICE)`
  ([wrappers/InputManager.java](https://github.com/Genymobile/scrcpy/blob/19c1261d2e2cbf2b5e6a71a8b64cc1dd3ede06ac/server/src/main/java/com/genymobile/scrcpy/wrappers/InputManager.java)).
- **Permesso**: `InputManagerService.injectInputEventToTarget` controlla solo
  `INJECT_EVENTS` (anche della «sorgente di strumentazione»), identico in 14, 15, 16 [V].
  La shell lo dichiara nel suo manifest in tutte e tre le versioni [V:
  [Shell/AndroidManifest.xml 16](https://github.com/LineageOS/android_frameworks_base/blob/lineage-23.0/packages/Shell/AndroidManifest.xml)].
  Commento nel codice: «having the permission already means you can inject into any window».
  Xiaomi toglie il permesso finché non si attiva «Debug USB (impostazioni di sicurezza)»
  ([README scrcpy](https://github.com/Genymobile/scrcpy#prerequisites)) — già gestito da Phonestra.
- **Modalità** (`InputEventInjectionSync`) [V]:
  - `0 ASYNC`: ritorna subito; `false` solo per errori immediati. **Da usare per tutto.**
  - `1 WAIT_FOR_RESULT`: aspetta che il dispatcher trovi un destinatario; utile per sapere se
    un tasto è «arrivato» (es. il tasto innocuo del controllo Xiaomi).
  - `2 WAIT_FOR_FINISH`: aspetta che l'app abbia finito di gestirlo, fino a **30 s**
    (`INJECTION_TIMEOUT_MILLIS`) [V]. scrcpy lo usa solo per `KEYCODE_COPY`/`CUT` prima di
    leggere gli appunti. Mai sul thread che legge i comandi (bloccherebbe tutto).
- **Ripetizione**: l'iniezione passa sempre `FLAG_DISABLE_KEY_REPEAT` [V]: Android non genera
  da solo la ripetizione dei tasti iniettati, deve farla il PC (GTK manda già le ripetizioni;
  conviene passarle con `repeatCount` crescente, oggi Phonestra manda sempre 0).
- **Dispositivo**: il dispatcher sostituisce il deviceId con `VIRTUAL_KEYBOARD_ID` (−1) per
  tutti gli eventi iniettati [V: `InputDispatcher::injectInputEvent`, Android 14]. Quindi le
  app vedono sempre la mappa caratteri **virtuale** (americana), mai un layout italiano.
- **API nascoste**: `setDisplayId` è `@hide @TestApi`, `MotionEvent.setActionButton` `@hide`.
  scrcpy le chiama per riflessione e funzionano da `app_process` [V: lo fa da anni]; con gli
  stub già usati da `android/helper` si possono anche chiamare direttamente [I: da provare,
  la riflessione resta il ripiego].
- Gli eventi iniettati **non sono «verificabili»** (`InputManager.verifyInputEvent` → `null`,
  HMAC non valido) [V: architettura]. Qualche app anti-trucco potrebbe ignorarli [I, raro].

### 1.2 Indirizzare uno schermo specifico

- `InputEvent.setDisplayId(displayId)` prima dell'iniezione (scrcpy: `Device.injectEvent`,
  solo se `displayId != 0`) [V].
- **Tocchi/mouse**: vanno alla finestra sotto le coordinate *di quello schermo*. Senza
  displayId un evento di puntatore va allo schermo 0 [V: `InputDispatcher.cpp`, riga ~4583].
- **Tasti**: `getTargetDisplayId()` usa il displayId dell'evento se c'è, altrimenti lo schermo
  che ha il fuoco globale [V]. Poi serve una **finestra con fuoco su quello schermo**: in
  `DisplayContent.findFocusedWindowIfNeeded` uno schermo che non è in cima ha una finestra
  col fuoco solo se `hasOwnFocus()` = `config_perDisplayFocusEnabled` (falso sui telefoni) o
  flag `FLAG_OWN_FOCUS` [V]. `VirtualDisplayAdapter` accetta `VIRTUAL_DISPLAY_FLAG_OWN_FOCUS`
  **solo insieme a `TRUSTED`**, altrimenti lo ignora con un avviso nel log [V].
  scrcpy crea gli schermi con `TRUSTED | OWN_DISPLAY_GROUP | ALWAYS_UNLOCKED |
  TOUCH_FEEDBACK_DISABLED | OWN_FOCUS | DEVICE_DISPLAY_GROUP` da 14 in su
  ([NewDisplayCapture.java](https://github.com/Genymobile/scrcpy/blob/19c1261d2e2cbf2b5e6a71a8b64cc1dd3ede06ac/server/src/main/java/com/genymobile/scrcpy/video/NewDisplayCapture.java)).
  **È questo che fa funzionare i tasti in più finestre insieme.** Nota: nel ramo `main` di
  AOSP di marzo 2025 la shell aveva perso `ADD_TRUSTED_DISPLAY`/`CREATE_VIRTUAL_DEVICE`, ma
  in AOSP 16 finale sono di nuovo presenti (spostati in fondo al manifest) [V]. Da
  controllare comunque sul S23+/S26 (vedi prove).
- Esiste anche `VIRTUAL_DISPLAY_FLAG_STEAL_TOP_FOCUS_DISABLED` (da 14, richiede TRUSTED +
  OWN_FOCUS) [V]: impedisce che un tocco sullo schermo virtuale rubi il fuoco «globale».
  Utile se si vuole che il telefono in mano non perda il fuoco; **dannoso** se si userà la
  tastiera UHID (che scrive nello schermo col fuoco globale, §4.5).

### 1.3 Cambiamenti 14 → 16 rilevanti

| Tema | 14 | 15 | 16 |
|---|---|---|---|
| `injectInputEvent` + `INJECT_EVENTS` | sì | sì | sì [V] |
| `InputManagerGlobal` | introdotto | sì | sì |
| Associare un dispositivo di input a uno schermo | `addUniqueIdAssociation(port, uniqueId)` | rinominato `addUniqueIdAssociationByPort`, + `…ByDescriptor` | come 15 [V] |
| Permesso `ASSOCIATE_INPUT_DEVICE_TO_DISPLAY` alla shell | sì | sì | sì [V] |
| Un puntatore del mouse per schermo (PointerChoreographer) | no | sì | sì ([PR scrcpy #6009](https://github.com/Genymobile/scrcpy/pull/6009)) |
| `setPointerIcon(icon, displayId, deviceId, pointerId, token)` | no | sì | sì [V] |

Nessuna nuova restrizione all'iniezione trovata in 15 e 16 [V per il codice AOSP; One UI
potrebbe aggiungerne, I].

## 2. Tocchi

### 2.1 Costruzione degli eventi (come scrcpy, [Controller.injectTouch](https://github.com/Genymobile/scrcpy/blob/19c1261d2e2cbf2b5e6a71a8b64cc1dd3ede06ac/server/src/main/java/com/genymobile/scrcpy/control/Controller.java))

- `MotionEvent.obtain(downTime, eventTime, action, pointerCount, PointerProperties[],
  PointerCoords[], metaState, buttonState, 1f, 1f, deviceId=0, edgeFlags=0, source, flags=0)`
  ([MotionEvent](https://developer.android.com/reference/android/view/MotionEvent)).
- Stato dei puntatori (`PointersState`, max 10): ogni dito ha un id locale 0–9 (0 riservato
  al mouse); `PointerProperties.id` = id locale, `toolType = TOOL_TYPE_FINGER`;
  `PointerCoords.x/y` in pixel dello schermo di destinazione, `pressure` 0–1 (scrcpy manda 1
  in giù/movimento, 0 in su), `size = 0`, `orientation = 0`.
- Con più dita: il primo è `ACTION_DOWN`, gli altri `ACTION_POINTER_DOWN | (indice <<
  ACTION_POINTER_INDEX_SHIFT)`; al contrario `ACTION_POINTER_UP` e l'ultimo `ACTION_UP`; ogni
  evento porta **tutte** le dita ancora appoggiate [V]. Un movimento di più dita va mandato come
  **un solo** `ACTION_MOVE` con tutte le coordinate: oggi Phonestra manda un messaggio per
  dito e scrcpy genera un MOVE per ciascuno — funziona, ma nel componente nostro conviene un
  messaggio «dita» unico (meno eventi, pizzico più liscio) [I].
- `downTime` deve restare quello del primo `DOWN` del gesto (scrcpy: `lastTouchDown`).
- `source = SOURCE_TOUCHSCREEN`, `buttonState = 0` («Buttons must not be set for touch events»).

### 2.2 Coordinate, scala, rotazione

- Con lo schermo virtuale di Phonestra (1 punto del PC = 1 dp; pixel dello schermo =
  punti × moltiplicatore) la conversione è una scala: `x_tel = x_fin × L_schermo / L_immagine`.
  scrcpy manda sempre la dimensione dell'immagine insieme alle coordinate e **scarta** gli
  eventi calcolati su una dimensione vecchia (`PositionMapper.map` → `null`) [V]: da tenere,
  perché durante un ridimensionamento arrivano eventi con la misura precedente.
- **Rotazione**: sugli schermi virtuali Phonestra imposta `set-ignore-orientation-request`,
  quindi lo schermo non ruota da solo; se ruota (comando «Ruota») cambia larghezza/altezza
  dello schermo e basta scalare con la nuova misura [I: la rotazione del contenuto la gestisce
  WindowManager, le coordinate iniettate sono sempre nello spazio logico dello schermo].
  Per lo schermo principale scrcpy compone la matrice di rotazione (`displayTransform`) [V].
- La pressione a pannello spento e la gestione del «tocco in tasca» non sono influenzate
  dall'iniezione.

## 3. Mouse

- **Dito o mouse?** scrcpy tratta un clic del puntatore −1 come **dito** se è solo il tasto
  primario, e come **mouse** (`SOURCE_MOUSE`, `TOOL_TYPE_MOUSE`) solo per hover o tasti
  secondari [V]. Motivo: con `SOURCE_MOUSE` + tasto primario il trascinamento **seleziona**
  (TextView, Chrome) invece di scorrere, e la pressione lunga non apre il menu (lo nota già
  il codice di Phonestra, `dito()`). Conclusione: tenere così.
- **Sequenza corretta per il mouse** (altrimenti Chrome non funziona,
  [#3635](https://github.com/Genymobile/scrcpy/issues/3635)) [V]: primo tasto →
  `ACTION_DOWN` + `ACTION_BUTTON_PRESS` (con `setActionButton`); ogni altro tasto →
  `ACTION_BUTTON_PRESS`; ogni rilascio → `ACTION_BUTTON_RELEASE`; ultimo rilascio → anche
  `ACTION_UP`. `buttonState` = maschera `BUTTON_PRIMARY(1)/SECONDARY(2)/TERTIARY(4)/BACK(8)/FORWARD(16)`.
- **Clic destro vero** (`BUTTON_SECONDARY`): arriva come `performContextClick`; poche app lo
  usano (Chrome sì) [I]. Phonestra usa la pressione lunga del dito: resta la scelta migliore
  per un utente non tecnico.
- **Hover** (`ACTION_HOVER_MOVE`, `SOURCE_MOUSE`): dà effetti al passaggio, suggerimenti,
  cursore «mano» nei siti. Costa un evento per movimento; scrcpy lo manda di serie
  (`--no-mouse-hover` per toglierlo). Proposta: mandarlo con frequenza limitata (es. 60/s) [I].
- **Rotellina**: `ACTION_SCROLL`, `SOURCE_MOUSE`, un solo puntatore alle coordinate del
  cursore, `AXIS_VSCROLL` (positivo = su) e `AXIS_HSCROLL` (positivo = destra) [V]. I valori
  sono `float`: 1.0 = uno scatto; frazioni accettate, quindi lo **scorrimento ad alta
  risoluzione del touchpad** si passa così com'è (oggi Phonestra lo converte in virgola fissa
  ±16 per il formato di scrcpy; nel nostro componente basta un `float`). Le app moltiplicano
  per `ViewConfiguration.getScaledVerticalScrollFactor()` [V: API pubblica].
- **Puntatore visibile**: gli eventi iniettati vanno dritti a `InputDispatcher`, saltando
  `InputReader` e `PointerChoreographer`, quindi **nessun cursore** compare sul telefono [I
  forte, dall'architettura]. Va bene: il cursore è quello del PC.
- **Zoom col pizzico**: si continua con due dita simulate. Android 14+ conosce i gesti del
  touchpad (`CLASSIFICATION_PINCH`, `AXIS_GESTURE_PINCH_SCALE_FACTOR`), ma la classificazione
  non si imposta dagli eventi iniettati con le API pubbliche [I]; non conviene.
- **Mouse UHID** (relativo, puntatore di Android): richiede di «catturare» il mouse del PC
  ([doc/mouse.md](https://github.com/Genymobile/scrcpy/blob/master/doc/mouse.md)); su 15+ il
  puntatore può stare sullo schermo virtuale (`addUniqueIdAssociationByPort`), su 14 no
  ([PR #6009](https://github.com/Genymobile/scrcpy/pull/6009)). Solo per giochi, eventualmente.

## 4. Tastiera

### 4.1 KeyEvent iniettati (modalità «SDK»)

- `new KeyEvent(now, now, action, keyCode, repeat, metaState, KeyCharacterMap.VIRTUAL_KEYBOARD,
  0, 0, InputDevice.SOURCE_KEYBOARD)` + `setDisplayId` [V: `Device.injectKeyEvent`].
- **Modificatori**: le app leggono `metaState` dell'evento (`isCtrlPressed()`…); basta
  `META_CTRL_ON | META_CTRL_LEFT_ON` sul tasto C per Ctrl+C, senza mandare il tasto Ctrl a
  parte [V: così fa scrcpy e funziona in Phonestra]. Per i giochi (WASD, Shift tenuto) servono
  invece giù/su separati anche dei modificatori.
- **Scorciatoie di sistema**: Android gestisce da sé alcune combinazioni (Alt+Tab →
  selettore delle app recenti, Meta+… da 15 in `KeyGestureController`) [I: da AOSP, One UI
  può differire]. Il PC comunque si tiene Alt+Tab; non vanno inoltrati Super/Meta.
- **Esc = indietro**: meglio `KEYCODE_BACK` diretto (come `BACK_OR_SCREEN_ON`). scrcpy su 14+
  controlla prima se lo schermo è acceso (`isScreenOn(displayId)`) e altrimenti preme POWER:
  sullo schermo virtuale non serve.
- **Tastiera a schermo**: con tasti iniettati Android non vede nessuna tastiera fisica, quindi
  quando un campo prende il fuoco **l'IME si apre** — sugli schermi virtuali senza decorazioni
  la politica predefinita è `FALLBACK_DISPLAY`: compare sullo **schermo principale** (il
  pannello spento) [V: [doc/virtual-display.md](https://github.com/Genymobile/scrcpy/blob/master/doc/virtual-display.md)].
  Per questo in Phonestra «non compare». Effetti collaterali probabili: tastiera aperta sul
  telefono quando l'utente lo riprende in mano; i tasti fisici passano comunque dall'IME
  (Samsung Keyboard) prima dell'app [I]. Rimedio disponibile:
  `IWindowManager.setDisplayImePolicy(displayId, DISPLAY_IME_POLICY_HIDE=2)` sugli schermi
  virtuali (la shell ha `INTERNAL_SYSTEM_WINDOW` [V]; scrcpy `--display-ime-policy=hide`),
  da ripristinare non serve perché lo schermo muore con la sessione.

### 4.2 Testo e caratteri accentati

- **Mappa virtuale**: `KeyCharacterMap.load(VIRTUAL_KEYBOARD).getEvents(char[])` restituisce la
  sequenza di tasti (con Shift ecc.) per i caratteri della mappa `Virtual.kcm` (tipo FULL,
  layout USA) o `null` [V: [KeyCharacterMap.getEvents](https://developer.android.com/reference/android/view/KeyCharacterMap#getEvents(char[]))].
- **Accentate**: `Virtual.kcm` contiene **tasti morti** con Alt: Alt+` = grave (U+0300),
  Alt+E = acuto (U+0301), Alt+Shift+` = tilde, ecc. [V: [Virtual.kcm](https://github.com/aosp-mirror/platform_frameworks_base/blob/android15-release/data/keyboards/Virtual.kcm)].
  scrcpy scompone «à» in «U+0300 a» (`KeyComposition`) e ottiene così i tasti
  [V: [KeyComposition.java](https://github.com/Genymobile/scrcpy/blob/19c1261d2e2cbf2b5e6a71a8b64cc1dd3ede06ac/server/src/main/java/com/genymobile/scrcpy/control/KeyComposition.java)].
  **Ma** la composizione del tasto morto la fa chi riceve: i `KeyListener` delle `EditText`
  classiche sì; IME, WebView/Chrome, Compose probabilmente no → lettera senza accento o
  doppia ([#650](https://github.com/Genymobile/scrcpy/issues/650),
  [#2689](https://github.com/Genymobile/scrcpy/issues/2689)) [I]. Caratteri senza
  scomposizione (š, đ, €, ß…) falliscono: «Could not inject char»
  ([#3734](https://github.com/Genymobile/scrcpy/issues/3734)). Le specifiche di Phonestra
  dicono «in pratica ASCII»: **non risulta provato** se le accentate italiane passino col tasto
  morto nelle app vere; è la prima prova da fare.
- **Appunti + incolla** (scelta attuale): funziona per ogni carattere, ma: sovrascrive gli
  appunti dell'utente; su Samsung riempie la cronologia appunti; quando l'app incolla un clip
  che appartiene alla shell Android può mostrare il toast **«<app> ha incollato dagli appunti»**
  (una volta per clip e per app; esclusi IME, proprietario del clip e chi ha
  `SUPPRESS_CLIPBOARD_ACCESS_NOTIFICATION`) [V: `ClipboardService.showAccessNotificationLocked`].
  Un carattere accentato = un clip nuovo = potenzialmente un toast. Probabile origine del
  «avviso di Android sugli appunti» nelle specifiche (§ problemi aperti) [I].
- **Evento `ACTION_MULTIPLE` con stringa** (`new KeyEvent(time, "à", deviceId, flags)`,
  deprecato): alcune `TextView` lo accettano; Compose/WebView incerti [I]. Da provare solo
  come curiosità.
- **IME dedicato** (tipo «ADB Keyboard», `commitText` diretto): sarebbe la soluzione perfetta
  per il testo, ma richiede di **installare un'app** → contrario al principio di Phonestra.
  Scartato. Dalla shell non si può ottenere l'`InputConnection` dell'app [I forte].
- **Tastiera UHID** (§4.3): con layout italiano sul telefono à è ì ò ù sono tasti veri, nessun
  problema di caratteri.

### 4.3 Tastiera fisica simulata con UHID

- **Come funziona** (scrcpy `UhidManager`) [V]: `Os.open("/dev/uhid", O_RDWR)`; scrive un
  `UHID_CREATE2` (nome ≤128, `phys` ≤64, `uniq` ≤64, `rd_size`, `bus = BUS_VIRTUAL`,
  vendor, product, version, **country**, descrittore HID); poi `UHID_INPUT2` con i report
  (8 byte: modificatori, riservato, 6 tasti); legge `UHID_OUTPUT` (LED Bloc Maiusc/Num) per
  risincronizzare lo stato. Chiudere il descrittore distrugge il dispositivo: **se il processo
  muore, la tastiera sparisce da sola** (niente residui sul telefono).
- **Permessi** [V]: `adbd` dà alla shell il gruppo `AID_UHID` («for using 'hid' command to
  read/write to /dev/uhid», [daemon/main.cpp 14](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/android14-release/daemon/main.cpp));
  SELinux `allow shell uhid_device:chr_file rw_file_perms` (sepolicy 14).
- **Vantaggi**: Android vede una tastiera fisica vera → layout del telefono (italiano), tutti
  i caratteri e i tasti morti, ripetizione gestita da Android, giochi, Ctrl/Alt/Shift tenuti,
  IME in modalità «tastiera fisica» (la tastiera a schermo si nasconde se l'impostazione è su
  «no»), eventi **non iniettati** (verificabili).
- **Layout automatico** [V per Android]: da 14 `EventHub` legge il file `country` del
  dispositivo HID in sysfs e lo traduce in lingua (`14 → "it"`), usata da
  `KeyboardLayoutManager` per scegliere il layout
  ([EventHub.cpp 14](https://android.googlesource.com/platform/frameworks/native/+/refs/heads/android14-release/services/inputflinger/reader/EventHub.cpp));
  in mancanza, Android 14+ usa la lingua della tastiera a schermo attiva
  (`getPhysicalKeyboardHintLanguageTag`). scrcpy mette `country = 0`: **noi possiamo mettere
  14**. Dipende dal kernel che esponga `country` in sysfs e da UHID che copi il campo [I:
  da verificare sul S23+]. In più la shell ha `SET_KEYBOARD_LAYOUT` [V] e potrebbe impostare
  il layout per il nostro dispositivo (resta salvato nelle impostazioni del telefono: meglio
  il codice paese).
- **Descrittore stabile**: impostare nome/vendor/product/`uniq` fissi, così Android ricorda il
  layout scelto per «la tastiera di Phonestra» [I].
- **Limiti noti**:
  - **Fuoco**: gli eventi di una tastiera vera vanno allo schermo col fuoco globale, non alla
    finestra del PC attiva («the whole system still only has one input focus», PR #6009).
    **Però** `KeyboardInputMapper` dà ai tasti il displayId del *viewport associato* al
    dispositivo [V: [KeyboardInputMapper.cpp 16](https://github.com/LineageOS/android_frameworks_native/blob/lineage-23.0/services/inputflinger/reader/mapper/KeyboardInputMapper.cpp)],
    e gli schermi di Phonestra hanno il fuoco proprio: associando la tastiera UHID allo
    schermo virtuale (`phys` = porta, `addUniqueIdAssociationByPort(porta, uniqueId dello
    schermo)`) i tasti dovrebbero andare alla finestra giusta [I, da provare]; con più
    finestre servirebbe una tastiera UHID per finestra o spostare l'associazione quando
    cambia la finestra attiva (Android riconfigura in modo asincrono) [I].
  - **AltGr** con UHID: segnalati problemi (danese, spagnolo:
    [#7050](https://github.com/Genymobile/scrcpy/issues/7050),
    [#4725](https://github.com/Genymobile/scrcpy/issues/4725)); in italiano @ # [ ] passano da
    AltGr → rischio concreto [I].
  - **Samsung**: S22 Ultra/Android 14 con UHID scriveva solo numeri e simboli
    ([#5590](https://github.com/Genymobile/scrcpy/issues/5590), senza risposta); S25/Android 16
    la tastiera a schermo compare anche con l'impostazione su «no»
    ([#6805](https://github.com/Genymobile/scrcpy/issues/6805)); in DeX la tastiera UHID smette
    col pannello spento ([#4747](https://github.com/Genymobile/scrcpy/issues/4747)) —
    Phonestra lavora proprio col pannello spento → da provare subito.
  - Primo collegamento: Android/One UI può mostrare la notifica «Configura tastiera fisica».
  - La tastiera esiste per tutto il telefono: se l'utente riprende il telefono in mano mentre
    è collegata, la tastiera a schermo può non comparire (Android crede ci sia una tastiera
    fisica). Va chiusa quando la finestra perde il fuoco o dopo un po' di inattività [I].
- **AOA** (l'altra via di scrcpy) funziona solo via USB: non adatta a Phonestra (Wi-Fi).
- **Alternativa scartata**: `VirtualDeviceManager` (`VirtualKeyboard`, `VirtualTouchscreen`):
  la shell ha `CREATE_VIRTUAL_DEVICE`, ma serve un'associazione del CompanionDeviceManager a
  nome di un'app [I]; più complicata di UHID senza vantaggi per noi.

### 4.4 Cosa proporre all'utente

Due modi di scrivere, stessa finestra:
- **normale** (predefinito): testo dal PC + tasti speciali iniettati; accentate col tasto morto
  se la prova va bene, altrimenti appunti;
- **«tastiera fisica»** (UHID, layout italiano): per giochi e scorciatoie, e forse come
  predefinito se le prove su Samsung (pannello spento, AltGr, fuoco) vanno bene.

## 5. Controller da gioco e altri dispositivi

- **Gamepad UHID** (scrcpy 3.x, `--gamepad=uhid`) [V: [doc/gamepad.md](https://github.com/Genymobile/scrcpy/blob/master/doc/gamepad.md),
  `app/src/uhid/gamepad_uhid.c`]: descrittore HID «Gamepad» (2 stick, 2 grilletti, croce, 15
  pulsanti), presentato come **Microsoft X-Box 360 Pad (045e:028e)** così Android usa la sua
  mappa `Vendor_045e_Product_028e.kl`. Più controller = più dispositivi UHID. Sul telefono
  basta lo stesso codice della tastiera UHID; **sul PC** va letto il controller (in Rust:
  `gilrs` o evdev) e tradotto nel report. Nessuna vibrazione (servirebbe un driver
  force-feedback nel kernel) [I]. Gli eventi vanno allo schermo col fuoco globale (stesso
  problema del §4.3) [I]. Fattibile, priorità bassa.
- **Penna/tavoletta**: con l'iniezione SDK si può mandare `TOOL_TYPE_STYLUS` con pressione e
  inclinazione (GTK le fornisce) → app di disegno con pressione [I, fattibile].
- **Touchscreen UHID** (digitalizzatore multi-touch associato allo schermo virtuale): darebbe
  tocchi «veri» e verificabili; più complesso, nessun vantaggio pratico oggi [I].
- **Disattivare il touch del telefono a pannello spento** (rischio tocchi in tasca, §5.9 delle
  specifiche): `InputManager.disableInputDevice` richiede `DISABLE_INPUT_DEVICE`, che la shell
  **non ha** [V]. Resta l'idea di un `EVIOCGRAB` su `/dev/input/eventX` (la shell è nel gruppo
  `input`), ma servirebbe codice nativo e SELinux potrebbe vietarlo [I, bassa priorità].

## 6. Appunti

- **Accesso** [V: `ClipboardService.clipboardAccessAllowed`, 14–16]: lettura permessa a chi ha
  `READ_CLIPBOARD_IN_BACKGROUND` — commento «Shell can access the clipboard for testing
  purposes» — e la shell lo ha in 14, 15, 16. Scrittura sempre permessa. Poi conta l'AppOp
  `READ_CLIPBOARD` del pacchetto (`com.android.shell`, uid 2000). A **telefono bloccato**
  `getPrimaryClip` restituisce `null` (`isDeviceLocked`).
- **Contesto**: `ClipboardManager` va creato con un contesto il cui pacchetto sia
  `com.android.shell` (scrcpy `FakeContext`, anche `getAttributionSource()` con uid 2000).
  **Samsung**: il servizio `semclipboard` usa il suo `mContext`; senza sostituirlo `setPrimaryClip`
  va in `SecurityException` «calling package android does not match caller's uid 2000»
  ([#6224](https://github.com/Genymobile/scrcpy/issues/6224)); scrcpy sostituisce `mContext`
  per `clipboard`, `semclipboard` e `activity` [V: [FakeContext.java](https://github.com/Genymobile/scrcpy/blob/19c1261d2e2cbf2b5e6a71a8b64cc1dd3ede06ac/server/src/main/java/com/genymobile/scrcpy/FakeContext.java)].
  Stesso tipo di controllo più severo su Android 16 per altre API
  ([#6523](https://github.com/Genymobile/scrcpy/issues/6523)).
- **Notifica delle modifiche**: `addPrimaryClipChangedListener`; il servizio avvisa solo i
  listener che passano lo stesso controllo di accesso (la shell sì) [V]. Il callback arriva
  sul `Handler` del contesto: serve un **Looper principale in esecuzione** (scrcpy lo ha
  aggiunto con la PR #6009) [V].
- **Evitare rimbalzi e cronologia piena**: scrcpy non riscrive gli appunti se contengono già lo
  stesso testo e ignora la notifica della modifica che sta facendo lui (`isSettingClipboard`)
  [V]. Phonestra fa già qualcosa di simile.
- **Sensibili**: `ClipDescription.EXTRA_IS_SENSITIVE` (`android.content.extra.IS_SENSITIVE`,
  API 33) nelle `extras` della descrizione
  ([ClipDescription](https://developer.android.com/reference/android/content/ClipDescription#EXTRA_IS_SENSITIVE)).
  `getPrimaryClipDescription` **non** mostra l'avviso di accesso (controllo senza `noteOp`,
  nessun toast) [V]: leggere prima la descrizione, poi il testo solo se non sensibile.
  Oggi Phonestra usa un secondo processo (l'aiutante) per il controllo: nel componente unico
  diventa una chiamata.
- **Avviso «Shell ha incollato dagli appunti»**: leggere un clip che non è della shell mostra il
  toast una volta per clip (la shell non ha `SUPPRESS_CLIPBOARD_ACCESS_NOTIFICATION`) [V], se
  l'impostazione sicura `clipboard_show_access_notifications` è attiva. La shell potrebbe
  spegnerla (`WRITE_SECURE_SETTINGS`), ma è un'impostazione del telefono da non lasciare
  modificata (regola del progetto): solo con consenso dell'utente e ripristino al termine,
  come il tempo di spegnimento [I]. One UI potrebbe avere un avviso suo.
- **Scadenza**: gli appunti si svuotano da soli dopo 1 ora (`DEFAULT_CLIPBOARD_TIMEOUT_MILLIS`) [V].
- **Incollare**: `setPrimaryClip` + `KEYCODE_PASTE` iniettato allo schermo della finestra
  (scrcpy) [V]. **Copiare** dal telefono con Ctrl+C: il tasto arriva all'app, la notifica di
  modifica porta il testo al PC.

## 7. Come lo fa scrcpy 4.x (riassunto dei file)

- `control/ControlMessageReader` + `ControlMessage`: messaggi binari; quelli usati da Phonestra:
  `INJECT_KEYCODE(0)`, `INJECT_TEXT(1)`, `INJECT_TOUCH_EVENT(2)`, `INJECT_SCROLL_EVENT(3)`,
  `BACK_OR_SCREEN_ON(4)`, `SET_CLIPBOARD(9)`, `SET_DISPLAY_POWER(10)`, `START_APP(16)`,
  `RESET_VIDEO(17)`, `RESIZE_DISPLAY(21)`; per UHID `UHID_CREATE/INPUT/DESTROY`.
- `control/Controller`: un thread legge i messaggi e inietta tutto in **ASYNC**; due id di
  schermo: quello richiesto (`displayId`) per i tasti e quello virtuale per gli eventi con
  coordinate; con `--new-display` tutto va allo schermo virtuale [V].
  Puntatori speciali: `-1` mouse, `-2` dito generico, `-3` dito virtuale (pizzico di scrcpy).
- `control/PointersState`, `PositionMapper`: stato delle dita e conversione coordinate.
- `control/KeyComposition`: tabella delle accentate → tasto morto + lettera.
- `control/UhidManager`: `/dev/uhid`, `CREATE2`/`INPUT2`/`OUTPUT`, associazione allo schermo
  su 15+ con porta `"scrcpy:" + pid`.
- `wrappers/InputManager`: `injectInputEvent`, `setDisplayId`, `setActionButton`,
  `add/removeUniqueIdAssociationByPort` per riflessione.
- `wrappers/ClipboardManager` + `FakeContext`: appunti (vedi §6).
- **Issue note**: tasti che non arrivano su Xiaomi (permesso), su Honor 14 (problema di
  alimentazione, non input), Chrome e clic (#3635), accentate (#650, #2689, #3734), UHID su
  Samsung (#5590, #4747, #6805), AltGr (#4725, #7050), appunti Samsung (#6224).
- Licenza Apache 2.0: si può copiare il codice citandolo (come già fatto per `ConfigurationController`).

## 8. Rischi

| Rischio | Probabilità | Mitigazione |
|---|---|---|
| One UI toglie `OWN_FOCUS`/`TRUSTED` → tasti solo alla finestra col fuoco globale | bassa (oggi funziona con scrcpy) | controllo `dumpsys display` all'avvio |
| Accentate col tasto morto non funzionano nelle app vere | alta [I] | appunti come ripiego; UHID |
| Toast sugli appunti a ogni accentata | media | UHID; tasto morto |
| UHID non va a pannello spento su Samsung | media [I da #4747] | prova; ripiego SDK |
| AltGr (@ # [ ]) con UHID | media | inviare quei caratteri come testo SDK anche in modalità UHID |
| UHID: tasti allo schermo sbagliato con più finestre | alta senza associazione | associazione per schermo (15+), prova |
| Tastiera a schermo che compare sul telefono (IME `FALLBACK_DISPLAY`) | media | politica IME `HIDE` sugli schermi virtuali |
| `WAIT_FOR_FINISH` che blocca fino a 30 s | se usato male | solo su un thread a parte |
| Samsung `semclipboard` senza trucco `mContext` | certa | copiarlo da scrcpy |

---

## Domande aperte

1. Le accentate italiane col tasto morto (`KeyComposition`) arrivano giuste in Samsung Keyboard
   attiva, WhatsApp, Chrome, Gmail, app in Compose?
2. Sul S23+ (Android 16, One UI 8.5) il kernel espone `country` per i dispositivi UHID, e il
   codice 14 fa scegliere davvero il layout italiano?
3. L'associazione `addUniqueIdAssociationByPort` porta i tasti UHID allo schermo virtuale
   giusto (con `OWN_FOCUS`)? Funziona anche su Android 14 con `addUniqueIdAssociation`?
4. La tastiera UHID funziona col pannello spento (`SET_DISPLAY_POWER` off) sul Samsung?
5. One UI mostra un avviso proprio quando un'app incolla da un clip della shell?
6. Conviene rendere UHID il modo predefinito di scrivere, o solo un'opzione «giochi»?
   (decisione dell'utente dopo le prove)
7. `DISPLAY_IME_POLICY_HIDE` sugli schermi virtuali cambia qualcosa per l'utente (tastiera che
   non si apre più sul telefono) senza effetti sui tasti?
8. Gamepad: l'utente lo vuole davvero? (serve lettura del controller lato PC)

## Prove da fare sul telefono

Con `phonestra-prova` / componente di prova (`app_process`); `adb` di sistema solo per le
diagnosi; a fine prova chiudere server `adb` e sessioni, nessuna impostazione lasciata cambiata.

1. **Permessi della shell** su S23+ e S26:
   `dumpsys package com.android.shell | grep -E "INJECT_EVENTS|READ_CLIPBOARD_IN_BACKGROUND|ADD_TRUSTED_DISPLAY|ASSOCIATE_INPUT_DEVICE_TO_DISPLAY|SET_KEYBOARD_LAYOUT"`.
2. **Schermo virtuale fidato e con fuoco proprio**: con una finestra aperta,
   `dumpsys display | grep -A3 "Display <id>"` → cercare `FLAG_TRUSTED`/`OWN_FOCUS`;
   `dumpsys window | grep -E "mTopFocusedDisplayId|mCurrentFocus"`; nel log nessun
   «Ignoring VIRTUAL_DISPLAY_FLAG_OWN_FOCUS».
3. **Iniezione di base dal componente nostro**: tocco, due dita (pizzico in Maps/Foto),
   rotellina con valori frazionari (0.25) in Chrome e in Impostazioni, hover in Chrome
   (cursore a mano su un link), clic destro mouse vs pressione lunga, `KEYCODE_BACK`,
   Ctrl+C/Ctrl+V/Ctrl+A, frecce, Invio, Tab — su **due finestre aperte insieme**.
4. **Ripetizione**: tenere premuto Backspace e una lettera, con `repeatCount` crescente.
5. **Accentate col tasto morto**: iniettare `getEvents("̀a")` ecc. per à è é ì ò ù in:
   campo di Impostazioni (EditText), WhatsApp, Chrome (casella di ricerca e pagina web), Gmail,
   un'app Compose; con Samsung Keyboard e con Gboard.
6. **Avviso appunti**: incollare 5 accentate di seguito col metodo attuale e guardare se compaiono
   toast; leggere gli appunti dopo una copia da un'app (toast «Shell…»?).
7. **UHID base**: creare una tastiera (descrittore di scrcpy, `country = 14`, nome
   «Phonestra»), poi `dumpsys input` (dispositivo, layout scelto, `KeyboardLayoutInfo`),
   `cat /sys/bus/hid/devices/*/country`, scrivere à è ì ò ù @ # [ ] € in un campo.
8. **UHID a pannello spento**: come 7 ma dopo `SET_DISPLAY_POWER` off.
9. **UHID e fuoco**: due finestre; senza associazione, poi con `addUniqueIdAssociationByPort`
   verso l'`uniqueId` di uno schermo; guardare dove arrivano i tasti.
10. **UHID e tastiera a schermo**: con UHID attiva, prendere il telefono in mano e aprire un campo:
    compare la tastiera Samsung? Chiudendo il descrittore torna tutto come prima?
11. **Politica IME**: `setDisplayImePolicy(id, 2)` su uno schermo virtuale; verificare che sul
    pannello (acceso per la prova) la tastiera non compaia più e che i tasti arrivino ancora.
12. **Appunti dal componente**: listener attivo (Looper), clip sensibile da Samsung Pass /
    gestore password (descrizione con `IS_SENSITIVE`), telefono bloccato (`null` atteso),
    scrittura con testo accentato senza eccezioni `semclipboard`.
