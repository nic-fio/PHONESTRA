# Prove di collegamento — 26 settembre 2026

Telefono: Samsung Galaxy S23+ (SM-S916B, numero di serie R5CT0000000),
Android 16 (SDK 36), One UI 8.5. PC: Debian 13 (trixie), stessa rete Wi-Fi
«CASA» (PC 192.168.1.10, telefono 192.168.1.20). Per le prove si è usato il
programma `adb` del sistema al posto del codice di Phonestra: il protocollo è lo
stesso, i risultati valgono per Phonestra.

## 1. Riconoscere lo stato del telefono via USB

Letto da `/sys/bus/usb/devices/<porta>/<porta>:*/bInterface{Class,SubClass,Protocol}`:

| Stato | Interfacce |
|---|---|
| Debug USB spento | solo `06/01/01` (MTP) |
| Debug USB attivo | `06/01/01`, `02/02/01` + `0a/00/00` (seriale Samsung), **`ff/42/01` (ADB)** |
| Non autorizzato | `adb devices` → `unauthorized` |

## 2. Permessi

`/dev/bus/usb/001/NNN` ha l'ACL `user:utente:rw-`, data dal tag `uaccess` che
la regola di systemd `/usr/lib/udev/rules.d/70-uaccess.rules` mette su ogni
dispositivo con `ID_USB_INTERFACES` contenente `:060101:`. Su questo PC c'è anche
`android-udev-rules`, ma non è quella a dare il permesso.

## 3. Debug wireless attivato dalla shell

```
adb shell settings put global adb_wifi_enabled 1
```

La prima volta su una rete Android apre la conferma «Consentire il debug
wireless su questa rete?» e rimette l'impostazione a 0 finché l'utente non
accetta (visto nel logcat: `AdbDebuggingManager.startConfirmationActivity`).
Dopo l'OK: `adbd: adb wifi started on port 40859`.

## 4. Collegamento Wi-Fi senza QR

Dopo **«Revoca autorizzazioni debug USB»** e una nuova autorizzazione solo via
USB («Consenti sempre»), `adb connect 192.168.1.20:<porta>` è riuscito senza
abbinamento: l'autorizzazione USB vale anche per il Wi-Fi TLS. Il Debug
wireless era rimasto acceso durante la revoca e la rete non ha richiesto nuova
conferma.

## 5. Ricerca in rete

- `adb mdns services`: nessun risultato.
- `prove/mdns-cerca-telefono.py`: `adb-R5CT0000000-aBcDeF`, `192.168.1.20`, porta
  **40477** (la volta prima era 40859: cambia a ogni attivazione).
- Nessun firewall attivo sul PC (ufw, firewalld, nftables spenti).

## 6. Cavo scollegato

La sessione Wi-Fi è rimasta attiva; chiusa e riaperta da zero (ricerca mDNS →
collegamento) senza toccare il telefono.

## 7. Scadenza dell'autorizzazione

Nel logcat: `alwaysAllow = true, authWindow = 604800000` → **7 giorni**. Si
disattiva con «Disattiva timeout autorizzazioni ADB» nelle Opzioni sviluppatore,
cioè `settings get global adb_allowed_connection_time` = `0`. L'utente l'ha
disattivato a mano; poi `settings put global adb_allowed_connection_time 0`
dalla shell (via Wi-Fi) è stato accettato senza errori, quindi Phonestra può
farlo da solo.

## 8. Notifiche (seconda sessione, 11:20)

`dumpsys notification --noredact` restituisce `android.title=String (...)` e
`android.text=String (...)` in chiaro anche per notifiche `vis=PRIVATE`; senza
`--noredact` il testo è nascosto. Prova con una notifica pubblicata dalla shell
(`cmd notification post -S bigtext -t "Phonestra prova" phonestra_prova "..."`):
letta correttamente; `cmd notification cancel` non esiste, la notifica è rimasta
sul telefono. Per rispetto della privacy nelle prove si stampano solo conteggi,
non i testi delle notifiche dell'utente.

## 9. Display virtuale con scrcpy 4.1

`scrcpy --new-display=720x1280/320 --start-app=com.sec.android.app.clockpackage
--no-window --record=...` con `ADB=/usr/bin/adb`: l'Orologio si apre sul display
virtuale (id 26) e si vede a piena risoluzione. Samsung disegna la sua barra
delle applicazioni in basso sul display virtuale: da togliere con
`--no-vd-system-decorations`. Attenzione: `--no-vd-destroy-content` alla fine
sposta l'app sullo schermo del telefono.

## 10. Blocco del telefono → debug chiuso

`input keyevent KEYCODE_SLEEP`; ~15 s dopo, alla comparsa della schermata di
blocco (11:23:14), nel logcat:
`UsbDeviceManager: setEnabledFunctions ... usbDataUnlocked=false`,
`trySetEnabledFunctions: usbFunctions=sec_charging,adb forceRestart=true`,
`adbd: UsbFfs: connection terminated`, poi
`AdbDebuggingManager: adbd_auth domain socket unavailable: Connection refused`.
Il telefono risponde al ping ma la porta TLS è chiusa e mDNS non annuncia
niente. Allo sblocco (11:25:06) nuovo processo `adbd`, porta 45311.
L'interruttore Debug wireless resta acceso. Impostazioni trovate:
`secure advanced_protection_mode=1`, `secure block_usb_lock=1` (e
`rampart_snapshot_adb_*`, residui del Blocco automatico Samsung).
Decisione dell'utente: vedi SPECIFICHE §5.9.

**Correzione successiva (11:49–11:53):** con «Impedisci connessioni USB se
bloccato» (`block_usb_lock`) disattivata dall'utente, al blocco `adbd` si
riavvia subito (`adbwifi started on port 39737`) ma la porta non viene
annunciata via mDNS. Trovata con una scansione TCP 30000–50000 (5 porte aperte,
quella di adb è l'unica su cui `adb connect` dà `device`). Collegati a telefono
bloccato: display virtuale `state OFF`; dopo `KEYCODE_WAKEUP` (sveglio, non
sbloccato) il display si accende ma l'app avviata con `--start-app` finisce sul
Display #0 dietro la schermata di blocco, il video è nero. Quindi la causa del
debug chiuso era `block_usb_lock`, non la Protezione avanzata
(`dumpsys advanced_protection`: `UsbDataAdvancedProtectionHook available: false`).
Nel test col Calendario è stato usato `--start-app=+` (chiusura forzata
dell'app): da evitare, usare app senza dati personali.

## 11. Dimostrazione: telefono sbloccato, pannello spento (12:11–12:22)

`scrcpy --new-display --start-app=clock --turn-screen-off
--screen-off-timeout=1800` con finestra sul desktop dell'utente (Wayland: senza
barra del titolo, manca il plugin libdecor nella build statica).

- Telefono `Awake`, niente schermata di blocco, pannello spento: la finestra
  funziona col mouse (tocchi su `d=31`).
- L'utente ha visto il pannello ancora acceso per un po' dopo l'avvio; poi
  spento.
- Notifica di prova: il telefono vibra, il pannello resta spento.
- Tocchi fisici sullo schermo spento: consegnati alle app del display 0
  (`InputDispatcher: Delivering touch ... d=0`) → il touch resta attivo.
- 12:22:50 `goToSleep ... (double_tap)`: il doppio tocco Samsung sullo schermo
  spento ha addormentato e bloccato il telefono; con `block_usb_lock` attivo il
  debug si è chiuso e scrcpy è terminato.
- `screen_off_timeout` è rimasto a 1800000 (scrcpy non ha potuto ripristinarlo):
  rimesso a mano a 600000, il valore dell'utente.

## 12. Tappa 1: collegamento senza `adb` (pomeriggio)

Prototipo `phonestra-prova` (Rust, libreria `adb_client` 3.2.3 con libusb
compilata da sé, chiave propria in `~/.config/Phonestra/adbkey`, ricerca mDNS
propria). Esiti:

- `usb`: riconosce il telefono e lo stato del debug da sysfs.
- `prepara`: la chiave di Phonestra è nuova, il telefono chiede di autorizzarla.
  **Se «Consenti sempre» non è spuntato**, via cavo funziona ma il Wi-Fi TLS
  viene rifiutato con `received fatal alert: CertificateUnknown`: la chiave non è
  stata registrata. Ora `prepara` verifica il Wi-Fi alla fine e lo spiega.
- Subito dopo l'autorizzazione il primo comando via cavo è fallito una volta con
  `Open session failed: got AUTH in response instead of OKAY`; ai tentativi
  successivi tutto regolare → nuovo tentativo automatico.
- `collega` dopo aver staccato il cavo: il debug si riavvia su una porta nuova;
  la prima ricerca ha restituito ancora la porta vecchia (34833, `Connection
  refused`), la seconda quella nuova (43861). Corretto: si scartano i record mDNS
  con durata 0 («addio») e si rifà la ricerca se la porta rifiuta.
- Risultato finale: `Collegato via Wi-Fi a SM-S916B — batteria 72%`, senza cavo,
  senza `adb`, senza indirizzi.

## Stato lasciato sul telefono

Debug USB e Debug wireless accesi, timeout autorizzazioni disattivato,
questo PC autorizzato.

## 13. Tappa 2: prima app in una finestra GTK4 (sera)

`phonestra-finestra`: Orologio del telefono in una finestra libadwaita, video
H.264 → GStreamer (appsrc → h264parse → decodebin → gtk4paintablesink), clic →
tocchi. **I clic funzionano** (conferma dell'utente). Difetti trovati lungo la
strada:

- il componente manda il nome del dispositivo solo quando **tutti** i canali
  sono aperti: aprire prima video e comandi, poi leggere l'intestazione;
- leggere il video dentro `select!` insieme ai comandi può perdere byte se la
  lettura viene interrotta: il video va letto in un compito dedicato;
- `screen_off_timeout` del server scrcpy va passato in **millisecondi** (il
  valore finisce così com'è in `settings`): passando 1800 il telefono si
  addormentava dopo 1,8 s e, bloccato, ignorava ogni tocco. Il valore
  originale ora è salvato in `telefoni.toml` e ripristinato alla chiusura o al
  collegamento successivo;
- **allo sblocco Samsung riavvia il servizio di debug** su una porta nuova: un
  collegamento aperto sulla porta vecchia appena prima cade. Serve il
  ricollegamento automatico (SPECIFICHE §5.7);
- GTK per impostazione predefinita tiene un solo processo per applicazione: un
  secondo avvio risvegliava il primo (`NON_UNIQUE`);
- la ricerca mDNS a volte non risponde: si prova prima l'ultimo indirizzo buono.

## 14. Ricollegamento automatico (16:18)

`phonestra-finestra` con sessione in un ciclo: se il video si interrompe, un
comando non arriva o il controllo ogni 5 s non risponde entro 5 s, compare la
fascia «Collegamento perso» con «Riconnetti ora» (`AdwBanner`) e si riprova da
soli (2 s, poi fino a 10 s). Prova dell'utente: blocco col tasto di accensione →
`peer closed connection` (adbd riavviato) → sblocco → nuovo indirizzo, nuovo
display virtuale (id 42 → 43), Orologio di nuovo in finestra, **clic
funzionanti**, senza toccare il PC. Il tempo di spegnimento originale resta
quello letto al primo collegamento e viene rimesso a ogni ricollegamento.

## 15. Display virtuale che segue la finestra (16:40)

Server con `flex_display=true` e comando RESIZE_DISPLAY (tipo 21: larghezza e
altezza u16 BE); la densità resta quella iniziale. La finestra misura l'area
del video a ogni fotogramma (tick callback) e manda la nuova misura in pixel
(punti × fattore di scala, pari); densità 160 × fattore di scala, quindi 1
punto del PC = 1 dp del telefono. Il server raggruppa le richieste durante il
trascinamento. Provato dall'utente: verticale, orizzontale (1356×508),
massimizzata e ritorno; l'Orologio si ridispone, nessun errore.

Da capire:
- alla primissima prova il video si è fermato dopo il ridimensionamento a
  1430×690 (sul telefono il display era cambiato davvero); dalla prova
  successiva mai più. Sospetto il decodificatore VA di Intel (avvisi «driver
  bug» con `GST_DEBUG=2`);
- in due avvii di fila il collegamento è caduto pochi secondi dopo l'apertura
  (`peer closed connection`), subito dopo la fine della sessione precedente: il
  ricollegamento automatico l'ha coperto.

## 16. Tappa 3: drawer, più app insieme, tastiera, rotellina (17:30–18:30)

Programma unico `phonestra` (sostituisce `phonestra-finestra`): drawer con le
app del telefono e finestre delle app nello stesso processo, un solo
collegamento condiviso (`src/collegamento.rs`).

- **Aiutante** (`telefono/aiuto`): 40 app in 1,4 s con icone vere. Sul
  Samsung servono `ConfigurationController` (come scrcpy, altrimenti NPE in
  `CompatSandbox`) e un carattere predefinito creato a mano da
  `/system/fonts/Roboto-Regular.ttf` (Font → FontFamily →
  `nativeCreateFromArray`): senza, l'icona del Calendario (disegna la data)
  fa abortire con «gDefaultTypeface == nullptr»;
  `Typeface.loadPreinstalledSystemFontMap()` fallisce fuori dallo zygote.
  Le «icone animate» Samsung danno SecurityException alla shell: Android
  ripiega da solo sull'icona normale.
- **Tempo di spegnimento**: non più affidato al componente scrcpy (con più
  sessioni ognuna ripristinava il valore letto all'avvio, anche quello già
  allungato) ma a un «custode»: canale `exec:` con
  `trap '' HUP TERM PIPE; settings put …; cat; settings put <originale>`.
  **Senza `trap` adbd lo termina e non ripristina** (provato); con `trap`
  ripristina sia alla chiusura del canale sia quando il PC sparisce (processo
  terminato a forza: tornato a 600000 da solo).
- **Pannello**: il componente che termina lo riaccende se l'aveva spento; se
  restano altre sessioni, queste lo rispengono dopo 1,5 s.
- **Zoom sbagliato**: la sessione partiva prima che la finestra si misurasse
  (display 720×1280/320 poi ridimensionato a 400×714 tenendo 320 dpi → app al
  doppio). Ora la sessione aspetta la misura.
- **Tastiera**: testo composto dal PC (INJECT_TEXT), tasti speciali e
  Ctrl+lettera (INJECT_KEYCODE), non ASCII via SET_CLIPBOARD+incolla. Invio in
  Chrome non funzionava: il pulsante «‹» aveva il fuoco e prendeva Invio →
  controllo dei tasti in fase di cattura e pulsante non selezionabile.
- **Rotellina** (INJECT_SCROLL_EVENT, valori in [-16, 16] a virgola fissa):
  posizione presa dal movimento del puntatore sull'immagine (quella
  dell'evento di scorrimento è relativa alla finestra).
- Ogni **sblocco** del telefono riavvia adbd (visto ancora alle 17:44:03,
  `handleUserPresent` + `UsbFfs: connection terminated`): il ricollegamento
  automatico lo copre.

## 17. Barra di stato, notifiche, audio, chiusura delle app (18:00–18:25)

- **Notifiche**: `dumpsys notification --noredact`, solo la sezione
  «Notification List» (7 KB, 180 ms): lettura ogni 3 s insieme al controllo
  del blocco. Scartate le permanenti (ONGOING_EVENT, FOREGROUND_SERVICE) e i
  riepiloghi di gruppo. Nelle prove si stampano solo pacchetto e lunghezze.
- **Batteria e rete**: `dumpsys battery` (status 2 = in carica, 5 = carica),
  `cmd wifi status` → «Wifi is connected to "CASA"», ogni 30 s.
- **Consumo**: dal registro della batteria, 69% → 59% dalle 16:34 alle 18:00
  con Phonestra in uso (≈ 7%/ora, pannello spento, sessioni video attive).
- **Audio**: una sola cattura per telefono, aperta dal collegamento
  (`video=false control=false audio=true audio_codec=opus`); il telefono
  intanto non suona. Primo tentativo con `autoaudiosink sync=false`: audio
  fuori sincrono (ritardo senza limite). Ora coda di ~3 pacchetti con scarto
  dei vecchi e `pulsesink buffer-time=60000`.
- **Un file per ogni avvio del componente** (`phonestra-server-<scid>.jar`):
  il componente cancella il suo file appena partito; con un file unico due
  avvii contemporanei (audio + finestra) si sarebbero pestati.
- **Chiusura della finestra**: senza intervento Samsung sposta l'app del
  display virtuale sullo schermo del telefono e YouTube continuava lì. Ora
  prima di chiudere: `am stack list` → task con `displayId=<display>` →
  `am stack remove <id>` (come scorrerla via dalle recenti, senza arresto
  forzato). Provato: YouTube sparito dal telefono.
- **Orientamento**: YouTube a schermo intero ruotava il display virtuale; al
  ridimensionamento il componente scambiava larghezza e altezza (2560×1002 →
  1002×2560) e il video diventava una striscia. Ora su ogni display:
  `cmd window set-ignore-orientation-request -d <id> true` e
  `cmd window user-rotation -d <id> lock 0`. Provato: YouTube in finestra
  massimizzata con l'impaginazione da tablet.
- **Fluidità** (finestra 2560×1002): 24 fotogrammi/s costanti, ~1 MB/s,
  pause massime 80–130 ms (fotogrammi a coppie o terne): irregolarità di
  consegna, non mancanza di banda.
- Dopo le correzioni di orientamento e audio l'utente trova **audio e video
  sincronizzati** e **nessuno scatto** percepibile: niente margine di
  riproduzione (resta la massima reattività).

## 18. Densità, frecce, audio adattivo (18:30–19:10)

- **Elementi giganti in Facebook** (controllo del volume dei video, tagliato):
  l'attività aveva la configurazione giusta (160 dpi) ma Facebook, aperto
  anche sullo schermo del telefono (450 dpi), disegna alcuni elementi con
  quella densità. Ora il display virtuale ha la **densità del telefono** e i
  pixel in proporzione (1 punto del PC = 1 dp), con lato massimo **2560**: a
  3840×1496 il telefono scendeva a 12–20 fotogrammi/s, a 2560×1000 ne regge
  24. Il PC riduce l'immagine (più nitida).
- Le misure del display vanno **arrotondate a multipli di 8**: il componente
  arrotonda così il display appena creato (2006 → 2008) e scarta **in
  silenzio** (solo messaggio VERBOSE) clic e rotellina con una misura diversa.
- **Frecce**: in modalità tocco Android con le frecce sposta solo la
  selezione (a volte «non prese»). Ora su/giù fanno scorrere di un passo
  finché non si scrive (lettera, Backspace, Canc); dopo un clic o la
  rotellina tornano a scorrere. Pagina su/giù: una schermata. Confermato
  dall'utente.
- **Audio**: coda con scarto → buchi coi video di Facebook (il Wi-Fi trattiene
  tutto per 200–500 ms). Ora ogni pacchetto va all'orario del telefono più un
  **margine adattivo** (parte da 80 ms, +40 ms a ogni ritardo, massimo 300):
  nella prova si è fermato a 200 ms, 0 pacchetti in ritardo, sincronia
  giudicata buona dall'utente.
- **Pause del video**: confrontando orari del telefono e arrivi, le pause di
  150–200 ms hanno sul telefono intervalli di 40–50 ms → è il **Wi-Fi** a
  trattenere i fotogrammi, non il telefono a non produrli. L'utente ora trova
  YouTube e Facebook fluidi; niente margine sul video (massima reattività).
- «Ripetizione automatica video» grigia nel Lettore video Samsung: si esclude
  con «Riproduci video successivo autom.» (comportamento dell'app).

## 19. Appunti e selezione (19:15–19:40)

- Telefono → PC: sessione del componente solo-comandi con
  `clipboard_autosync=true`; a ogni copia l'aiutante controlla se gli appunti
  sono sensibili (`IClipboard.getPrimaryClipDescription` come
  `com.android.shell`, firma variabile tra versioni: argomenti per tipo;
  ~1,2 s). Il componente di scrcpy non filtra le copie sensibili da solo.
- PC → telefono: Ctrl+V / Maiusc+Ins nella finestra; testi con
  `x-kde-passwordManagerHint` non passano (avviso), oltre 200 KB nemmeno.
- Selezionare il testo col mouse era «difficilissimo» (serve la pressione
  lunga): **clic destro = pressione lunga del dito** (puntatore -2, 650 ms).
  Confermato dall'utente: la selezione funziona. Ctrl+C già passava come
  scorciatoia.
- Da provare ancora: copia di una password da Bitwarden (non deve arrivare al PC).
- **Password da Bitwarden**: nella finestra di Phonestra lo schermo è nero
  (FLAG_SECURE, previsto da §7.6: manca ancora il messaggio chiaro). Copiata
  sul telefono: Android **non la segnala** nemmeno al componente in ascolto
  (nessun evento nel registro) → non arriva al PC; resta comunque il controllo
  «sensibile» di Phonestra.
- **Wayland (GNOME 48)**: gli appunti del PC si possono cambiare solo mentre
  una finestra del programma è attiva. Copiando direttamente sul telefono (PC
  su un'altra finestra) la richiesta veniva ignorata. Ora la copia resta in
  attesa e passa appena una finestra di Phonestra torna attiva (anche un clic
  sul drawer). Provato: «finestra attiva: false» → impostati all'attivazione.
  Poi però «Copia» dal menu di Android in una finestra di Phonestra risultava
  «nessuna finestra attiva» anche subito dopo il clic: `is-active` di GTK non
  è affidabile. Regola finale: la copia si mette **subito**; se gli appunti non
  risultano nostri (`is_local` falso) si rimette alla prossima attivazione; una
  copia fatta nel frattempo sul PC la annulla. Confermato dall'utente: il menu
  «Copia» di Android funziona.

## 20. Fluidità dei video: la causa era la cattura audio (19:40–20:15)

L'utente trovava YouTube e Facebook di nuovo a scatti. Misure:
- sul PC nessun fotogramma scartato (`stats` di gtk4paintablesink): il PC
  disegna tutto quello che riceve;
- dal telefono 12–20 fotogrammi/s con intervalli di produzione (pts) di ~80 ms:
  è il telefono a non produrli.

**Banco di prova** `phonestra-prova banco <l> <a> <dpi> <s> [video] [audio]
[controllo]`: display virtuale con testufo.com (animazione continua) o YouTube
«Big Buck Bunny» (canale Blender); conta i fotogrammi prodotti, poi toglie il
task dalle recenti. Risultati (Galaxy S23+):
- testufo: 60 fotogrammi/s a 1120×1992/448 con e senza audio e controllo
  periodico → densità, appunti e controllo non c'entrano;
- YouTube senza audio: 60/s a 1120×1992/448 e a 2560×1000/160;
- YouTube con audio `audio_source=output` (cattura dell'uscita intera, REMOTE
  SUBMIX): **24/s in 4 prove su 5**; con `audio_source=playback`
  (AudioPlaybackCapture, Android 13+): **60/s in 4 prove su 5**. I lettori
  video con l'uscita deviata perdono il riferimento dell'audio e saltano
  fotogrammi.

Ora la sorgente predefinita è **playback** (il telefono resta muto lo stesso).
Limite: le app che vietano la cattura dell'audio non arrivano al PC (§10).
`PHONESTRA_AUDIO=output` per confrontare. Dopo il cambio, in uso reale: 24/s
costanti (contenuto a 24), nessuno scartato; l'utente: «per il momento va
bene così».

Aggiunto anche il messaggio per le schermate protette (FLAG_SECURE `0x2000` in
`dumpsys window windows`, finestre con `mHasSurface=true`): da provare con
Bitwarden. Un avvio si è chiuso da solo dopo 2 s (drawer rimosso prima di
caricare le app): causa sconosciuta, non ripetuto.

## 21. Preferiti e zoom (20:20–20:45)

- Preferiti: clic destro su un'app → «Aggiungi/Togli dai preferiti»; salvati
  in `telefoni.toml` (conservati se si rifà la configurazione). Confermati
  dall'utente.
- Zoom: Android non ha un comando di zoom generale, le app rispondono al
  pizzico. Prima versione: un pizzico completo per scatto (16 messaggi, ognuno
  con la sua attesa di conferma ADB, 150–250 ms) → poco reattivo. Ora pizzico
  continuo e tocchi di un passo in un solo invio: «abbastanza reattivo».
  In Maps il pizzico simulato va meno bene (Maps interpreta anche rotazione e
  inclinazione); Maps zooma già con la rotellina semplice.

## 22. Interfaccia: primo tentativo di drawer «da launcher» (21:30–21:45)

L'utente giudica l'interfaccia «da versione pre-alfa» e vuole sistemarla prima
dell'AppImage. Fatto:
- `PHONESTRA_FOTO=<cartella>`: ogni finestra si salva in PNG dopo 6 s e poi
  ogni 10 s (`src/foto.rs`), così l'aspetto si controlla senza screenshot
  dell'utente;
- drawer rifatto (in corso, **non approvato**): scheda del telefono in alto
  accanto alla ricerca (barra di stato in basso tolta), icone da 56 px senza
  riquadri, larghezza fissa 104 px, evidenziazione al passaggio, titoli di
  sezione in maiuscoletto, sfumatura col colore d'accento, «aggiorna» in un
  menu. Immagini in `memoria/immagini/` (prima / nuovo).

**Difetto aperto**: nel drawer nuovo i testi risultano bianchi su fondo
chiaro, illeggibili (confermato dallo screenshot dell'utente). Da capire: forse
la regola CSS sulla finestra (`window.phonestra-drawer { background: … }`) o
lo stile «trasparente» della barra del titolo; provare senza la sfumatura.

**Sfondo del telefono**: l'aiutante (`sfondo`) fallisce con
«Invalid package or package does not belong to uid:2000», anche con
`createPackageContext("com.android.shell")`. Strade: `IWallpaperManager`
direttamente col pacchetto `com.android.shell`, oppure rinunciare.

## 23. Interfaccia «vetro» nel programma (27 set 2026, mattina)

- `src/cassetto.rs` riscritto sul mockup `mockup/proposte/drawer-vetro*`:
  barra laterale (App, Notifiche con contatore, Telefoni, Strumenti,
  Preferenze, Informazioni), pillola del telefono col suo menu, schede
  Preferiti / Tutte le app, notifiche raggruppate per app con «altre N», pallino
  sotto le app aperte, telefono disegnato con proporzioni vere (ora, Wi-Fi,
  batteria, carica, velo con «Riconnetti ora»). Voci non ancora fatte: visibili
  e spente («In arrivo»), su richiesta dell'utente.
- **Testi bianchi risolti**: colori espliciti per tema chiaro e scuro
  (classe `scuro` dalla `StyleManager`).
- Immagine: `memoria/immagini/drawer-vetro-gtk-2026-09-27.png`.
- `phonestra` ora si chiude bene con SIGTERM / Ctrl+C (chiude le finestre come
  l'utente, il telefono viene rimesso a posto). **Attenzione**: la chiusura
  blocca il telefono (§5.9): ogni riavvio di prova blocca il telefono
  dell'utente. Chiedere prima di riavviare.
- Lezione: confrontare voce per voce con i mockup approvati prima di mostrare,
  invece di andare a tentativi (richiesta dell'utente).

## 24. Tutte le voci del drawer attive (27 set 2026)

Su richiesta dell'utente («attiva le voci mancanti eccetto debug wireless»):
- **Rinomina, Dimentica** (menu della pillola); **Installa app…, Invia file…**
  (e trascinamento sul telefono disegnato, con avanzamento e annullamento);
  **Non disturbare** (`cmd notification set_dnd priority|off`, *da verificare*
  su Android 16); **Informazioni** su Phonestra; menu dell'app **Informazioni
  sull'app** (`am start --display N -a …APPLICATION_DETAILS_SETTINGS`) e
  **Disinstalla…** (solo `pm list packages -3`).
- **Schermo del telefono**: sessione senza `new_display` (schermo principale),
  Home, recenti, tendina, impostazioni rapide, volume, Ruota, Blocca.
- **Preferenze** (`preferenze.toml`): Esc indietro, avvisi a comparsa, solo il
  nome dell'app, app che possono avvisare, cartella dei file inviati, Aggiorna
  ora. **Avvisi a comparsa** via `org.freedesktop.Notifications` (GNotification
  su GNOME richiede un `.desktop`): provato con `gdbus`, accettato.
- **Aggiungi telefono** (`src/procedura.rs`): procedura guidata col cavo che
  avanza da sola, istruzioni per famiglia (`dati/istruzioni.toml`, incorporato),
  passaggio Xiaomi rilevato con `input keyevent 0`. Senza telefoni configurati
  si apre all'avvio. Altri telefoni nella barra laterale: il clic riavvia
  Phonestra su quello (`Telefoni::metti_primo` = l'ultimo usato).
  Prova dell'interfaccia senza cavo: `PHONESTRA_PROVA_PASSO=debug|consenti|xiaomi|wifi|fatto
  phonestra-prova procedura`. Immagine: `immagini/procedura-debug-usb-2026-09-27.png`.
- **Non ancora provato col telefono**: tutto quanto sopra (l'istanza
  dell'utente era aperta). Restano spenti Webcam e Microfono (§12) e il Debug
  wireless alla chiusura.

## 25. Schermo vero nel drawer (27 set 2026, fine mattina)

- **Funziona** (confermato dall'utente): lo schermo principale del telefono
  in diretta nella cornice a destra, interattivo; l'utente ha mandato un
  messaggio WhatsApp e guardato un video di YouTube dallo schermo del drawer.
  Col video: 22–24 fotogrammi/s, 0,8–1,3 MB/s, pause massime ~100 ms.
  Immagine: `immagini/drawer-schermo-vero-2026-09-27.png`.
- Registro: «sbloccato a mano: il pannello resta acceso» dopo una caduta del
  collegamento (regola §5.9 punto 5 attiva). Da verificare con l'utente che al
  primo clic dal PC il pannello si rispenga.
- Nel registro solo avvisi innocui del componente
  (`IDisplayWindowListener.onDisplayAnimationsDisabledChanged`, Android 16).

## 26. Prima AppImage (27 set 2026, durante la pausa dell'utente)

- Contenitore `phonestra-appimage` (`costruzione/Containerfile`): Ubuntu 22.04,
  glib 2.80, wayland 1.22, graphene, GTK 4.14.5, libadwaita 1.5.4 e
  gtk4paintablesink (gst-plugins-rs 0.13.5) in `/opt/phonestra`. Phonestra
  compilato dentro: richiede al massimo GLIBC_2.34.
- `costruzione/raccogli.sh` → `target/appimage/Phonestra-0.1.0-x86_64.AppImage`
  (~100 MB, runtime statico). `costruzione/AppRun` imposta GStreamer, caricatori
  delle immagini, schemi, niente moduli GIO del sistema.
- Problemi trovati e risolti:
  - **libstdc++/libgcc_s inclusi** → i driver del sistema (Mesa, VA-API iHD)
    non partivano (`CXXABI_1.3.15`): ora restano del sistema;
  - **libwayland-client 1.22 inclusa** → Mesa di Debian 13 si fermava
    (`wl_display_create_queue_with_name`): wayland, zlib, zstd, libxml2, libffi,
    libelf, libva, libxcb-* ecc. sono ora **di riserva**: AppRun le usa solo se
    il sistema non le ha (wayland anche se più vecchia della 1.21);
  - **disegnatore «ngl» di GTK 4.14** sbaglia le sfumature coi Mesa meno
    recenti (Debian 12, Ubuntu 22.04): AppRun imposta `GSK_RENDERER=gl`.
- Prova su 4 distribuzioni in contenitori con lo schermo e la scheda grafica
  del PC (`costruzione/prova-distribuzioni.sh`, configurazione vuota → si apre
  «Aggiungi un telefono»): **Ubuntu 22.04, Debian 12, Fedora 43, Arch**, tutte
  corrette. Immagine: `immagini/appimage-4-distribuzioni-2026-09-27.png`.
  Nei contenitori servono `libgles2`/`libglvnd-gles` e i dati di xkb, che i
  desktop veri hanno sempre.
- **Incidente**: la prima prova sul PC è partita con la configurazione vera e
  il telefono, che l'utente pensava irraggiungibile, era raggiungibile: si è
  collegato per meno di un minuto; chiuso subito col SIGTERM (telefono rimesso
  a posto e bloccato). Da allora le prove usano `XDG_CONFIG_HOME` vuoto.
- **Non ancora provati nell'AppImage**: video, audio, cavo USB, avvisi (serve
  il telefono).

## 27. App solo verticali, audio, pausa alla chiusura (27 set 2026, pomeriggio)

- **Finestre ingrandite e app solo verticali** (Facebook). Schermo del PC 2560
  punti; finestra ingrandita alta ~1400 punti → alla densità del telefono (450)
  servirebbero ~3900 pixel, oltre il tetto di 2560: il display veniva rifatto a
  densità 160, poi (prima prova) come colonna a ~290 dpi. In entrambi i casi
  Facebook si disegnava male («il risultato è terribile»). **Decisione
  dell'utente** (insistita, preferita alla mia controproposta di ingrandire
  l'immagine): per le app solo verticali **niente pulsante di ingrandimento**;
  se la finestra viene ingrandita per altre vie (doppio clic, Super+↑) torna
  alla sua misura. Resta come riserva: finestra larga → display a colonna con
  la forma del telefono e la **sua densità** (mai più bassa; il PC ingrandisce).
- **Come si sa che un'app è solo verticale**: `dumpsys activity activities`,
  per display, l'attività in cima al primo gruppo visibile e opaco:
  `requestedOrientation=SCREEN_ORIENTATION_PORTRAIT` (con le bande:
  `areBoundsLetterboxed=true`). Costa ~100 ms al telefono.
- **Errore mio, poi corretto**: messo nel controllo ogni 3 s, faceva arrivare
  in ritardo un pacchetto audio ogni 5–10 s («il video su FB è tornato a fare
  schifo»: audio a scatti e fuori sincrono, perché il margine audio cresce a
  ogni ritardo). Ora lo chiede solo la finestra: 2,5 s e 6,5 s dopo l'avvio e
  quando diventa larga.
- **Audio**: provati e **scartati** (tornati alla versione del mattino) il
  margine che torna a scendere e il margine fisso di 0,3 s (YouTube peggiorava).
  I rari buchi rimasti vengono dal Wi-Fi: telefono a 5 GHz con **−74 dBm**
  (pause della rete di 0,2–0,5 s). Prova da fare: telefono vicino al router.
- **Monitoraggio con l'utente** (16:22–16:38, registro con l'ora, awk
  `mawk -W interactive`): YouTube a tutto schermo 40 s senza difetti; i soli
  rallentamenti sono la ripartenza di 1–2 s al cambio di display (è il
  telefono a non produrre fotogrammi, non la rete). Il tutto schermo **non**
  era la causa dei problemi; l'utente aveva proposto di toglierlo del tutto,
  tenuto per le app che accettano l'orizzontale.
- **Audio dopo la chiusura**: tolto dalle recenti, il lettore di YouTube resta
  vivo e continua a suonare (e, uscendo da Phonestra, dall'altoparlante del
  telefono). Ora: chiudendo la finestra, pausa se l'app è la «Media button
  session»; uscendo da Phonestra, pausa di qualunque riproduzione **prima** di
  staccare l'audio. Provato dall'utente: audio fermo in 1–2 decimi di secondo,
  telefono muto all'uscita.
- Chiudere la finestra principale lascia aperte le finestre delle app (è un
  lanciatore); Phonestra esce con l'ultima finestra. L'utente non ha chiesto di
  cambiarlo.

## 28. AppImage provata col telefono (27 set 2026, 16:43–17:05)

Con la configurazione vera dell'utente e il registro con l'ora:
- **Barra delle finestre con la sola X**: senza moduli GIO, GSettings non
  leggeva le impostazioni del desktop (dconf) e GTK usava quelle predefinite.
  Ora l'AppImage contiene la **sua** copia di `libdconfsettings.so` (dal
  contenitore, dconf 0.40): tornati Riduci a icona e Ingrandisci. Stessa
  causa, stessa cura per caratteri e ridimensionamento del testo.
- **Chiusura presa per caduta** (difetto a caso, anche fuori dall'AppImage):
  la finestra ferma la riproduzione prima che arrivi il comando «chiudi»; il
  compito del video finiva e la sessione risultava «interrotta» → app non tolta
  dalle recenti né messa in pausa, YouTube continuava a suonare. Ora un video
  finito da sé perché la riproduzione è ferma conta come chiusura. Provato
  2 volte su 2.
- **Tolto «Apri in una finestra»** sotto lo schermo del drawer (decisione
  dell'utente: doppione). Tolti con lui i comandi della finestra «Schermo del
  telefono» (Home, recenti, tendina, volume, ruota, blocco).
- Superate: aspetto, YouTube con audio (anche a tutto schermo), chiusura di
  una finestra, avviso di una notifica (mandata con `cmd notification post`:
  l'utente non poteva farsi scrivere su WhatsApp), invio di una foto
  trascinata sullo schermo del drawer.
- Da provare ancora nell'AppImage: il cavo USB (procedura «Aggiungi telefono»).

## 29. App solo verticali a misura fissa (27 set 2026, 17:05–17:12)

L'utente ha allargato Facebook trascinando il bordo: display a colonna con
bande ai lati («stesso problema del full screen»). **Decisione dell'utente**:
per le app solo verticali niente ridimensionamento, bordi compresi. Ora,
appena si sa che l'app è solo verticale (2,5–6,5 s), la finestra diventa non
ridimensionabile alla misura attuale (9:16 se era più larga che alta); GTK
toglie da solo il pulsante di ingrandimento e il doppio clic. La misura la
decide un riquadro vuoto sotto l'immagine (l'immagine chiederebbe i pixel del
telefono). Provato: Facebook fisso; YouTube ancora ingrandibile e allargabile.

## 30. Prima configurazione vera, dall'AppImage (27 set 2026, 17:13)

Prova proposta dall'utente: configurazione di Phonestra tolta (rinominata in
`~/.config/Phonestra.prova-bak`, per poter tornare indietro), AppImage avviata
da zero col Galaxy S23+ e il cavo. Procedura «Aggiungi un telefono» completata
in **53 s** (17:13:10 → 17:14:03), poi drawer, video, audio e appunti subito.
Configurazione nuova giusta (nome, indirizzo, spegnimento originale 10 min).
L'utente: «Eccezionale. Procedura a prova di stupido, così deve funzionare una
procedura guidata!». Da fare: la procedura non scrive i suoi passi nel
registro (con PHONESTRA_DEBUG si vede solo l'inizio e la fine).

## 31. I rari buchi dell'audio non vengono dal Wi-Fi (27 set 2026, 17:24–17:27)

L'utente: router a 50 cm, rete a 5 GHz, «non dipende certo dal wifi». Anche il
PC è in Wi-Fi (`wlo1`). Prova: ping dal PC al telefono 5 volte al secondo per
3 minuti, con YouTube in finestra e l'utente che girava per casa col telefono.
- Ping: 897/897, media 8 ms, 9 sopra 50 ms (max 221) tutti alle 17:24:19–21,
  prima di aprire YouTube: nessun effetto su Phonestra.
- Phonestra: solo due intoppi, 17:24:30 (video trattenuto 328 ms, telefono in
  orario) e 17:24:48 (1 pacchetto audio in ritardo), con ping normale (48 e
  26 ms). Poi oltre 2 minuti senza ritardi, 33–38 fotogrammi/s, pause 50–100 ms.
- **Conclusione**: gli intoppi arrivano nei primi 20–25 s dopo l'apertura di
  un'app o un cambio di misura (display nuovo, app che parte, video che
  carica): il telefono è sotto sforzo e l'invio dei dati resta indietro un
  attimo. Non il Wi-Fi (il −74 dBm letto prima non pesa: collegamento a
  288 Mbit/s). Lasciato così; eventuale cura mirata: margine audio più grande
  solo nei primi 20 s di ogni sessione.

## 32. Primo beta-tester: Debug USB grigio su Galaxy S26 (27 set 2026, sera)

L'amico dell'utente, senza istruzioni, si è fermato al passo 2: la voce
«Debug USB» era grigia. Causa (Samsung One UI 6+): **Blocco automatico** (Auto
Blocker), che blocca Debug USB e wireless. Ora `dati/istruzioni.toml` ha per
ogni famiglia `grigio` = cosa fare se la voce è grigia, mostrato come «La voce è
grigia?» sotto il passo 3 (Samsung: spegnere il Blocco automatico; Xiaomi: SIM
e account Xiaomi; Pixel/Motorola: Protezione avanzata; altri: cercare le due
voci, o telefono aziendale). Solo il caso Samsung è visto davvero.

## 33. «Phonestra abbassa il microfono»? No (27 set 2026, 17:55–18:05)

L'utente, in una videochiamata WhatsApp avviata da Phonestra, era sentito a
volume basso. Prova a coppie proposta dall'utente e precisata: due vocali
WhatsApp alla stessa distanza, A da Phonestra (WA0014) e B dal telefono
(WA0015), scaricati e misurati: volume −19,7 / −18,2 LUFS, bande grave e acuta
simili, picchi a 0 dB (WhatsApp normalizza i vocali); A ha solo un filtro
antirumore più forte (rumore di fondo −94,7 contro −84,8 dB). Riprodotti dalle
casse del PC uno dopo l'altro: l'utente «hanno volume uguale». La differenza
sentita prima veniva dall'ascolto. Phonestra non tocca il microfono (cattura
solo ciò che il telefono suona). Per le chiamate conta il vivavoce sulla
scrivania (microfono lontano, cancellazione dell'eco); prova risolutiva
proposta: chiamata dal telefono, stesso punto, in vivavoce.
Seguito: l'utente aveva poi notato «se sono in chiamata e attivo Phonestra
l'interlocutore mi sente bassissimo»; preparato `phonestra-prova audio <s>`
(solo la cattura dell'audio, per isolarla). **Falso allarme**: era
l'interlocutore ad avere il vivavoce spento. Phonestra non abbassa il microfono.

## 34. Telefono di fabbrica in «Solo ricarica»: non visto al passo 1 (27 set 2026, 18:40)

Il Galaxy S26 del beta-tester, collegato col cavo, lasciava la procedura al
passo 1: in «Solo ricarica» col Debug USB spento il telefono non offre né MTP
né ADB, e `usb::telefoni()` lo scartava. Col nostro S23+ non era mai capitato
(Debug USB già acceso). Ora: `Stato::SoloRicarica` per i dispositivi dei
produttori Android noti (codici USB in `usb.rs`) → passo «Scegli Trasferimento
file»; il passo 1 dice anche cosa fare se non succede niente. Scelto
«Trasferimento file» a mano, anche la versione precedente vede il telefono.
Il beta-tester (a 700 km, «non è in grado di fare indagini così sofisticate»)
restava al passo 1 anche con «Trasferimento file». Proposta dell'utente:
arrendersi; controproposta accettata di fatto: la diagnosi la fa Phonestra.
Dopo 20 s al passo 1 la procedura mostra «Cosa vede il PC»: cosa provare in
ordine (cavo, porta senza hub, Blocco automatico con «Restrizioni massime»,
telefono sbloccato) e l'elenco dei dispositivi USB (`usb::tutti()`, i
produttori Android segnati «← telefono»): basta una foto della finestra.

## 35. Beta-tester: era la presa USB del monitor (27 set 2026, sera)

Il Galaxy S26 fermo al passo 1 anche in «Trasferimento file» era collegato a
una **presa USB del monitor** invece che del PC: il monitor dava solo
corrente. Collegato al PC, l'amico dell'utente si è collegato. Conferma il
controllo «Direttamente al PC» della diagnosi (`guida-diagnosi.html`, che già
cita le prese del monitor) e il valore della strada senza cavo.
Prova di associazione via Wi-Fi interrotta dall'utente a questo punto: il QR
l'ha trovato «complicato» → preferito il **codice di associazione a 6 cifre**
(Phonestra trova da solo indirizzo e porta via mDNS). L'`adb` di sistema non ha
visto annunci mDNS (come in §5.4). Associazione non ancora provata.
Dopo la prova interrotta, l'S23+ dell'utente non si ricollegava: il **Debug
wireless si era spento** (aperta e chiusa la schermata del Debug wireless,
senza associare; causa esatta non vista). Riacceso a mano dall'utente → di
nuovo in rete (`phonestra-prova cerca`: 192.168.1.20:34107). Da ricordare
per la guida: il Debug wireless si spegne facilmente, e senza cavo non si
riaccende dal PC.

## 36. Righe orizzontali nel drawer sul PC del beta-tester (27 set 2026, sera) — in sospeso

Foto dal PC dell'amico (Zorin OS, AppImage beta 5): due righe scure nello
spazio vuoto della barra laterale (fra «Invia file…» e «Preferenze») e una
doppia sotto la griglia delle app, che sborda dalla scheda. Sul PC
dell'utente (stessa AppImage) non ci sono; nel codice non c'è nulla che le
disegni. Riprodotto Ubuntu 22.04 + X11 in un contenitore: la foto
`PHONESTRA_FOTO` del drawer è pulita (ma ridisegna da capo: non mostra
residui dello schermo). Ipotesi: residui del disegnatore «gl» (AppRun) sul
suo sistema. Chiesti all'amico: versione di Zorin (17 = Ubuntu 22.04, 18 =
24.04), se le righe cambiano col mouse o ridimensionando, ridimensionamento
dello schermo. Prova risolutiva: `GSK_RENDERER=ngl`. Nota a margine: nel
contenitore le icone SVG non si caricano («Unrecognized image file format»),
sul PC dell'amico sì: da capire se è solo del contenitore.

## 37. Associazione col codice a 6 cifre: funziona (27 set 2026, 21:28)

Col S23+ dell'utente (SM-S916B, Android 16, **One UI 8.5**: `ro.build.version.oneui`
= 80500, la stessa dell'S26) e l'`adb` di sistema 34.0.5:
- la voce sul telefono: Debug wireless › «Associa dispositivo con codice di
  associazione»;
- la nostra interrogazione mDNS (bit QU, `strumenti/mdns-adb.py`) ha visto
  l'annuncio **`_adb-tls-pairing._tcp`** appena aperta la schermata del codice
  (192.168.1.20:40437), accanto a `_adb-tls-connect` → Phonestra può trovare
  indirizzo e porta da solo, l'utente scrive solo le 6 cifre;
- `adb pair 192.168.1.20:40437 229325` → «Successfully paired»; poi `adb
  connect` alla porta di `_adb-tls-connect` e `getprop` → collegato;
- `cmd statusbar add-tile COMPONENT` esiste; il riquadro
  `com.android.settings/.development.qstile.DevelopmentTiles$WirelessDebugging`
  c'è nel pacchetto (se sia abilitabile dalla shell: da provare);
- Phonestra (AppImage) è rimasto collegato; il Debug wireless è rimasto acceso.
Chiusi `adb` e la ricerca. Sul telefono resta il PC associato con la chiave
dell'`adb` di sistema: l'utente lo toglie da Debug wireless › Dispositivi
associati. Prossimo: SPAKE2 nel nostro ADB.

## 38. Associazione col codice nel nostro ADB: funziona (27 set 2026, notte)

`src/adb/abbina.rs`: il protocollo di `adb pair` riprodotto dai sorgenti di
Android (`pairing_connection`, `pairing_auth`, `aes_128_gcm`) e BoringSSL
(`spake25519`): TLS con la chiave di Phonestra, password = codice + 64 byte
esportati dal TLS («adb-label\0»), SPAKE2 su Ed25519 (curve25519-dalek; la
«correzione della password» di BoringSSL equivale a usare la parte di ordine
primo di M e N), HKDF-SHA256 → AES-128-GCM (ring), PeerInfo da 8192 byte con
la chiave pubblica nel formato di Android. `rete::cerca_abbinamento` trova la
schermata del codice (`_adb-tls-pairing`).
Prova con una **chiave nuova** (`XDG_CONFIG_HOME` vuoto, così il telefono non
la conosceva): `phonestra-prova abbina 292348` → schermata trovata
(192.168.1.20:35701), «Associato: adb-R5CT0000000-aBcDeF», collegamento Wi-Fi
con quella chiave e `getprop` → SM-S916B. **Al primo tentativo.** Il telefono
dell'utente ha ora in «Dispositivi associati» anche questa chiave di prova
(«Phonestra@…»): da togliere a mano.

## 39. Finestra «Prepara il telefono» provata col telefono vero (27 set 2026, 22:00)

`src/prepara.rs`, lanciata con `XDG_CONFIG_HOME` vuoto (chiave nuova,
nessun telefono salvato) mentre l'Phonestra dell'utente restava collegato: la
finestra ha visto da sola il Debug wireless acceso (prime tre voci spuntate,
«✓ visto da Phonestra») e la schermata del codice (campo attivo, cursore già
dentro). L'utente ha scritto le 6 cifre: associato, collegato, autorizzazione
senza scadenza, telefono salvato («Galaxy S23+», SM-S916B, Android 16,
192.168.1.20:33739), «Fatto! Il Galaxy S23+ è collegato via Wi-Fi».
Primo collegamento completo **senza cavo**, al primo tentativo.
Nota: «Il S23+» — l'articolo davanti al nome scelto dall'utente può essere
sbagliato: ora «Fatto! «Galaxy S23+» è collegato via Wi-Fi».

## 40. Prova «utente inesperto»: il beta-tester ci è riuscito (27 set 2026, sera)

L'amico dell'utente (Galaxy S26, Zorin OS, a 700 km), con la nuova AppImage
e il telefono riportato a zero (Opzioni sviluppatore spente, «Dimentica
questo telefono»), ha completato da solo il primo collegamento **senza
cavo** con «Prepara il telefono»: **circa 5 minuti, nessun blocco**
(riferito dall'utente). Per confronto: l'utente, esperto, ci ha messo 20 secondi.
Dopo la prova: il nome del telefono nelle frasi va tra virgolette senza
articolo («Dimenticare «S26 di …»?»); il testo di «Dimentica» non dice più
che serve il cavo.

## 41. Reel di Facebook senza audio: volume del telefono a 0 (28 set 2026)

- Segnalazione dell'utente: nell'app di Facebook i reel non attivano l'audio;
  dal sito, in Chrome, sì.
- Diagnosi (`phonestra-prova shell`): Facebook in `ImmersiveActivity` teneva il
  fuoco audio ma **nessun suo lettore era attivo**; volume multimediale del
  telefono **0/15**. Portato a 7: Facebook ha avviato subito un `AudioTrack`
  (`USAGE_MEDIA`, `state:started`) e l'audio è arrivato al PC. Chrome non guarda
  il volume, l'app di Facebook sì. Non era un problema di cattura.
- Correzione (decisione dell'utente: «il volume del telefono sempre al
  massimo»): il custode legge il volume, lo porta al massimo e lo rimette alla
  chiusura del canale. Provato: 0 → 15 collegato → 0 dopo `kill -9` di
  Phonestra; tempo di spegnimento ripristinato (120000).
- Una prova con l'AppImage finita male (il mio `pkill -f` ha colpito anche la
  shell della prova) ha lasciato il telefono a volume 15 e spegnimento 30 min:
  il custode era terminato senza ripristinare. Ripetuta in modo pulito,
  funzionava. Trovata comunque una debolezza vera: il custode rileggeva il
  volume a ogni ricollegamento, quindi dopo una caduta poteva prendere «15»
  per il valore dell'utente. Ora il volume funziona come il tempo di
  spegnimento: letto una volta per tutti i ricollegamenti, salvato in
  `telefoni.toml` (`volume_originale`), passato al custode; se al collegamento
  il volume è già al massimo vale quello salvato; riserva diretta alla
  chiusura. Provato: caduta improvvisa 0 → 15 → 0; volume lasciato a 15 con
  0 salvato → 15 → 0.
- **Ancora aperto**: con l'audio attivo i reel **vanno a scatti** («terribile»,
  l'utente ha chiuso la finestra). Da misurare con `PHONESTRA_DEBUG=1`
  (ritardi dei pacchetti audio, fotogrammi/s) guardando un reel 20–30 s, e da
  confrontare con i video normali del feed.

## 42. Tre sorgenti audio a confronto col componente nostro (28 set 2026)

Strumento di misura nell'aiutante (`phonestra-prova audio-nostro`), PCM non
compresso, orari dal conteggio dei campioni, thread a priorità −19 (concessa:
nice −19). Reel parlati di Facebook sul telefono, senza Phonestra, 60 s per
sorgente:

| Sorgente | Esito | Zeri / tagli | Deriva AudioTimestamp |
|---|---|---|---|
| submix (`REMOTE_SUBMIX`) | non partita: «Cannot create AudioRecord» | — | — |
| **loopback** (AudioPolicy `LOOP_BACK`) | 60 s, 0 letture perse | **0 / 0** | entro 1 ms |
| render (`LOOP_BACK_RENDER`, il telefono suona) | 60 s | un vuoto di 0,25 s all'avvio | fino a 64 ms |

L'utente ha ascoltato il file del loopback: **«perfetto»**. Le
micro-interruzioni di scrcpy (§41) non vengono dalla cattura di Android in sé
ma dal modo di catturare (remote submix) e di spedire di scrcpy. Scelta per il
componente: **loopback, PCM**, orari dal conteggio, thread −19. Da provare
ancora mentre il telefono codifica il video (A8 dello studio audio).
Elenco dei codificatori del telefono (`phonestra-prova codificatori`): audio
tutti software (anche AAC, compreso `c2.sec.aac.encoder`); video hardware
Qualcomm H.264/H.265 (16 istanze dichiarate), nessun AV1 hardware.

**A5 — loopback in AAC** (software, `c2.android.aac.encoder`), 60 s: 0 letture
perse, 0 zeri, 0 tagli, deriva entro 3 ms; file «perfetto anche lui»
all'ascolto. Ipotesi dell'utente verificata e scartata: la compressione
software non causa le interruzioni. Conclusione (osservazione dell'utente:
«forse il difetto è nel codice di scrcpy»): la modalità «playback» di scrcpy
usa lo **stesso** meccanismo (AudioPolicy `LOOP_BACK`) e dava 59 vuoti in
38 s; il nostro codice nessuno. Il difetto è nel modo in cui scrcpy legge,
comprime e spedisce (un solo thread senza priorità, orari presi all'uscita
del codificatore). **Scelta: loopback + AAC-LC 192 kbit/s** (otto volte meno
banda del PCM, utile col video sullo stesso Wi-Fi), PCM come riserva.


## 43. Video col componente nostro e audio sotto carico (28 set 2026)

Strumento `phonestra-prova video-prova` (aiutante, `VideoProva.java`), S23+
Android 16 (SoC SM8550). Telefono sbloccato (a telefono bloccato lo schermo
virtuale senza `ALWAYS_UNLOCKED` non disegna niente).
- **Permessi della shell**: tutti quelli che servono (`ADD_TRUSTED_DISPLAY`,
  `ADD_ALWAYS_UNLOCKED_DISPLAY`, `CAPTURE_VIDEO_OUTPUT`, `READ_FRAME_BUFFER`,
  `MANAGE_ACTIVITY_TASKS`, `REMOVE_TASKS`, `INTERNAL_SYSTEM_WINDOW`,
  `START_ACTIVITIES_FROM_BACKGROUND`, `MANAGE_DISPLAYS`, `DEVICE_POWER`); manca
  solo `CAPTURE_SECURE_VIDEO_OUTPUT`, come previsto.
- **Schermo virtuale nostro**: flag effettivi `TRUSTED|OWN_DISPLAY_GROUP|OWN_FOCUS`,
  Orologio avviato e catturato, pulizia completa.
- **Fotogramma chiave a comando** (`REQUEST_SYNC_FRAME`): 33–57 ms (medio ~40)
  in H.264 e H.265, 0 richieste mancate; con `prepend-sps-pps` l'intestazione
  arriva davanti a ogni IDR. **60 fotogrammi/s** pieni a 1120×1992 (scrcpy:
  ripartenze di 1–2 s e 24–37/s).
- **Istanze**: 8 schermi + codificatori aperti insieme (limite della prova),
  dichiarate 16; `PerformancePoint` 3840×2160@120 (circa 12 finestre a 60/s).
  Solo la prima aveva contenuto in movimento.
- **Schermate protette**: `captureDisplay` → `containsSecureLayers` vero con
  Bitwarden (immagine nera), falso con l'Orologio: niente più `dumpsys`.
- **Eventi dei task** (`ITaskStackListener`): avvio, fuoco, primo piano e
  orientamento richiesto (Facebook: PORTRAIT dopo 0,5 s) in tempo reale.
  Nota: un'app già aperta sul telefono viene spostata sullo schermo virtuale e
  chiusa alla fine della prova.
- **Gesto «indietro» (scrcpy #6007)**: schermo con `ALWAYS_UNLOCKED`, telefono
  bloccato e sbloccato: al blocco il Debug wireless cade (Samsung), il processo
  muore e Android chiude da solo lo schermo virtuale; gesto funzionante.
  Il caso a rischio (sblocco con lo schermo ancora aperto) **non si presenta**
  sui Samsung via Wi-Fi; non è escluso in assoluto.
- Difetto da ricordare: un aiutante interrotto di colpo lascia la sua copia
  in `/data/local/tmp` (il `rm` finale non parte): il componente vero deve
  ripulire anche dopo una caduta (custode).
- **A8, audio sotto carico**: 60 s di loopback AAC mentre uno schermo virtuale
  codificava H.265 a 60/s con fotogrammi chiave a richiesta: 0 zeri, 0 tagli,
  0 letture perse, deriva entro 1 ms; file «perfetto anche questo»
  all'ascolto. **Fase 0 chiusa** (resta la caduta del Wi-Fi per il custode).

## 44. Scheletro del componente sul telefono (28 set 2026)

`phonestra-prova servizio` (servizio unico, canali `localabstract` con segreto,
battito, custode `sh` con `setsid`), S23+ Android 16:
- **Avvio**: `CIAO` in 0,3–0,7 s (avvio del processo 138 ms, autotest 128 ms).
  Autotest tutto «ok»: contesto shell, 9/9 permessi, `createVirtualDisplay`,
  `captureDisplay` (tipi `ScreenCaptureInternal` di Android 16),
  `ITaskStackListener`, `injectInputEvent` con `setDisplayId`, AudioPolicy,
  appunti (anche `semclipboard` Samsung), custode con `setsid`.
- **Memoria**: servizio ~145 MB RSS (ART), custode ~3 MB. Da confrontare con
  i processi di scrcpy.
- **Chiusura ordinata** (codice 0), **PC sparito** (nessun battito: uscita
  da sola dopo ~4–5 s, codice 3), **`kill -9`** (137) e **`kill -HUP`** (129):
  in tutti i casi nessun processo rimasto, azione di ripristino di prova
  eseguita dal custode, nessun jar in `/data/local/tmp`. (Nella prova col
  segnale l'unico «NO» è il codice d'uscita, atteso diverso da 0.)

## 45. Modulo audio del componente (28 set 2026)

`phonestra-prova audio-componente` (canale `audio` del servizio, loopback +
AAC, `src/audio_nostro.rs` sul PC), reel parlato sul telefono:
- **60 s con ascolto dal vivo** dalle casse del PC: 2805 pacchetti, orari
  regolari, 0 letture perse, 0 zeri, 0 tagli (anche sull'AAC decodificato),
  deriva < 1 ms, 1 solo pacchetto in ritardo, margine finale 120 ms; primo
  audio 0,4 s dopo l'apertura del canale. L'utente: «audio senza nessuna
  interruzione, ma un piccolo ritardo tra audio e video» — il video era quello
  dello schermo del telefono (istantaneo) contro l'audio del PC (~0,3 s):
  la sincronia vera va misurata col video anch'esso sul PC.
- Politica audio: 0 prima, 1 durante, 0 dopo la chiusura del canale.
- **`kill -9` del servizio**: codice 137, Android toglie da solo la politica
  (0 rimaste), nessun processo né jar.

## 46. Moduli video e input del componente (28 set 2026)

**Video** (`phonestra-prova video-componente`, S23+ Android 16):
- App (Orologio) su schermo virtuale: primo fotogramma 0,6 s, ridimensionamento
  in 83 ms senza ricreare il codificatore a misura uguale, pannello spento e
  riacceso, eventi di orientamento e schermata protetta, pulizia completa.
- Il codificatore Qualcomm **ignora `repeat-previous-frame-after`**: a schermo
  fermo il fotogramma chiave a comando non usciva. Correzione: se entro 80 ms
  non esce, si stacca e riattacca la Surface dello schermo virtuale (ridisegno
  forzato). Ora 5/5, ritardo medio 120–180 ms, a schermo fermo ~160–325 ms.
- Il canale `video:<id>` non si chiudeva (lettura bloccata in un altro thread):
  ora `shutdownInput/Output` prima di `close`.
- Specchio dello schermo principale (drawer): riuscito, 880×1920.
- Il controllo «nessun task rimasto» va ripetuto per qualche secondo:
  `removeTask` è asincrono.

**Input** (`phonestra-prova input-componente tutte`): rotellina, trascinamento
con coordinate scalate, tocco che apre una voce, «indietro», tocco con misura
vecchia scartato; testo, Ctrl+A/Ctrl+C, incolla di «àèìòù €»; appunti in
scrittura e lettura, avviso delle copie di altre app, contenuti sensibili
senza testo; 104 eventi iniettati, 0 falliti. Samsung notifica ogni copia
**due volte**: il componente scarta lo stesso avviso entro 0,5 s.
Numerazione dei messaggi: video 0x40–0x4f, input 0x50–0x5f (test che vieta
i doppioni).
