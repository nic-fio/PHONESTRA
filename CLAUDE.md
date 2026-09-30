# Phonestra — istruzioni per Claude Code

- Con l'utente si parla **in italiano**; anche i documenti del progetto sono in
  italiano.
- All'inizio di una sessione leggere **`memoria/prossima-sessione.md`**: dice
  da dove ripartire.
- Prima di proporre o cambiare qualcosa, leggere `SPECIFICHE.md` e la cartella
  `memoria/`: lì ci sono le decisioni già prese **e il loro perché**. Non
  riproporre alternative già scartate (es. Flatpak, icone nel menu di sistema,
  temi scuri come stile, più telefoni attivi insieme).
- Il repository `nic-fio/PHONESTRA` è l'unica copia completa del progetto: ogni
  materiale nuovo (documenti, mockup, script di prova) va committato e spinto
  qui. È (o diventerà) **pubblico**: niente dati personali (nomi, numeri di
  serie, nomi di reti Wi-Fi, indirizzi, schermate vere non sfocate).
- Quando si aggiorna il canvas dei mockup, aggiornare anche i file in
  `mockup/canvas/` (e viceversa): il repository deve bastare a ricrearlo.
- Nei mockup: niente loghi di marchi reali, niente animazioni CSS infinite
  (bloccano il canvas). Mostrando il canvas nel browser, lasciare aperta la
  scheda: impiega 20–30 s a comparire.
- Codice in Rust: `cargo` sta in `~/.cargo/bin` (aggiungerlo al PATH nei
  comandi). Prima di ogni commit: `cargo build`, `cargo test`, `cargo clippy`
  senza avvisi. Commenti e messaggi del programma in italiano.
- Per le prove si usa `phonestra-prova` quando basta; `adb` di sistema solo per
  diagnosi che il prototipo non sa ancora fare (usa un'altra chiave: il telefono
  chiede una nuova autorizzazione).
- A fine prova chiudere il server `adb` eventualmente avviato e le sessioni
  Wi-Fi aperte; non lasciare modificate impostazioni del telefono (es. tempo di
  spegnimento dello schermo).
- I manuali (`docs/Phonestra_Manuale_Tecnico.html` e
  `docs/Phonestra_Manuale_Utente.html`) sono **generati** da `docs/sorgenti/`
  (un file per capitolo in `tecnico/` e `utente/`; nomi, stile e struttura dei
  manuali di IR_Service): mai modificarli a mano. Quando cambia un
  comportamento, aggiornare nello stesso commit i capitoli che lo descrivono
  (tecnico e, se l'utente lo vede, utente), poi
  `python3 docs/sorgenti/build.py`. `cargo test` rigenera e controlla (mappa dei
  file, simboli, file, variabili `PHONESTRA_*` e comandi citati), ma non vede i
  comportamenti cambiati.
