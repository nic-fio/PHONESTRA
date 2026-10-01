# Phonestra — istruzioni per Claude Code

- Con l'utente si parla **in italiano**; anche i documenti del progetto sono in
  italiano, **tranne i due manuali**: `docs/User Manual.html` e
  `docs/Technical Manual.html`, e i loro sorgenti in `docs/sources/` (il testo dei
  capitoli), sono **in inglese**. Tutto il resto (conversazione, README,
  SPECIFICATION, `notes/`, commenti, nomi nel codice, messaggi e interfaccia del
  programma) resta in italiano; nei manuali le etichette dell'interfaccia si
  citano come appaiono, con la traduzione tra parentesi.
- All'inizio di una sessione leggere **`notes/next-session.md`**: dice
  da dove ripartire.
- Prima di proporre o cambiare qualcosa, leggere `SPECIFICATION.md` e la cartella
  `notes/`: lì ci sono le decisioni già prese **e il loro perché**. Non
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
  comandi; il `Makefile` lo fa da sé). Prima di ogni commit: `cargo build`, `cargo test`, `cargo clippy`
  senza avvisi (`make`, `make test`, `make clippy`). Commenti e messaggi del programma in italiano.
- Per le prove si usa `phonestra-prova` quando basta; `adb` di sistema solo per
  diagnosi che il prototipo non sa ancora fare (usa un'altra chiave: il telefono
  chiede una nuova autorizzazione).
- A fine prova chiudere il server `adb` eventualmente avviato e le sessioni
  Wi-Fi aperte; non lasciare modificate impostazioni del telefono (es. tempo di
  spegnimento dello schermo).
- I manuali (`docs/Technical Manual.html` e `docs/User Manual.html`) sono
  **generati** da `docs/sources/` (un file per capitolo in `technical/` e
  `user/`): mai modificarli a mano. Lo stile è lo **stile comune dei manuali dei
  sette progetti**, incorporato in ogni pagina e identico in tutti:
  `docs/sources/style.css` e `docs/sources/manual.js` sono il canone, copiati
  byte per byte, e non si modificano in un solo progetto; le sole aggiunte di
  Phonestra (elementi che esistono solo qui) stanno in `EXTRA_CSS` di
  `build.py`, dopo il canone. Copertina: logo, «User Manual»/«Technical
  Manual», Version e Date; piè di pagina `Phonestra · User Manual · Version X ·
  Mese AAAA · © 2026 Nicola Fiorillo · Licence: free for personal use`. Quando cambia un
  comportamento, aggiornare nello stesso commit i capitoli che lo descrivono
  (tecnico e, se l'utente lo vede, utente), poi
  `python3 docs/sources/build.py`. `cargo test` rigenera e controlla (mappa dei
  file, simboli, file, variabili `PHONESTRA_*` e comandi citati, testo rimasto
  in italiano), ma non vede i comportamenti cambiati.

## Il repository

| Dove | Cosa |
|---|---|
| `src/` | Il programma per il PC (moduli Rust con nomi italiani, come il codice). |
| `tests/` | Prove d'integrazione (`tests/manual.rs`: controllo dei manuali). |
| `android/` | Il componente del telefono: sorgenti Java in `android/helper/` (`build.sh` lo ricompila), `phonestra-helper.jar` già compilato e incluso nell'eseguibile. |
| `data/instructions.toml` | Istruzioni per marca della procedura col cavo, incluse nell'eseguibile. |
| `packaging/` | Contenitore, script (`build.sh`, `collect.sh`, `rust-licenses.py`, `test-distributions.sh`) e `AppRun` dell'AppImage, che porta con sé le licenze dei componenti di terzi (`usr/share/doc/phonestra/`). |
| `docs/` | `index.html`, i due manuali (generati da `docs/sources/`), `README.md`, `.nojekyll`. |
| `notes/` | Il perché delle decisioni, studi, misure, registro dei problemi (`next-session.md`, `user-decisions.md`, `issue-log.md`, `connection-tests.md`, `component.md`, `study/`…). |
| `logos/` | Il logo di Phonestra (`icons/`, `icons-with-text/`). |
| `mockup/` | Proposte dell'interfaccia (`proposals/`), icone (`icons/`), sorgenti del canvas (`canvas/`). |
| `experiments/` | Strumenti delle prove fuori dal programma (`mdns-find-phone.py`). |
| `tools/` | `setup-dev.sh` (pacchetti, Rust, identità git), `backup.sh` (bundle git). |
| `SPECIFICATION.md`, `LICENSE.md`, `NOTICE.md` | Specifiche (i «§» del codice), licenza, componenti di terzi. |
| `Makefile` | `make` (build), `make test`, `make clippy`, `make docs`, `make docs-check`, `make dist` (AppImage), `make helper` (jar), `make clean`. |

Il repository segue la struttura standard comune ai sette progetti (AMS,
EFI_PARTITION_MANAGER, HOSTER, MTERM, NESH, PHONESTRA, SCRAPER): `src/`,
`tests/`, `tools/`, `docs/` (con `index.html`, i due manuali, `README.md`,
`.nojekyll`), `NOTICE.md`, la CI (`.github/workflows/ci.yml`) e gli obiettivi
comuni di `make`: `all`, `test`, `docs-check`, `clean`. Nomi di file e
cartelle in inglese, tranne quelli che coincidono con identificatori del
codice (moduli Rust, classi Java); il contenuto dei documenti resta in
italiano.
