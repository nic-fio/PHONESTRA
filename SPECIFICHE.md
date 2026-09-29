# Phonestra — Specifiche

> **§0 — Nome.** Dal 28 set 2026 il programma si chiama **Phonestra**
> (*phone* + *finestra*). Scelto dopo aver escluso
> nomi già usati come marchi o progetti nello stesso campo (dettagli in
> `memoria/decisioni-utente.md`).
>
> Bozza 0.1 del 26 settembre 2026.
> Questo documento raccoglie le decisioni prese con l'utente e i risultati delle
> prove fatte finora. Dove una cosa non è ancora verificata, è scritto
> esplicitamente («da verificare»).

## 1. Scopo

Phonestra serve a **usare le app del telefono Android su un PC Linux**, ciascuna
nella sua finestra, come se fossero programmi Linux.

È un'alternativa a scrcpy, con tre differenze di fondo:

- è un **programma grafico completo**, non uno strumento da riga di comando;
- **non richiede `adb`** né altri programmi installati: il protocollo ADB è
  integrato;
- il modello è **«app, non schermo»**: la vista principale è l'elenco delle app
  del telefono, non lo specchio dello schermo.

### Principi

1. **Niente da installare sul telefono.** Nessuna app. L'utente attiva solo le
   impostazioni che Android riserva a lui.
2. **Niente sporcizia sul PC.** Phonestra non scrive fuori dalle sue cartelle di
   configurazione e cache: niente icone nel menu di sistema, niente regole udev,
   niente moduli del kernel. Togliendo l'AppImage e le sue cartelle non resta
   traccia.
3. **Semplicità prima delle opzioni.** Quando c'è una scelta, si preferisce una
   regola chiara a un'impostazione in più.
4. **Reattività prima della qualità** nel video.

## 2. Piattaforme

| Voce | Scelta |
|---|---|
| Android supportato | **14 e successivi** (confermato il 28 set 2026 per il componente nostro: si studiano e usano solo le API di Android 14+, niente rami per le versioni vecchie) |
| Interfaccia | **GTK4 + libadwaita** |
| Formato di distribuzione | **AppImage** (vincolo: niente Flatpak), 64 bit, **x86_64 e aarch64** |
| Dimensione | non è un limite (anche oltre 100 MB) |
| Contenuto dell'AppImage | tutto incluso (GTK4, libadwaita, GStreamer/FFmpeg, componente per il telefono) tranne kernel, glibc e driver grafici (Mesa/VA-API), che devono venire dal sistema |
| Runtime AppImage | quello nuovo **statico**, che non richiede `libfuse2` |
| Audio e video su Linux | **PipeWire** |
| Sistema di riferimento | **Debian 13 «trixie»**: sviluppo e prove si fanno qui |
| Compatibilità | l'AppImage deve funzionare **su tutte le distribuzioni** diffuse: la versione da distribuire si compila in un contenitore (podman) con una glibc vecchia (base Ubuntu 22.04, glibc 2.35), perché un eseguibile compilato su trixie (glibc 2.41) non partirebbe sui sistemi più vecchi |

## 3. Architettura (proposta)

- **Programma per Linux**: un solo eseguibile in **Rust** (l'utente non ha
  preferenze sul linguaggio, conta solo che l'AppImage funzioni ovunque: Rust
  produce un eseguibile unico che dipende dal sistema solo per glibc). ADB
  parlato direttamente via USB e TCP/TLS, senza il server `adb` (libreria
  `adb_client` o implementazione propria); interfaccia con `gtk4-rs` e
  `libadwaita-rs`. Riferimento per l'abbinamento TLS di Android 11+: `ruri`.
- **Componente sul telefono: nostro** (decisione del 28 set 2026,
  `memoria/decisioni-utente.md`; architettura in `memoria/componente.md`,
  studio in `memoria/studio/`). È l'aiutante (`telefono/aiuto`, Java → dex con
  D8, licenza del progetto) diventato un **servizio unico per collegamento**:
  avviato con `app_process` come la shell, canali `localabstract` protetti da
  un segreto (comandi, audio, video di ogni finestra), battito ogni secondo,
  **custode** separato che ripristina il telefono e cancella le copie anche se
  il servizio muore. Java solo dove Android lo impone; codice di scrcpy mai
  copiato. A ogni collegamento viene copiato in `/data/local/tmp` e cancellato
  alla fine. **Non è un'app installata.** Fa **tutto**: audio, video di
  finestre e drawer, tocchi e tasti, appunti, pannello. Ha sostituito scrcpy
  un pezzo alla volta (fase 2: audio, poi video e input, poi appunti; prove
  §44–50); il **28 set 2026 scrcpy è stato tolto del tutto** (fase 3): niente
  server di scrcpy nell'AppImage, niente riserva né variabili per sceglierlo.
  Motivi (`memoria/decisioni-utente.md`, «Via da scrcpy»; prove §41–50):
  licenza tutta nostra, niente difetti non nostri, prestazioni (audio senza
  micro-interruzioni, video a 60 fotogrammi/s, un solo processo per
  collegamento). Se il componente non parte, o si ferma più volte nello
  stesso collegamento, Phonestra lo dice nel drawer e nelle finestre
  («Phonestra non parte sul telefono», con «Riconnetti ora» per riprovare).
- **Ogni finestra di app = un display virtuale** sul telefono, grande quanto la
  finestra, con l'app avviata lì (flag `TRUSTED|OWN_DISPLAY_GROUP|OWN_FOCUS`,
  prove §43).
- **Ricerca in rete**: implementazione mDNS propria (non Avahi, non `adb`).

## 4. Dati sul PC

| Cosa | Dove |
|---|---|
| Configurazione (telefoni, preferenze) | `$XDG_CONFIG_HOME/Phonestra`, cioè `~/.config/Phonestra` se la variabile non c'è |
| Chiave di abbinamento ADB | file nella cartella di configurazione, permessi `600` (niente portachiavi di sistema: non c'è su tutti i desktop) |
| Cache (icone delle app, ecc.) | `~/.cache/Phonestra`, cancellabile senza perdere niente |
| Screenshot e registrazioni | `Immagini/Phonestra` e `Video/Phonestra` dell'utente, cartella modificabile |

Cancellare `~/.config/Phonestra` riporta Phonestra allo stato iniziale.

## 5. Collegamento

### 5.1 Logica all'avvio

| Situazione | Cosa fa Phonestra |
|---|---|
| Nessun telefono nella configurazione | **Primo collegamento**: guida animata, associazione via Wi-Fi col QR (il cavo come riserva) |
| Telefono configurato e trovato | Si collega in silenzio e mostra il drawer |
| Telefono configurato ma non trovato | Schermata «Telefono non raggiungibile» con le cose da controllare (acceso, stessa rete Wi-Fi, Debug wireless attivo), ricerca continua in sottofondo e pulsante «Collega col cavo» |

«Non trovo il telefono» **non** significa «primo collegamento»: se il Debug
wireless è spento il telefono è invisibile in rete e Phonestra non può
distinguerlo da un telefono spento o su un'altra rete.

### 5.2 Primo collegamento

**Decisione del 27 set 2026 (notte), «scelta radicale» dell'utente:** al primo
collegamento Phonestra mostra una finestra con l'**elenco delle impostazioni
da attivare** sul telefono; come farlo sul proprio modello lo scopre
l'utente. Per ogni voce: perché serve, la parola da cercare nelle
Impostazioni del telefono, «Chiedi a Google ↗» (Modalità IA di Google Search
con la domanda già scritta). Le voci che Phonestra vede si spuntano da sole.
Niente libreria di moduli per modello, niente guide animate, niente «Che
telefono hai?» (mockup: `mockup/proposte/guida-elenco.html`). **Senza cavo** (decisione
dell'utente: «le opzioni le porto a 5 ed elimino il cavo»): **4 voci**:
stessa rete Wi-Fi, Opzioni sviluppatore, protezioni se ci sono, e **Debug
wireless** — una sola voce del telefono con due azioni: accendere
l'interruttore (più «Consenti» per la rete) e «Associa dispositivo con codice
di associazione», con le 6 cifre scritte in Phonestra (osservazione
dell'utente: «si fa quasi tutto da Debug wireless, una sola voce di menu, 2
opzioni»). Rispetto al cavo: 5 passi invece di 7–8, e spariscono cavo, presa,
«Solo ricarica» e la spunta «Consenti sempre». Poi Phonestra rende
tutto il più possibile **definitivo**: autorizzazione senza scadenza, Debug
wireless riacceso a ogni collegamento, e (da verificare) il riquadro rapido
«Debug wireless» aggiunto alla tendina, perché quando si spegne da solo
(cambio di rete, riavvio) basti un tocco. Provati il 27 set con l'`adb` di sistema: associazione col codice e annuncio
mDNS della schermata del codice (`_adb-tls-pairing._tcp`). Associazione nel nostro ADB
(`src/adb/abbina.rs`) provata il 27 set: funziona. Il percorso via cavo resta nel codice come riserva.

**Decisione del 27 set 2026 (sera): strada principale senza cavo.** Guida
animata col telefono in mano (Blocco automatico, Opzioni sviluppatore, Debug
wireless), poi **associazione col codice a 6 cifre** che il telefono mostra e l'utente
scrive in Phonestra («Associa dispositivo con codice di associazione»; indirizzo
e porta li trova Phonestra via mDNS; il QR solo come alternativa: l'utente lo ha
trovato complicato), Android 11+, protocollo SPAKE2 di
`adb pair` nel nostro ADB). Toglie cavo, porte, hub, «Solo ricarica», permessi
USB e la spunta «Consenti sempre». Prima dell'associazione Phonestra vede
l'avanzamento dagli annunci mDNS del telefono (Debug wireless acceso,
schermata di associazione aperta). Il modello non si conosce prima: «Che
telefono hai?». Le istruzioni sono moduli per marca › versione del sistema
(`memoria/libreria-procedure.md`), non più file per famiglia. Il percorso via
cavo qui sotto resta **la riserva** (reti diverse, Wi-Fi ospiti o
aziendale). *Da verificare: fattibilità dell'associazione.*

#### Riserva: via cavo

Phonestra rileva da solo ogni passo: l'utente non preme mai «Avanti».

1. «Collega il telefono col cavo USB» — rilevato all'inserimento.
2. Se serve: «Imposta la modalità USB su **Trasferimento file**» (in «Solo
   ricarica» Linux non dà il permesso senza root; vedi §5.6).
3. «Attiva le Opzioni sviluppatore e il **Debug USB**» — con istruzioni
   illustrate; rilevato quando compare l'interfaccia ADB.
4. «Tocca **Consenti**, spuntando *Consenti sempre da questo computer*» —
   rilevato quando il telefono risulta autorizzato. **La spunta è
   indispensabile**: senza, il cavo funziona ma il Wi-Fi viene rifiutato
   (`CertificateUnknown`). Phonestra lo verifica alla fine della procedura e, se
   serve, chiede di ripetere l'autorizzazione spuntandola.
5. Phonestra da solo:
   - disattiva la scadenza delle autorizzazioni
     (`settings put global adb_allowed_connection_time 0`);
   - attiva il Debug wireless (`settings put global adb_wifi_enabled 1`);
6. «Tocca **Consenti** per questa rete Wi-Fi» — solo la prima volta per ogni rete.
7. Phonestra trova il telefono in rete, prova il collegamento Wi-Fi e salva il
   telefono nella configurazione.
8. «Fatto, puoi scollegare il cavo» → drawer.

**Istruzioni per famiglia, non per modello**: la marca si legge dal cavo USB
già al passo 1; il percorso dipende dalla personalizzazione di Android della
marca (Samsung One UI; Xiaomi/Redmi/Poco HyperOS-MIUI; Pixel e Motorola;
Oppo/OnePlus/Realme; Honor/Huawei), non dal modello. Sempre indicata anche la
strada universale: la ricerca delle Impostazioni («numero build», «debug USB»),
che è l'istruzione principale per le marche sconosciute. Le istruzioni stanno
nei sorgenti in file di dati a parte (uno per famiglia), **incorporati nel
programma alla compilazione** (`include_str!`): l'AppImage resta un file unico e
funziona anche senza internet.

**Nessun aggiornamento da internet** (decisione dell'utente del 28 set 2026):
le istruzioni cambiano solo con una versione nuova del programma. Il
repository dei dati, previsto per aggiornarle, non è mai stato usato dal
codice ed è stato eliminato; così nessun servizio esterno vede l'indirizzo del
PC né la marca del telefono.

**Passaggi in più per famiglia**: una sola procedura; il file di una famiglia
può aggiungere passaggi propri, anch'essi rilevati da soli. **Xiaomi, Redmi,
Poco**: dopo «Consenti», attivare «Debug USB (impostazioni di sicurezza)» —
richiede account Xiaomi, internet, su alcune versioni la SIM — altrimenti mouse
e tastiera non arrivano alle app; Phonestra lo rileva mandando un tasto innocuo
(rifiutato = spento). «Installa tramite USB» si chiede solo al primo «Installa
app…», non nella procedura. Oppo, Realme, OnePlus, Honor: strada normale
(*da verificare*). Scartati i «due rami» (normali / cinesi): il confine è la
famiglia, non il paese. Telefoni **provati**: Samsung; gli altri **supportati**
finché non provati su un telefono vero.

Le Opzioni sviluppatore e il Debug USB **non si possono attivare dal PC**:
Android lo impedisce per sicurezza (sette tocchi su «Numero build», PIN).
Scartata l'ipotesi di navigare nelle impostazioni alla cieca via AOA (troppo
fragile). Scartata anche l'ipotesi degli assistenti sul telefono (Gemini,
AppFunctions, Routine Samsung): nessuno oggi controlla il Debug wireless.

**Senza cavo:** associazione col codice QR dalla schermata Debug wireless del
telefono: dal 27 set 2026 è la strada principale (vedi sopra).

### 5.3 Collegamenti successivi

Nessuna azione: Phonestra cerca in rete il numero di serie del telefono salvato,
legge la porta del momento e si collega. Se il Debug wireless si è spento
(cambio di rete, forse riavvio: *da verificare*), si riattiva dal riquadro
rapido «Debug wireless» del telefono oppure ricollegando il cavo.

### 5.4 Ricerca in rete (mDNS)

- Servizio: `_adb-tls-connect._tcp.local`.
- Nome dell'istanza: `adb-<numero di serie>-<suffisso>` (es.
  `adb-R5CT0000000-aBcDeF`): il numero di serie permette di riconoscere il
  telefono salvato senza conoscerne l'indirizzo.
- **La porta cambia a ogni attivazione** del Debug wireless (e a ogni riavvio
  del servizio di debug: cavo staccato, blocco/sblocco): va sempre letta dal
  record SRV. Per qualche istante può circolare ancora quella vecchia: si
  scartano i record con durata 0 e, se la porta rifiuta, si rifà la ricerca.
- La ricerca integrata in `adb` non ha trovato il telefono; una semplice
  interrogazione con risposta diretta (bit QU) sì. Phonestra fa la sua.
- Alcuni firewall bloccano mDNS: Phonestra lo rileva e spiega cosa sbloccare.

### 5.5 Cavo e Wi-Fi

- Se si collega il cavo mentre si usa il Wi-Fi, Phonestra **passa al cavo** da
  solo (latenza più bassa, ricarica); al distacco torna al Wi-Fi **senza chiudere
  le finestre**.
- Col cavo, Phonestra può anche riaccendere il Debug wireless.

### 5.6 Permessi USB su Linux

Verificato su Debian 13: la regola standard di **systemd**
(`70-uaccess.rules`) dà all'utente collegato l'accesso a qualsiasi dispositivo
USB che espone l'interfaccia MTP/PTP (`06/01/01`). L'accesso vale per tutto il
dispositivo, quindi anche per l'interfaccia ADB (`ff/42/01`). **Nessun root e
nessuna regola da installare**, purché il telefono sia in *Trasferimento file*.
In *Solo ricarica* c'è solo l'interfaccia ADB e l'accesso sarebbe negato: la
procedura guidata lo chiede. Se l'accesso è comunque negato, si propone il QR;
installare una regola udev con la password resta l'ultima scelta.

### 5.7 Caduta del collegamento

- Le finestre restano aperte con l'ultima immagine sfocata, la scritta
  «Riconnessione…» e **subito** i pulsanti «**Riconnetti ora**» e «Chiudi».
- I tentativi automatici continuano in sottofondo; «Riconnetti ora» rifà subito
  la ricerca.
- Se fallisce, la finestra dice perché e cosa fare.
- Alla riconnessione Phonestra ricrea i display virtuali e riporta ogni app nella
  sua finestra, dov'era (*da verificare* che l'app torni nel display).

### 5.8 Sicurezza del Debug wireless

Il Debug wireless acceso annuncia il telefono a tutta la rete (chi non è
autorizzato non può collegarsi). Opzione alla chiusura di Phonestra: «Spegni il
Debug wireless sul telefono», **attiva di default sulle reti nuove**, disattiva
su quelle di casa.

### 5.9 Il telefono deve restare sbloccato (verificato)

**A telefono bloccato le app non si aprono nelle finestre**, su nessuna
configurazione provata: il display virtuale resta spento finché il telefono
dorme, e se lo si sveglia senza sbloccarlo Android apre l'app sullo schermo
principale, dietro la schermata di blocco. È una regola di sicurezza di Android
e non va aggirata.

Modello d'uso adottato:

1. l'utente **sblocca il telefono**; Phonestra si collega;
2. Phonestra **spegne il pannello** del telefono (il telefono resta sveglio e
   sbloccato) e **porta al massimo il tempo di spegnimento** dello schermo per
   la durata del collegamento (i tocchi dal PC non contano come attività del
   telefono: con 30 minuti si addormentava durante l'uso), **salvando il valore originale nella configurazione del PC**;
3. l'utente usa le app sul PC e può dimenticarsi del telefono; le notifiche non
   riaccendono il pannello (verificato: vibra soltanto);
4. alla chiusura o alla caduta del collegamento il componente sul telefono
   **riaccende il pannello, ripristina il tempo di spegnimento e blocca il
   telefono** (il tempo di spegnimento letto dal telefono all'avvio; se
   l'utente lo ha cambiato durante il collegamento, resta il suo); se non ci riesce (collegamento già perso), Phonestra ripristina il
   tempo di spegnimento al collegamento successivo usando il valore salvato.
   (Nella prova scrcpy lo ha lasciato a 30 minuti.)
5. se l'utente **sblocca il telefono a mano** durante l'uso (dopo un blocco),
   Phonestra lascia il pannello acceso: lo sta usando in mano. Il pannello si
   rispegne al primo clic o tasto dal PC (schermo nel drawer o finestra di app).

Rischi verificati da gestire:

- a pannello spento **il touch del telefono resta attivo**: i tocchi arrivano
  alle app dello schermo principale (rischio tocchi in tasca);
- il **doppio tocco** sullo schermo spento (gesto Samsung) e il **tasto di
  accensione** mettono il telefono a dormire e lo bloccano: Phonestra mostra
  «Sblocca il telefono per continuare» e si ricollega da solo allo sblocco;
- consumo di batteria del telefono sveglio: *da misurare*.

### 5.10 «Impedisci connessioni USB se bloccato» (Samsung)

Impostazione Samsung in Sicurezza e privacy › Altre impostazioni di sicurezza
(`secure block_usb_lock`). A **ogni** blocco Samsung riavvia le funzioni USB e
con loro il servizio di debug:

- **attiva**: il debug (anche Wi-Fi) resta chiuso finché il telefono è bloccato;
- **disattivata**: il debug riparte subito su una porta nuova, ma il telefono
  bloccato non la annuncia in rete (trovata solo cercando le porte aperte).

Dato che a telefono bloccato le app non si aprono comunque (§5.9), **disattivarla
non serve ad Phonestra**: nessuna finestra al riguardo. Phonestra si limita a
ricollegarsi allo sblocco (la porta nuova viene annunciata).

La **Protezione avanzata** di Android (`secure advanced_protection_mode=1`,
attiva sul telefono dell'utente) non c'entra: la sua parte USB
(`UsbDataAdvancedProtectionHook`) non è disponibile su questo telefono. Resta
valida la decisione dell'utente: se in futuro una protezione impedisse ad
Phonestra di funzionare, Phonestra la spiega e offre di aprire **sul telefono** la
schermata giusta; non la disattiva mai da sé via shell (esiste
`cmd advanced_protection set-protection-enabled false`, deliberatamente non
usato).

## 6. Più telefoni

- Phonestra mostra **solo i telefoni configurati**, mai quelli di altri sulla
  stessa rete. Un telefono nuovo si aggiunge solo con «Aggiungi telefono».
- Si possono configurare più telefoni, ma **uno solo è attivo alla volta**.
- Nel drawer: nome (letto dal telefono, modificabile), stato (Wi-Fi / cavo / non
  raggiungibile), batteria. Menu: **Rinomina**, **Dimentica** (può anche revocare
  l'autorizzazione sul telefono), **Aggiungi telefono**.
- **Cambio di telefono** dal selettore del drawer: le finestre del precedente si
  chiudono (regola di §7.4), i suoi media vanno in pausa e il suo audio torna al
  suo altoparlante; poi Phonestra si collega al nuovo.
- **Notifiche dei telefoni non attivi**: arrivano comunque (collegamento leggero,
  solo notifiche), sempre col nome del telefono. Per passare a quel telefono
  serve il pulsante esplicito «**Passa a <telefono>**»: il clic semplice espande
  soltanto la notifica, per non chiudere per sbaglio il lavoro in corso.

## 7. Drawer e finestre delle app

### 7.1 Icona di Phonestra

- KDE, Xfce, Cinnamon: icona nell'area di notifica che apre il drawer.
- GNOME (niente area di notifica senza estensioni): l'icona di Phonestra nella
  dock, attiva finché c'è almeno un'app aperta.
- Le finestre delle app sono raggruppate sotto l'icona di Phonestra, ognuna col
  nome dell'app nel titolo. **Nessun file `.desktop` nel menu di sistema**, nemmeno
  come opzione.

### 7.2 Drawer

- Griglia delle app del telefono che hanno un'icona nel launcher (comprese
  Impostazioni, Fotocamera), con **icone vere** fornite dal componente sul
  telefono, **ricerca** e **preferiti** in cima.
- Si aggiorna da solo quando si installano o rimuovono app.
- **Schermo del telefono nel drawer**: a destra lo schermo principale del telefono
  in diretta dentro una cornice, sempre interattivo, anche col pannello fisico
  spento (tendina, impostazioni rapide, widget, schermata di blocco). Costo
  misurato: a schermo fermo circa un fotogramma al minuto e 1–2 KB/s. Senza
  collegamento, un disegno del telefono con «Riconnetti ora». Niente finestra
  separata dello schermo (tolta il 27 set 2026, decisione dell'utente: doppione
  dello schermo nel drawer; Home, recenti e tendina si fanno toccandolo).
- Pannello **Notifiche** (§8).

### 7.3 Aprire un'app

Un clic apre l'app in **una sua finestra** su un display virtuale. Se è già
aperta, viene portata in primo piano. Lo schermo del telefono si spegne da solo
(risparmio e privacy).

### 7.4 Chiudere un'app

Chiudere la finestra chiude l'app anche sul telefono: l'app viene **tolta dalle
recenti**, **mai arrestata forzatamente** (l'arresto forzato bloccherebbe le sue
notifiche finché non viene riaperta).

### 7.5 Comandi nelle finestre

| Comando | Finestre delle app |
|---|---|
| Indietro | sì: pulsante, **Esc** (attivo, disattivabile nelle impostazioni), **Alt+←**, tasto «indietro» del mouse |
| Home, App recenti | no (il drawer fa da Home, le finestre sono già nella barra di Linux) |
| Tendina notifiche, impostazioni rapide | no (si aprono dallo schermo del telefono nel drawer) |
| Volume del telefono | no (conta il volume del PC) |
| Ruota | sì |
| Ingrandisci, bordi da trascinare | sì, salvo le app che accettano solo il verticale: finestra a misura fissa (decisione dell'utente, prove-collegamento §27–29) |
| Screenshot, Registra | sì |

### 7.6 Casi particolari

- Se l'app viene aperta anche sul telefono, di norma Android la sposta lì: la
  finestra sul PC mostra «App aperta sul telefono – riportala qui».
- Le schermate protette (FLAG_SECURE: banche, contenuti protetti) risultano nere:
  messaggio chiaro, non si aggirano.
- Il numero di finestre contemporanee è limitato dai codificatori hardware del
  telefono: *da misurare*, avviso al superamento.
- Ridimensionamento (verificato): il display virtuale segue la finestra
  (messaggio `VIDEO_RIDIMENSIONA` del componente), con 1 punto del PC = 1 dp
  del telefono.
- La tastiera sullo schermo di Android non compare: si usa quella del PC.
  **Versione attuale**: il PC interpreta i tasti col suo layout e manda testo
  già composto; tasti speciali (Invio, Cancella, frecce, Tab, Home/Fine,
  Pagina su/giù) e Ctrl+lettera come tasti Android; le lettere non ASCII
  (à è ì ò ù…) passano dagli appunti del telefono con «incolla», quindi
  sostituiscono il loro contenuto. Più avanti: tastiera emulata via UHID per
  giochi e scorciatoie complesse (richiede il layout italiano impostato sul
  telefono per la tastiera fisica).
- Rotellina del mouse e scorrimento a due dita del touchpad: scorrono nel
  punto sotto il puntatore. Pagina su/giù: una schermata; frecce su/giù: un
  passo, finché non si scrive (poi muovono il cursore).
- **Zoom**: Ctrl + rotellina (nel punto del puntatore), Ctrl + «+»/«−» (al
  centro), pizzico sul touchpad → pizzico a due dita simulato, continuo (dita
  sollevate 300 ms dopo l'ultimo scatto), tocchi di un passo in un solo
  invio. Maps ha uno zoom proprio con la rotellina semplice.
- **Preferiti** in cima al drawer (clic destro su un'app), salvati per
  telefono in `telefoni.toml`; nascosti durante la ricerca.
- **Clic destro = pressione lunga del dito**: seleziona la parola e apre il
  menu di Android; Ctrl+C / Ctrl+V come sul PC (Ctrl+V incolla gli appunti del
  PC).

## 8. Notifiche

- **Pannello nel drawer**, ultima in cima, raggruppate per telefono e app,
  contatore sull'icona.
- **Avviso a comparsa** all'arrivo (disattivabile): una **notifica del sistema**
  (GNOME, KDE, Xfce), non una finestra propria — su Wayland un programma non può
  scegliere dove mettere le sue finestre. Titolo «Phonestra · <telefono>»;
  pulsanti «Apri» (telefono attivo) o «Passa a <telefono>», e «Nascondi».
  Rispetta «Non disturbare» e resta nell'elenco delle notifiche del sistema.
- **Privacy**: filtro per app; opzione «mostra solo il nome dell'app».
- Clic su una notifica del telefono attivo: apre l'app nella sua finestra.
- Fattibilità tecnica senza app sul telefono:

| Funzione | Stato |
|---|---|
| Leggere le notifiche (controllo ~1/s con i permessi della shell) | probabile, *da verificare* |
| Aprire l'azione della notifica (la chat giusta) | *da verificare*; ripiego: aprire l'app dall'inizio |
| Risposta rapida | *da verificare* |
| Cancellare la notifica sul telefono | improbabile: «Nascondi» agisce solo in Phonestra |

## 9. Appunti

- **Solo testo semplice**, nei due sensi, in automatico.
- Dal PC al telefono: il testo va **solo al telefono attivo**, al momento
  dell'incolla (Ctrl+V nella finestra dell'app).
- Dal telefono al PC: il testo copiato è subito negli appunti del PC.
- **Password escluse**: gli appunti segnati come sensibili (Android 13+) non vanno
  al PC; quelli marcati dai gestori di password del PC (KeePassXC e simili) non
  vanno al telefono.
- Testi molto lunghi: non passano, con l'avviso «usa il trasferimento file».
- **Realizzato** (26 set 2026; col componente nostro dal 28 set): telefono →
  PC con l'avviso `APPUNTI_CAMBIATI` del componente, che controlla da sé il
  segno «sensibile» (extra `android.content.extra.IS_SENSITIVE`; nel dubbio
  non passa); PC → telefono con Ctrl+V (appunti del telefono + incolla),
  escluso il segno `x-kde-passwordManagerHint`; niente rimbalzi di ciò che
  Phonestra mette negli appunti del telefono (lettere accentate comprese).

## 10. Audio

- Con il telefono collegato, l'audio delle sue app esce **solo dalle casse del
  PC**, al **volume del PC**. Nessuna opzione.
- Un flusso per telefono nel mixer di PipeWire («Phonestra – <telefono>»).
- **Volume multimediale del telefono al massimo** finché dura il collegamento
  (decisione dell'utente, 28 set 2026): con il volume a 0 l'app di Facebook non
  avvia l'audio dei reel. Il custode sul telefono (§5.9) legge il valore
  dell'utente, porta il volume al massimo e lo rimette quando il collegamento
  si chiude, anche se cade all'improvviso (provato). Il valore dell'utente è
  salvato anche in `telefoni.toml`, come il tempo di spegnimento: se una
  sessione lascia il volume al massimo, al collegamento successivo vale
  quello salvato. Il telefono non suona:
  l'audio esce solo dal PC.
- Alla caduta del collegamento o al cambio di telefono, il componente **mette in
  pausa i media** prima di chiudersi, così il telefono non riparte a suonare.
- Limiti: le chiamate (telefoniche e VoIP) restano sul telefono; le app che
  vietano la cattura dovrebbero comunque andare sul PC con la cattura
  dell'uscita (*da verificare*).
- **Sorgente di cattura** (28 set 2026, prove §41–47): **loopback**
  (AudioPolicy `ROUTE_FLAG_LOOP_BACK`) dal componente nostro, compresso in
  **AAC-LC 192 kbit/s**, orari dal conteggio dei campioni, lettura a priorità
  −19. Con scrcpy la stessa cattura («playback») dava vuoti di 50–120 ms e la
  cattura dell'uscita intera micro-interruzioni: il difetto era nel codice di
  scrcpy, non in Android. Col componente l'utente ha trovato Facebook e
  YouTube perfetti e in sincrono. Le app che vietano la cattura non arrivano
  al PC.

## 11. Installazione e rimozione di app

- Trascinare un `.apk` sul drawer, o «Installa app…».
- **Scheda di conferma** letta dal file: icona, nome, versione, firma,
  permessi; aggiornamento «dalla X alla Y» o avviso di firma diversa. Serve
  perché via ADB manca la richiesta «origini sconosciute» di Android.
- Installa sul telefono attivo, con barra di avanzamento; l'icona compare nel
  drawer.
- Play Protect può chiedere conferma o bloccare: Phonestra lo segnala.
- App per Android anteriori alla 6: Android 14 le rifiuta; Phonestra lo spiega e
  **non forza** l'installazione.
- **Disinstallazione** dal drawer (tasto destro → Disinstalla, con conferma), solo
  app dell'utente.
- **Rinviati**: pacchetti divisi (`.apks`, `.xapk`, `.apkm`).

## 12. Fotocamera e microfono del telefono

- **Le app del telefono usano fotocamera e microfono del telefono**, anche
  aperte in una finestra di Phonestra: niente da fare da parte di Phonestra.
  Provato il 27 set 2026 con una videochiamata WhatsApp: video e microfono
  perfetti.
- **Nelle chiamate la voce dell'interlocutore esce dal telefono**, non dal PC:
  Android non permette di catturare l'audio delle chiamate con la cattura usata
  per il resto (§10). Portarla alle casse del PC (altra cattura, da attivare
  solo durante le chiamate: quella intera rendeva i video a scatti) è **rimandato
  a una futura evoluzione**, da decidere con l'utente.
- **Tolto** (27 set 2026, decisione dell'utente, che aveva perplessità): il
  telefono come webcam e microfono per i programmi **del PC** (sorgenti
  PipeWire, echo-cancel). Non va riproposto senza una richiesta dell'utente.

## 13. Screenshot e registrazione

- Pulsanti nella barra di ogni finestra (e scorciatoie).
- Screenshot PNG alla risoluzione nativa, copiato anche negli appunti del PC come
  immagine.
- Registrazione in MP4 **senza ricodifica** (si salva il flusso che arriva dal
  telefono) con l'audio del telefono; pallino rosso e tempo nella barra.
- Durante la registrazione la finestra non si ridimensiona.
- L'audio registrato è quello di tutto il telefono, non della sola app.

## 14. Qualità video

- Misura continua di tempi di arrivo e coda; al peggioramento si abbassa il
  bitrate al volo, poi i fotogrammi al secondo, poi la risoluzione; al
  miglioramento si risale.
- **Mai accumulare ritardo**: i fotogrammi vecchi si scartano.
- Col cavo: qualità massima fissa.
- La finestra in primo piano ha la precedenza sulla banda.
- Codifica scelta da sola: H.265 di norma, H.264 come riserva, AV1 se supportato
  da entrambi. Decodifica VA-API con ripiego software.
- Nessuna impostazione, salvo «Priorità: reattività (predefinita) / qualità» tra
  le avanzate; avviso «Connessione debole» nella barra.

## 15. Interfaccia

- Stile scelto (per il momento): **fedele a libadwaita** — barra del titolo GNOME,
  selettore «App / Notifiche», liste a schede, pulsanti a pillola per le azioni
  principali, carattere di sistema (Adwaita Sans / Inter).
- Phonestra segue il tema chiaro/scuro del sistema. I mockup sono solo chiari: i
  temi scuri come proposta di stile sono stati scartati.
- Mockup: cartella [`mockup/`](mockup/) e canvas pubblicato (vedi
  `mockup/README.md`). Nei mockup le app hanno pittogrammi generici: nel
  programma le icone saranno quelle vere prese dal telefono.
- Stili «Barra laterale» e «Ricerca prima di tutto» restano nel canvas nella
  versione precedente, come alternative non approfondite.

## 16. Limiti noti (non aggirabili senza un'app sul telefono)

- SMS, chiamate, microfono del PC verso il telefono.
- Schermate protette nere.
- Cancellazione delle notifiche sul telefono (probabile).
- Opzioni sviluppatore e Debug USB da attivare a mano.

## 17. Prove effettuate

26 settembre 2026 — Galaxy S23+ (SM-S916B), **Android 16**, One UI 8.5;
PC Debian 13, stessa rete Wi-Fi. Dettagli e comandi in
[`memoria/prove-collegamento.md`](memoria/prove-collegamento.md).

| Prova | Esito |
|---|---|
| Rilevare debug USB spento / attivo / non autorizzato | ✅ dalle interfacce USB e dallo stato ADB |
| Accesso USB senza root | ✅ con telefono in Trasferimento file (regola uaccess di systemd) |
| Attivare il Debug wireless dalla shell via USB | ✅ con conferma sul telefono la prima volta per rete |
| Collegamento Wi-Fi senza QR, con la sola autorizzazione USB | ✅ anche dopo «Revoca autorizzazioni debug USB» |
| Ricerca del telefono in rete per numero di serie | ✅ con interrogazione mDNS propria; ❌ con `adb mdns` |
| Collegamento che resta attivo scollegando il cavo; ricollegamento da zero | ✅ |
| Scadenza autorizzazione 7 giorni e sua disattivazione dalla shell | ✅ `adb_allowed_connection_time` scrivibile |
| Leggere titolo e testo delle notifiche di altre app con i permessi della shell | ✅ `dumpsys notification --noredact`, anche notifiche private |
| Cancellare una notifica dalla shell | ❌ `cmd notification cancel` non esiste su questa versione |
| App su display virtuale, telefono sbloccato (scrcpy 4.1) | ✅; Samsung aggiunge la sua barra delle applicazioni: usare `--no-vd-system-decorations` |
| App su display virtuale, telefono bloccato | ❌ il display resta spento; svegliato senza sblocco, l'app va sullo schermo principale (§5.9) |
| Telefono sbloccato con pannello spento, app nella finestra del PC | ✅ funziona; interazione col mouse ok |
| Notifica con pannello spento | ✅ vibra, il pannello resta spento |
| Touch del telefono a pannello spento | ⚠ resta attivo (tocchi consegnati alle app) |
| Doppio tocco sullo schermo spento | ⚠ il telefono si addormenta e si blocca, il collegamento cade |
| **Tappa 1**: rilevamento USB, autorizzazione, Wi-Fi e collegamento TLS **senza `adb`** (prototipo `phonestra-prova`) | ✅ |

## 18. Prove da fare

1. **Consumo di batteria** con telefono sbloccato e pannello spento; come
   ridurre il rischio dei tocchi a pannello spento.
2. Apertura e risposta rapida alle **notifiche**; aggiornamento immediato invece
   del controllo periodico.
3. **Numero massimo di finestre** contemporanee.
4. Ritorno delle app nei display dopo una **riconnessione**.
5. **Audio**: influenza del volume del telefono; app che vietano la cattura.
6. Avviso di Android sugli **appunti**.
7. Qualità del **microfono** del telefono ed eco.
8. Fotocamera con telefono bloccato.
9. Debug wireless dopo un **riavvio** del telefono.

## 19. Decisioni aperte

- (nessuna: il nome è deciso, vedi §0)

## 20. Rinviato a versioni successive

- Pacchetti APK divisi.
- Silenziamento per singola finestra (Android permette di catturare l'audio di
  una sola app).
- Qualsiasi funzione che richieda un'app di supporto sul telefono (SMS,
  chiamate, microfono del PC).

## 21. Licenze

Phonestra ha una licenza propria, «sorgente visibile» e non open source
([`LICENZA.md`](LICENZA.md), decisione dell'utente del 28 set 2026): gratis
solo per uso personale; vietati senza accordo scritto con l'autore modifica,
redistribuzione, uso commerciale e uso in aziende, enti o per lavoro. Va
inclusa nell'AppImage.

Il componente sul telefono è nostro e ha la licenza di Phonestra: non ci
sono componenti di terzi da citare. Il server di scrcpy 4.1 (Genymobile,
Apache 2.0), tenuto come riserva durante la sostituzione, è stato tolto il
28 set 2026 (fase 3) insieme alla sua licenza, che l'AppImage non include
più (motivi in `memoria/decisioni-utente.md`, «Via da scrcpy», e prove
§41–50).

Solo la documentazione (`docs/`, non il programma né l'AppImage) include
**Mermaid** (Knut Sveidqvist, MIT) per disegnare i diagrammi del manuale
tecnico (li disegna `docs/disegna-diagrammi.py`: la pagina non lo carica), con la sua licenza in `docs/assets/vendor/mermaid.LICENSE` (decisione
dell'utente del 28 set 2026).
