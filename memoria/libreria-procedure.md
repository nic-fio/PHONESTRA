# Libreria di procedure guidate (progetto, 27 set 2026) — ACCANTONATA

> Il 27 set 2026 (notte) l'utente ha scelto una «scelta radicale»: solo l'elenco
> delle impostazioni (SPECIFICHE §5.2, `decisioni-utente.md`). Quanto segue
> resta come storia e come riserva.

## Perché

Il primo beta-tester (Galaxy S26, Zorin OS, a 700 km, «non è in grado di fare
indagini sofisticate») si è fermato più volte: Debug USB grigio per il Blocco
automatico, telefono in «Solo ricarica», PC che non vede il telefono. L'utente:
«i telefoni odierni sono pieni di blocchi disseminati in vari menu… bisogna
cambiare completamente il modo di pensare»; «vere e proprie procedure guidate,
con animazioni, video, grafica: è su questo territorio che si distinguono le
app ben fatte dalle amatoriali». Niente aggiramenti delle protezioni.

## Decisione dell'utente

«L'unica cosa che l'app deve fare è identificare marca e modello del telefono
e far partire il modulo di procedura guidata giusto… Il primo modulo potrebbe
essere proprio il Samsung S26, e poi man mano aggiungere progressivamente
tutti gli altri moduli.» Un sistema **modulare**; un modulo può pesare anche
decine di MB.

## Come

**Phonestra = motore.** Riconosce il telefono, sceglie il modulo, lo esegue:
mostra le scene e osserva lo stato del telefono per avanzare da solo. Tutta la
conoscenza dei telefoni sta nei moduli, incorporati nell'AppImage (il
repository `ANDROLIN-DATA` previsto in origine è stato eliminato il 28 set
2026: niente aggiornamenti da internet, vedi `decisioni-utente.md`).

**Riconoscimento**, dal più sicuro:
1. col Debug USB attivo: `ro.product.model` (es. SM-S936B) e versione;
2. col cavo ma senza debug (anche in «Solo ricarica»): produttore dal codice
   USB, nome prodotto se c'è;
3. se non vede niente (caso del beta-tester): chiede «Che telefono hai?»
   (marche disegnate, poi serie o modello) e avvia quel modulo, che comincia
   proprio da cavo, «Trasferimento file» e protezioni.

**Scelta del modulo:** modello → stessa serie/versione del sistema («istruzioni
di un modello simile», detto chiaramente) → marca → procedura generica.

**Contenuto di un modulo:** solo dati e immagini, **mai codice**. Scene
descritte (schermata, righe, tocco da mostrare, testo, condizione per
avanzare: «Debug USB acceso», «telefono visto sul cavo»…), animate dal motore
col telefono disegnato; file multimediali (foto, video) ammessi dove un
disegno non basta, ma non come regola (invecchiano a ogni versione di One UI).
Niente loghi di marchi.

**Ereditarietà** per non copiare: marca → versione del sistema → modello
(solo le differenze).

**Distribuzione (superata dal 28 set 2026, vedi sopra):** `indice.toml` in ANDROLIN-DATA con pacchetti, versioni,
impronte e **firma** (chiave pubblica dentro Phonestra: la procedura dice cosa
toccare nel telefono, non deve poter essere falsificata). Scaricati in
`~/.cache/Phonestra/procedure/` all'avvio della procedura; una copia base
(generica + i moduli pronti) **dentro l'AppImage** per chi è senza rete.
Ogni pacchetto dichiara la versione del formato: un Phonestra vecchio ignora
ciò che non sa leggere.

## La guida in due tempi (27 set 2026, sera)

L'utente: «la parte più grossa la deve fare l'utente, quindi serve una guida in
cui vengono mostrati quali impostazioni cambiare e come farlo… brevi
animazioni»; e «il vero problema è che l'app può cominciare a lavorare solo
dopo che l'utente ha fatto le operazioni preliminari». Conseguenze:

- **Prima, col telefono in mano, senza cavo**: Blocco automatico spento,
  Numero build × 7, Debug USB acceso. Il PC non vede niente: l'utente va
  avanti da sé («Avanti»); la guida lo dice.
- **Poi, col cavo**: da qui lavora Phonestra e ogni passo avanza da solo
  (Trasferimento file solo se serve, Consenti con la spunta, rete). Niente
  «Avanti»; l'aiuto è «Non succede niente?» → diagnosi «Cosa vede il PC».
  Collegare il cavo dopo evita il caso del beta-tester (cavo attaccato col
  Blocco automatico acceso, PC che non vede niente).
- **Il modulo non si sceglie dal modello**: il modello si legge solo col Debug
  USB acceso, cioè a guida finita. Si parte da «Che telefono hai?» (marca,
  poi serie) o dalla marca letta dal cavo se è già attaccato. La parte
  preliminare dipende dalla marca e dalla versione del sistema (One UI), non
  dal singolo modello: il «modulo S26» è di fatto «Samsung, One UI 8».
- Ogni impostazione = **una breve animazione** (5–15 s) del telefono
  disegnato: il dito scorre, tocca, i 7 tocchi contati, il PIN; la riga
  delle istruzioni in corso si evidenzia insieme. Finisce da sola; «Rivedi» e
  «Più lento».
- Il Blocco automatico blocca anche il Debug wireless: resta spento finché si
  usa Phonestra (non dire «potrai riaccenderlo»).

Mockup: `mockup/proposte/guida-s26.html` — approvato dall'utente: «ECCEZIONALE! Altro che scrcpy!» (le scene sono già dati: `guida[]`
con righe, passi, gesti; si apre servendo la cartella in locale, in Chrome
`file://` non è raggiungibile dall'estensione). Nomi dei menu One UI da
verificare su un telefono vero (l'S23+ dell'utente, se ha la stessa One UI).

## I cancelli e le versioni del software (27 set 2026, sera)

L'utente: «stesso telefono ma versioni software diverse hanno menu diversi» →
le schermate dipendono dalla **versione del sistema** (One UI, HyperOS…), non
dal modello: la libreria si organizza per marca › versione, con le
differenze dei modelli sopra. Serve quindi sapere la versione, che l'utente
non conosce.

L'utente: «c'è sempre un'impostazione che viene prima delle altre» (il suo
amico era in «Solo ricarica»). Regola: il collegamento si apre per
**cancelli**, ognuno dà ad Phonestra un'informazione in più e la guida del
cancello successivo è più precisa:

0. **cavo dati** (cavo solo corrente, hub, porta) → la **marca** dal codice
   USB, anche in «Solo ricarica»;
1. **Trasferimento file** («Solo ricarica» è il predefinito a ogni
   collegamento) → via MTP **modello** e forse **versione** (da verificare
   sull'S23+: risposta MTP con i dati del dispositivo);
2. **Debug USB** (Blocco automatico, Opzioni sviluppatore) → tutto;
3. **Consenti** → Phonestra lavora.

Quindi il cavo torna **all'inizio**, per riconoscere il telefono (e col cavo
attaccato anche il Debug USB acceso si rileva da solo); se il PC non vede
niente → diagnosi → «Che telefono hai?» e guida della versione più probabile,
con «Sul tuo telefono è diverso?» → ricerca nelle Impostazioni. Chiedere la
versione all'utente (animazione su «Versione One UI») solo se MTP non la dà.
Da verificare: se il Blocco automatico con «Restrizioni massime» blocca anche
il Trasferimento file, è lui il primo cancello sui Samsung.
I mockup (`guida-s26.html`) hanno ancora l'ordine «prima il telefono, poi il
cavo»: da riordinare dopo la prova MTP.

**Aggiornamento, stessa sera:** strada principale **senza cavo** (associazione
col QR, SPECIFICHE §5.2): i cancelli 0 e 1 spariscono; resta il cancello 2
(configurazione preliminare, col Debug wireless) e il modello va chiesto. La
prova MTP perde importanza (serve solo alla riserva via cavo).

## Pagina «Prepara il telefono»: l'elenco come base (27 set 2026, sera)

L'utente: «una pagina principale di istruzioni dove si elenca cosa bisogna
impostare… poi sarà l'utente a cercare come fare sul proprio modello»; in
alternativa «tramite API web consultare un LLM free online». Valutazione:
- l'**elenco** regge ogni telefono e non invecchia → diventa la **base**; ma
  «cerca tu» sposta il ginepraio sull'utente inesperto → ogni voce dice anche
  **come si fa con la ricerca delle Impostazioni** (universale);
- **LLM dentro Phonestra: no** — chiave d'accesso nell'AppImage (estraibile),
  servizi gratuiti instabili, percorsi di menu inventati con sicurezza (per un
  inesperto peggio di niente), nessun controllo. Al suo posto, idea dell'utente («ha
  sicuramente un account Google: si può chiedere a Gemini»): **«Chiedi a
  Google ↗»** apre nel browser la **Modalità IA di Google Search**
  (`google.com/search?udm=50&q=…`, risposta di Gemini con le fonti) con la
  domanda già scritta e il modello scelto. Niente API, chiavi né costi; le
  fonti citate riducono il rischio di percorsi inventati. Provato il 27 set
  2026 con «Debug wireless su Galaxy S26, One UI 8.5»: risposta in italiano,
  passi giusti, Blocco automatico citato, video e fonti accanto (una
  imprecisione: «Versione build» invece di «Numero build»). Gemini.google.com
  non accetta la domanda nell'indirizzo (serve un'estensione): scartato.
Mockup `mockup/proposte/guida-elenco.html`: 5 voci (stessa rete Wi-Fi; Opzioni
sviluppatore; protezioni se ci sono; Debug wireless; codice a 6 cifre), ognuna
con perché, «Come si fa», «Chiedi a Google ↗», «Fatto/Già fatto».
Phonestra spunta da sé ciò che vede in rete (Debug wireless acceso, schermata
del codice aperta: il campo del codice si attiva). Prima dell'associazione
vede solo il numero di serie, non il modello. Le guide animate per modello
restano un di più («Guida illustrata ›»).
In discussione, non deciso: app di Phonestra sul telefono (Play Store) che apre
direttamente le pagine giuste e rileva gli stati.

### «Che telefono hai?» (mockup `mockup/proposte/guida-scelta.html`)

Primo schermo della guida, perché prima del Debug USB Phonestra non sa che
telefono è. Marca a schede (nome + telefono stilizzato, niente loghi; iPhone
presente ma attenuato: «Phonestra funziona solo con i telefoni Android», detto
subito invece che dopo dieci minuti; «Un altro / non lo so» → guida generica
con la ricerca delle Impostazioni). Se il cavo è già attaccato, la marca letta
dal codice USB è segnata «● sul cavo». Poi serie (Galaxy S, Z pieghevoli, A,
altri) e modello con l'anno; ogni modello dice se la guida è **provata** su
quel telefono o è quella della sua versione di One UI. «Non so il modello»
anima sul telefono disegnato dove leggerlo (Impostazioni › Informazioni sul
telefono, nome in alto). «Inizia» porta alla guida; la guida ha «‹ Cambia
telefono». Da verificare: nomi dei modelli 2025–26 e versioni di One UI
(scritti a memoria), codice SM- dell'S26.

### «Non succede niente?» (mockup `mockup/proposte/guida-diagnosi.html`)

Si apre dai passi col cavo. Non è un elenco generico: Phonestra sa fin dove
arriva il collegamento e propone solo i controlli di quel caso, uno alla
volta, ognuno con la sua animazione; appena il PC vede il telefono se ne
accorge da solo («Adesso ti vedo!» → torna alla guida). In alto la catena
PC ─ telefono ─ Debug USB ─ Phonestra col punto di rottura.
- **Il PC non vede niente sul cavo**: sblocca; altro cavo («molti portano solo
  la corrente»: se si carica ma il PC non lo vede, quasi sempre è il cavo);
  direttamente al PC, senza hub (al posto del telefono si disegna il PC);
  riavvia; chiedi aiuto (foto della finestra con l'elenco USB, ex «Cosa vede
  il PC» della beta 5).
- **Visto, Debug USB spento**: interruttore; Blocco automatico ancora acceso.
- **«Consenti» non compare**: sblocca e guarda; stacca e riattacca; revoca le
  autorizzazioni del Debug USB.
Un tentativo che non cambia niente si dichiara con «Fatto, non cambia niente ›».

## Primo modulo: Samsung Galaxy S26

Caso guida il telefono del beta-tester. Scene: collegare il cavo; «Trasferimento
file» e «Consenti» l'accesso ai dati; Blocco automatico spento (con
«Restrizioni massime»); Numero build × 7; Debug USB; «Consenti sempre da questo
computer»; rete Wi-Fi. Si parte da un **mockup animato** in `mockup/proposte/`
(animazioni al clic, finite), costruito già col formato delle scene.

## Da valutare

- Associazione senza cavo col QR del Debug wireless (Android 11+): toglie
  cavo, porte e «Solo ricarica»; serve il protocollo di associazione (SPAKE2)
  nel nostro ADB.
- La diagnosi «Cosa vede il PC» (beta 5) diventa una scena del modulo.
