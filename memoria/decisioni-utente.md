# Decisioni dell'utente e perché

Le specifiche dicono **cosa**; qui c'è il **perché** di alcune scelte, così da
non riproporre alternative già scartate.

- **AppImage obbligatoria, niente Flatpak** — l'utente non vuole installare
  centinaia di MB di runtime per una sola app. La dimensione dell'AppImage non
  conta.
- **Niente icone nel menu di sistema** — «sporcherebbe il PC». Le app del
  telefono si lanciano solo dal drawer.
- **Chiudere la finestra chiude l'app** sul telefono (proposta mia era farla
  tornare sul telefono: scartata). Tecnicamente: rimuovere dalle recenti, mai
  force-stop.
- **Un solo telefono attivo alla volta** — l'utente usa sempre un telefono per
  volta; niente finestre di due telefoni affiancate. Ma **le notifiche arrivano
  da tutti**, col nome del telefono, per decidere se passare all'altro.
- **Pulsante «Riconnetti ora» subito**, invece di far aspettare 30 secondi prima
  di proporre qualcosa.
- **Appunti: solo testo, solo verso il telefono attivo** — scartati «Invia a
  tutti» e l'invio automatico: l'utente ha scelto la versione semplice.
- **Audio solo sul PC, volume del PC** — nessuna opzione «suona anche sul
  telefono», nessun silenziamento per finestra.
- **PipeWire per webcam e microfono** — «ormai è su tutte le distro»; microfono
  del telefono perché di qualità migliore di quello dei PC.
- **Primo collegamento via cavo**, con Phonestra che fa tutto il possibile da sé;
  il QR solo come alternativa. *Superata il 27 set 2026 (sera), vedi sotto.* L'utente voleva che Phonestra attivasse anche la
  modalità sviluppatore: non si può, Android lo impedisce.
- **Assistenti del telefono (Gemini ecc.) esplorati su richiesta**: nessuno oggi
  può attivare il Debug wireless. Da riguardare tra un anno.
- **Protezioni del telefono**: l'utente voleva che Phonestra, rilevando una
  protezione che lo ostacola, offrisse di disattivarla. Accordo raggiunto:
  finestra con spiegazione e scelta, ma il pulsante «Disattiva» apre la
  schermata **sul telefono** e l'utente la spegne lì con la sua autenticazione;
  Phonestra non la spegne mai via shell. «Così è l'utente a decidere.» Le prove
  successive hanno mostrato che per il blocco non serve (le app non si aprono
  comunque a telefono bloccato), ma il principio resta.
- **Telefono sbloccato con pannello spento**: l'utente ha chiesto se il flusso
  è «sblocco, collego, e mi dimentico del telefono» — sì, dimostrato il 26 set.
- **Android-Dex** (github.com/Shrey113/Android-Dex, 26 set 2026): valutato su
  segnalazione dell'utente. Proprietario a sorgenti chiusi (licenza vieta
  modifiche e ridistribuzione), installa un'app sul telefono, usa ADB e scrcpy,
  Linux «solo per prova» con molte segnalazioni di AppImage che non parte
  (librerie incluse troppo vecchie o in conflitto: libmount, libpcre2, EGL,
  caricatore SVG). L'utente riconosce che la loro interfaccia è molto più
  curata; decisione: **«continuiamo sulla nostra strada»** — app come finestre
  Linux native, alzando il livello visivo (idea da prendere: sfondo del telefono
  e un po' di scenografia nel drawer). Niente modalità «desktop nella finestra».
  Lezione per la nostra AppImage: non includere librerie che devono venire dal
  sistema e provarla su più distribuzioni prima di pubblicarla.
- **Niente avvio automatico all'accesso** — Phonestra resta un'AppImage
  lanciata dall'utente (26 set 2026; la mia proposta usava
  `~/.config/autostart`, scartata). Al suo posto, idea dell'utente: scegliere
  a quale telefono collegarsi all'avvio → preferenza «Telefono all'avvio»
  (predefinito l'ultimo usato, per non chiedere a chi ha un solo telefono).
- **AppImage = un solo file, sempre** — niente file di dati accanto
  all'AppImage né scaricati da internet (27 set 2026, a proposito delle
  istruzioni per famiglia di telefono): i dati stanno nei sorgenti e vengono
  incorporati alla compilazione. Eccezione scelta dall'utente: durante il
  primo collegamento le istruzioni della famiglia del telefono si aggiornavano
  dal repository pubblico `nic-fio/ANDROLIN-DATA`. **Tolta il 28 set 2026**:
  l'utente ha deciso che il repository dei dati non serve più (il codice non
  lo usava ancora); le istruzioni cambiano solo con una versione nuova.
- **Interfaccia**: vedi [interfaccia.md](interfaccia.md).

- **Webcam e microfono** (27 set 2026) — le app del telefono usano la sua
  fotocamera e il suo microfono anche in finestra (videochiamata WhatsApp
  provata: «mi sentiva in modo perfetto»); la voce delle chiamate esce dal
  telefono: «al momento va bene così, vediamo poi in un'eventuale futura
  evoluzione». Tolto il telefono come webcam/microfono per i programmi del PC
  (l'utente aveva perplessità): supera la voce «PipeWire per webcam e microfono».
- **App solo verticali** (27 set 2026) — finestra a misura fissa, senza
  ingrandimento né bordi da trascinare (insistito dall'utente, preferito alla
  mia controproposta di ingrandire l'immagine).
- **Niente «Apri in una finestra»** per lo schermo del telefono (27 set 2026):
  doppione dello schermo vero nel drawer.
- **Procedure guidate animate** (27 set 2026, dopo il beta-tester con l'S26) —
  l'utente: «i telefoni odierni sono pieni di blocchi disseminati in vari
  menu… quando capita in mano a un utente abituato ad avanti → click → avanti
  bisogna cambiare completamente il modo di pensare»; «vere e proprie
  procedure guidate, facendo largo uso di animazioni, video, grafica: è su
  questo territorio che si distinguono le app ben fatte dalle amatoriali».
  Esclusi aggiramenti delle protezioni («giustamente senza tecniche di
  cracking»). Mia precisazione: animazioni **disegnate e generate dai dati**
  (percorsi dei menu), non video registrati (invecchiano a ogni versione di
  One UI/HyperOS, pesano nell'AppImage); animazione + rilevamento dello stato
  che avanza da sé. Primo passo: mockup animato della procedura Samsung (caso
  S26: Blocco automatico, Numero build, Debug USB). Proposta collegata, da
  valutare: associazione senza cavo col QR del Debug wireless (Android 11+),
  che toglie cavi, porte e «Solo ricarica».
- **Primo collegamento senza cavo** (27 set 2026, sera) — l'utente: «se
  riusciamo a risolvere il problema della configurazione del telefono
  preliminare, anche la necessità del cavo USB viene meno». Il beta-tester si
  era fermato proprio su cavo, porta e «Solo ricarica». Mia precisazione: la
  configurazione preliminare resta (Blocco automatico, Opzioni sviluppatore,
  Debug wireless al posto del Debug USB) e il modello non si conosce prima
  dell'associazione. Decisione: associazione col QR come strada principale,
  cavo come riserva. Prossimo: prova di fattibilità (adb di sistema col S23+,
  poi SPAKE2 nel nostro ADB), poi discutere la configurazione preliminare
  («un ginepraio»).
- **Primo collegamento: solo l'elenco** (27 set 2026, notte) — dopo una
  giornata sulle guide animate per modello, sul codice a 6 cifre e sul QR,
  l'utente: «siamo finiti in un ginepraio, serve una scelta radicale… l'app
  visualizza la finestra con le funzionalità che l'utente deve abilitare,
  poi sarà lui a capire come farlo». Accettato: rende il progetto finibile.
  Aggiunte mie a costo quasi nullo: parola da cercare nelle Impostazioni,
  «Chiedi a Google ↗», spunte automatiche. Proposta mia di tornare al cavo
  (già costruito) respinta: «almeno 2-3 opzioni l'utente deve pure
  settarle; a quel punto le porto a 5 ed elimino il cavo. L'app poi avrà il
  compito di renderle definitive». Quindi **senza cavo**, associazione col
  codice; rischio da togliere per primo: SPAKE2 nel nostro ADB. Libreria di moduli e guide
  animate (`libreria-procedure.md`) accantonate, non cancellate: i mockup
  restano in `mockup/proposte/guida-*.html`.
- **Niente disegni dell'interfaccia del telefono, nemmeno «universali»**
  (27 set 2026, notte) — l'utente ha notato che le difficoltà vere sono due:
  trovare l'impostazione (la ricerca non la sanno usare) ed entrare nei
  sotto-menu (la scritta «Debug wireless» contro l'interruttore). Mia
  proposta: due disegni «universali» (come si cerca; la riga a due zone).
  L'utente: «sui Samsung non c'è nessuna lente, solo una finestrella, e si
  trova pure in basso. Ci stiamo infilando in un sentiero parecchio
  accidentato». Scartati: anche i gesti generici cambiano tra telefoni.
  Restano solo parole che non dipendono dall'aspetto (testo dell'utente: «usa lo
  strumento di ricerca delle Impostazioni per individuare l'impostazione»,
  più la parola da scrivere; «tocca la scritta, non l'interruttore») e «Chiedi a
  Google», che per il modello dell'utente mostra anche i video.
- **Licenza propria, non open source** (28 set 2026) — decisione dell'utente,
  presa mentre si valutava di rendere pubblico il repository: «il prodotto si
  può usare liberamente, ma è vietata la modifica, la redistribuzione e l'uso
  commerciale». Testo in `LICENZA.md` (italiano, prevale; traduzione inglese).
  Conseguenze spiegate all'utente: nessuna licenza standard lo fa (CC BY-NC-ND
  permette la redistribuzione); su GitHub il fork resta permesso dai termini
  del servizio; niente pacchetti delle distribuzioni né copie passate tra
  utenti, si scarica solo dalle Release. «Uso commerciale» definito come
  vendere, farsi pagare, includerlo in prodotti o servizi a pagamento.
  Poi l'utente ha precisato: «gratis per uso personale ma è vietato usarlo in
  ambiti aziendali». Aggiunto il divieto d'uso da parte di aziende, enti e
  organizzazioni o per lavoro, con «accordo scritto con l'autore» come porta
  per eventuali licenze aziendali (anche a pagamento).
- **Nome definitivo: Phonestra** (28 set 2026) — *phone* + *finestra*. Scartati
  dopo controllo su TMview (marchi in classe 9: Italia, UE, internazionali),
  GitHub e ricerca web: PHONIX (marchio italiano registrato in classe 9;
  azienda milanese di accessori per smartphone), LINDROID (progetto open source
  omonimo che porta Linux su Android; «droid» è un marchio), MOBIX (marchio
  internazionale del 2024 e software Mobix per telefoni collegati al PC),
  MOBILE DECK (estensione «Mobile Deck» che mostra telefoni sul PC; Elgato
  Stream Deck Mobile), ANDESK (quasi omofono di AnyDesk, marchio registrato
  per il controllo remoto; LANDESK di Intel/Ivanti), OBLO (decine di marchi),
  FENESTRA (molti progetti omonimi). PHONESTRA: nessun marchio né progetto
  trovato. Chi aggiorna dalla 0.2.0 ritrova tutto: al primo avvio le cartelle
  `Androlin` (configurazione, Immagini, Video) diventano `Phonestra`.
- **Repository nuovo senza storia** (28 set 2026) — l'utente: «è una semplice
  app: non credo serva conservare tutta la storia». `nic-fio/PHONESTRA` parte
  da un solo commit; il perché delle decisioni sta in `memoria/`. La storia
  completa non è stata conservata (la cartella `~/Documenti/ANDROLIN` e il
  bundle di sicurezza non ci sono più dal 28 set sera); le vecchie AppImage
  restano in `~/ANDROLIN-release-archivio`. Motivo: il repository diventa pubblico e la
  storia conteneva dati personali (poi ripuliti).
- **Via da scrcpy: componente per il telefono tutto nostro** (28 set 2026) —
  dopo una giornata sulle micro-interruzioni dell'audio (prove §41–42).
  L'utente: «ci sganciamo da scrcpy e da Java»; motivi: licenza libera, niente
  bug non nostri, prestazioni. Discusso: **Java resta solo dove Android lo
  impone** (cattura dell'audio, schermi virtuali, invio di tocchi e tasti sono
  servizi Java di Android, senza API native ufficiali); la prova col
  registratore Samsung (anch'esso Java, audio perfetto) mostra che il limite
  non è il linguaggio. L'utente: «mi piacerebbe liberarmi di scrcpy, al costo
  di allungare i tempi». Sostituzione **un pezzo alla volta**, scrcpy resta
  finché il pezzo nostro non è provato: 1) audio (ricetta del registratore
  Samsung: uscita intera + AAC), 2) video e finestre delle app, 3) tocchi,
  tasti, appunti, comandi, 4) scrcpy tolto da AppImage e licenza. Il
  componente è l'aiutante (`telefono/aiuto`), licenza del progetto.
- **Componente nuovo: stesse funzioni di oggi** (28 set 2026) — l'utente:
  «tenere le stesse funzionalità. L'app di adesso va bene, i problemi sono solo
  quelli delle prestazioni che conosciamo». Il componente nostro è pronto
  quando fa tutto quello che fa scrcpy oggi, senza i difetti di prestazioni
  (audio che si interrompe, video che scende di fotogrammi). Dentro solo le
  correzioni che toccano le prestazioni (cattura dell'audio col ritmo giusto,
  «delayed ack» nel nostro ADB, un solo processo con l'audio non rallentato
  dal video). **Rimandate** le novità emerse dallo studio: tastiera italiana
  UHID, notifiche in tempo reale, ripetizione dei tasti, controller da gioco.
- **Piano del componente approvato** (28 set 2026, `studio/README.md`):
  1) i pezzi delicati di scrcpy (contesto di sistema, accorgimenti Samsung) si
  **riscrivono**, scrcpy solo come documentazione; 2) la prova del gesto
  «indietro» si può fare anche se richiede di riavviare il telefono
  («nessun problema»); 3) ordine di lavoro: misure (fase 0), sviluppo in
  parallelo, passaggio un pezzo alla volta, scrcpy tolto.
  **Esito** (28 set 2026, sera): con l'audio del componente Facebook e YouTube
  «perfetti», senza micro-interruzioni e in sincrono (prove §47); video
  misurato a 60 fotogrammi/s e fotogramma chiave in 0,1–0,3 s (§43, §46).
  L'utente: «la scelta di abbandonare scrcpy e utilizzare una nostra applet ha
  abbondantemente pagato». Metodo che ha funzionato: studio → misure sul
  telefono → codice, un pezzo alla volta con scrcpy di riserva.

