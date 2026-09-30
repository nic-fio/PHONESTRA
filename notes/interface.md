# Interfaccia: come si è arrivati allo stile attuale

1. Proposti 5 stili × 4 schermate (drawer, finestra app, notifiche, primo
   collegamento) = 20 mockup, invece di 20 interfacce tutte diverse, per poterli
   confrontare a parità di schermata.
2. L'utente ha chiesto le **icone vere** delle app: non si usano i loghi dei
   marchi (WhatsApp, Gmail…); al loro posto pittogrammi generici su sfondo
   colorato (`mockup/icons/`). Nel programma le icone saranno quelle del telefono.
3. L'utente ha chiesto di **eliminare i temi scuri**: tolti gli stili «Pannello
   compatto scuro» e «Stile telefono»; schiarito lo sfondo dello stile «Ricerca».
   Phonestra seguirà comunque il tema chiaro/scuro del sistema.
4. L'utente ha giudicato i mockup **«amatoriali»**: lo stile 1 è stato rifatto
   **fedele a libadwaita** (misure della barra del titolo GNOME, pulsante di
   chiusura tondo, selettore App/Notifiche, liste a schede, pulsanti a pillola,
   carattere Inter ≈ Adwaita Sans, app interne in stile Material 3).
   Risposta: «per il momento può andar bene così».
5. Stili 2 (Barra laterale) e 3 (Ricerca prima di tutto) sono rimasti nella
   versione precedente, non rifiniti.

## Note pratiche sul canvas

- Il canvas pubblicato impiega **20–30 secondi** a comparire; nel frattempo la
  pagina è bianca. Non è un guasto.
- Un'animazione CSS infinita in un mockup ha **bloccato il canvas**: non usarne.
- Quando si mostra il canvas all'utente nel browser, **lasciare aperta la
  scheda**.

## Nuova direzione: stile «vetro» (26 set 2026, sera)

Base portata dall'utente (un mockup «Remotix Mobile» trovato in rete),
adattata in `mockup/proposals/drawer-glass*.html` (si aprono nel browser; PNG
accanto). L'utente: «un netto miglioramento rispetto all'interfaccia attuale».

- **Tenuto**: sfondo sfumato chiaro (colori da ricavare dallo sfondo del
  telefono), sezioni in riquadri bianchi semitrasparenti, testo scuro (risolve
  i testi bianchi illeggibili), pillola del telefono nella barra del titolo,
  telefono disegnato a destra.
- **Scartato dalla base**: voci che Phonestra non ha (File, Messaggi),
  preferiti ripetuti tre volte, ora/Wi-Fi/batteria del PC, griglia a 3
  colonne, sfocatura dello sfondo (GTK 4 non la sa fare).
- **Immagine del telefono**: non foto del modello (archivio impossibile,
  immagini protette) ma cornice generica con lo sfondo vero, proporzioni vere,
  nome e batteria. Lo sfondo vero oggi non si riesce a leggere (vedi
  prove-collegamento, «Sfondo del telefono»; il 26 set sera: «Killed»).
- **Telefono «vivo»** (idea dell'utente, contenuto proposto da me sul modello
  della schermata di blocco): musica in riproduzione con comandi, (le ultime
  3 notifiche, poi tolte), interruttori Webcam e Microfono, zona di rilascio.
  Niente schermo in diretta (batteria e rete).
- **Trascinare un file sul telefono** lo invia ai Download; un `.apk` lo
  installa (approvato dall'utente).
- **Barra laterale** (tolta, poi rimessa su richiesta dell'utente, con altre
  voci): pagine App e Notifiche (contatore); sezione «Telefoni» (telefoni
  configurati, clic = passa a quel telefono, in fondo «+ Aggiungi telefono»:
  l'utente voleva la configurazione di un nuovo telefono nella barra, messa
  accanto all'elenco invece che da sola); gruppo «Strumenti» con Schermo del
  telefono, Installa app…, Invia file…; in fondo Preferenze (di Phonestra; «Impostazioni» si confondeva con l'app del
  telefono) e Informazioni.
  Niente menu ☰. «Aggiorna l'elenco delle app» va dentro Preferenze (l'elenco
  si aggiorna da solo). Rinomina / Dimentica / Riconnetti nel menu della pillola del
  telefono. Pagine Foto e File (telefono → PC): diventate la voce «Ricevi
  file…» (30 set 2026, SPECIFICATION §11.1).
- **Riconnessione**: niente pulsante fisso; la pillola diventa arancione e il
  telefono disegnato mostra «Riconnetti ora»; voce anche nel menu del telefono.
- Da disegnare: stati del collegamento, menu del telefono (pillola), menu
  dell'app col clic destro, pallino sulle app aperte, Impostazioni, tema scuro.
- **Preferenze** (bozza, `drawer-glass-preferences.html`): Finestre (Esc
  indietro, Priorità reattività/qualità), Notifiche (avviso a comparsa, solo
  nome dell'app, app che possono avvisare), File (cartelle screenshot e
  registrazioni; cartella del telefono per i file inviati — nuova), Webcam e
  microfono (elimina l'eco), Avvio (Telefono all'avvio: l'ultimo usato / chiedi ogni volta / un telefono
  preciso; nascosta con un solo telefono), App del
  telefono (Aggiorna ora). Audio e appunti restano senza opzioni.
- **Aggiunta di un telefono**: sospesa, l'utente ci sta pensando (ha un'idea
  per semplificare la procedura).
- **Notifiche: una sola porta** (decisione finale dell'utente, dopo un giro
  in cui stavano sul telefono disegnato con «tutte ›», barra «sei qui» e
  «chiudi»: «si sta complicando»). Voce Notifiche nella barra laterale con
  contatore; **niente notifiche sul telefono disegnato** (restano musica,
  webcam/microfono, zona di rilascio); niente freccia ← nella pagina.
  «Nascondi tutte» resta (svuota l'elenco, non è un doppione).
- **Notifiche dei telefoni non attivi** (regola già decisa: arrivano da tutti):
  solo il contatore grigio sulla riga del telefono nella barra laterale;
  **niente nella pagina Notifiche** (scelta dell'utente: la pagina è del solo
  telefono attivo). Al passaggio del mouse sulla riga, anteprima delle ultime
  notifiche (controproposta mia: senza, per leggerle bisognerebbe passare al
  telefono, chiudendo le app aperte). Clic sulla riga = passa a quel telefono;
  con app aperte chiede conferma («Chiudere 3 app del Galaxy S23 e passare al
  Tablet?»). Da misurare: consumo di batteria del collegamento leggero sul
  telefono non attivo.
- **Avviso a comparsa** = notifica del sistema (vedi SPECIFICATION §8 e
  `mockup/proposals/system-alerts.html`): una mini-finestra propria in un
  angolo non si può posizionare su Wayland.
- **Pagina Notifiche, dettagli** (proposti e mostrati all'utente): al passaggio
  del mouse su una notifica «Apri» e «×» (nasconde solo quella, in Phonestra);
  ogni app mostra le ultime 2 con «altre N notifiche di <app> ›»; pagina vuota
  «Nessuna notifica» con campanella tenue.
- **Batteria e Wi-Fi**: «Wi-Fi · 78 %» si leggeva come intensità del Wi-Fi
  (segnalato dall'utente). Ora: sul telefono disegnato icone come su un
  telefono vero (Wi-Fi, batteria + «78 %»); nella pillola solo lo stato del
  collegamento («Galaxy S23 · collegato via Wi-Fi» / via cavo), senza batteria.
- **App aperte**: pallino blu sotto l'icona nella griglia (preferiti e tutte),
  come nelle dock; funziona su qualunque desktop (dubbio dell'utente: non tutti
  usano GNOME). Niente elenco delle app aperte sul telefono disegnato.
- **Telefono disegnato, scelte dell'utente**: musica resta (avanti/indietro
  senza aprire l'app); interruttore **Non disturbare** accanto a Webcam e
  Microfono; **In carica · piena alle 15:40** (fulmine anche sulla batteria);
  solo quando succedono (`drawer-glass-events.html`): **chiamata in arrivo**
  sopra la musica con «Rifiuta» e «Rifiuta e scrivi» (l'utente chiedeva
  «invia messaggio»: l'SMS automatico non si può senza app, §16; invece si
  apre Messaggi in finestra sulla conversazione con chi chiama — da
  verificare numero del chiamante e rifiuto dalla shell); **trasferimento in
  corso** con avanzamento e ×. Scartate per ora: ultime foto trascinabili.
- **Stati del collegamento** (`pill-states.html`, `drawer-glass-reconnection.html`,
  `drawer-glass-unreachable.html`): la pillola cambia colore e testo
  (verde collegato via Wi-Fi/cavo, grigio collegamento…, giallo connessione
  debole, arancione riconnessione…, rosso non raggiungibile); lo stesso stato
  sulla riga del telefono nella barra laterale. Collegamento perso: app
  attenuate, telefono disegnato oscurato con «Collegamento perso» e «Riconnetti
  ora». Non raggiungibile all'avvio: al centro «Non trovo il Galaxy S23» con 3
  cose da controllare, «Riconnetti ora» e «Collega col cavo»; telefono grigio
  con l'ora dell'ultimo collegamento. Clic sulla pillola: Riconnetti, Rinomina,
  Dimentica.
- **Procedura guidata del primo collegamento** (bozza, `procedure-*.html`;
  l'utente ci sta ancora pensando): tutta la finestra, senza barra laterale;
  a sinistra i 5 passi con spunta automatica, al centro le istruzioni, a
  destra il telefono disegnato che mostra la schermata da cercare con il punto
  da toccare evidenziato (es. «Numero build — tocca 7 volte», la casella
  «Consenti sempre»). Percorsi per marca: la marca si legge dal cavo USB prima
  del Debug USB. Il passo «Trasferimento file» compare solo se serve.
  Rilevamento: tutti i passi avanzano da soli tranne l'interno del Debug USB
  (7 tocchi, Opzioni sviluppatore, interruttore): finché è spento il telefono
  non dice niente al PC. Quei tre passaggi si sfogliano a mano (frecce ‹ › sul
  telefono disegnato); appena compare l'interfaccia di debug si salta al passo
  successivo.
  Istruzioni per famiglia di marca, non per modello (proposta dell'utente
  «per ogni modello» → migliaia di modelli, impossibile da mantenere); più la
  ricerca nelle Impostazioni come strada universale. Vedi SPECIFICATION §5.2.
- **Finestra dell'app** (`app-window.html`, 4 stati affiancati): barra
  sottile con la sfumatura del drawer; Indietro, titolo app + telefono,
  Screenshot, Registra (diventa «● 0:42» in rosso durante la registrazione),
  menu ⋮ (Ruota, Copia screenshot, Mostra sul telefono — nuova, da verificare
  —, Chiudi app), chiudi. Collegamento perso: ultima immagine sfocata con
  «Riconnessione…», «Riconnetti ora», «Chiudi». Schermata protetta: nero con
  lucchetto, «Indietro», «Mostra sul telefono». App aperta sul telefono:
  sfocata con «Riportala qui».
- **Menu del telefono** (clic sulla pillola, `drawer-glass-phone-menu.html`):
  intestazione con nome, modello, Android, rete e batteria; Riconnetti,
  Rinomina…, interruttore «Spegni il Debug wireless alla chiusura» (per la rete
  attuale, §5.8), Dimentica questo telefono… (rosso).
- **Menu dell'app** (clic destro, `drawer-glass-app-menu.html`): intestazione
  col nome e lo stato; Porta in primo piano / Apri, Aggiungi–Togli dai
  preferiti, Informazioni sull'app (la pagina di Android, in una finestra),
  Chiudi app (solo se aperta), Disinstalla… (rosso, solo app dell'utente).
  Stesso riquadro dei menu già approvati (richiesta dell'utente: non alterare
  lo stile).
- **Telefono disegnato = schermo vero** (idea dell'utente, 27 set 2026,
  decisa dopo una misura): nella cornice lo schermo principale del telefono in
  diretta, sempre interattivo, anche col pannello fisico spento. Misura con la
  finestra «Schermo del telefono»: schermo fermo 0 fotogrammi/s (uno al minuto,
  l'orologio), 1–2 KB/s; piccoli movimenti 2–3 fotogrammi/s, 10–30 KB/s.
  La mia obiezione (batteria e rete) era sbagliata: Android manda fotogrammi
  solo quando lo schermo cambia. Conseguenze:
  - via ora, batteria, carica e musica disegnate (c'è la barra di stato vera,
    e la tendina vera coi controlli dei brani); via la voce «Schermo del
    telefono» della barra laterale (resta un pulsante per aprirlo in una
    finestra grande); il disegno resta come riserva (collegamento perso,
    bloccato);
  - via l'interruttore **Non disturbare** (sta nelle impostazioni rapide vere,
    proposta dell'utente);
  - **Webcam e Microfono automatici**, niente interruttori: sempre presenti tra
    i dispositivi del PC, si accendono quando un programma del PC li apre;
    segnale «● fotocamera in uso» sulla cornice (l'utente pensava di usare le
    impostazioni rapide del telefono, che però bloccano fotocamera e microfono
    alle app: sono un'altra cosa);
  - trascinare un file sopra lo schermo vero lo invia (tutta la cornice è la
    zona di rilascio); la tastiera va al telefono solo dopo un clic sullo
    schermo, altrimenti resta alla ricerca delle app.
- **Pannello fisico** con lo schermo vero sempre aperto: quando l'utente
  sblocca il telefono a mano (dopo un blocco durante l'uso), Phonestra lascia il
  pannello acceso; si rispegne quando si torna a usarlo dal PC (clic nello
  schermo del drawer o in una finestra di app) o dopo il tempo di spegnimento
  dell'utente senza tocchi (dal 29 set, prove §58: il tempo di spegnimento del
  telefono è «mai»); una chiamata in arrivo lo riaccende. Senza questa regola il telefono non si potrebbe usare in mano
  finché Phonestra è aperto (approvato dall'utente).

- **Ricevi file…** (30 set 2026, richiesta del beta-tester dopo il successo
  del trascinamento PC → telefono). Scartati, con l'utente: una pagina di soli
  file recenti da trascinare fuori (mia prima proposta) e un esploratore senza
  posti rapidi (sua prima proposta: lenta per la foto appena scattata, sepolta
  in `DCIM/Camera`). Scelta: la sua finestra di navigazione con i posti rapidi
  a sinistra e Recenti come partenza. Destinazione **fissa in Scaricati**, niente
  «Salva in…» a ogni ricezione (decisione dell'utente, 30 set). Mockup
  `mockup/proposals/receive-files.html`. Provato dall'utente: «trasferimento da
  telefono a PC OK».
