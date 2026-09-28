# Da dove ripartire (aggiornato il 27 settembre 2026, ore 23)

## ▶ Ripartire esattamente da qui

**Domani (28 set)**: l'utente porta **una funzionalità nuova** che gli è venuta
in mente a fine sessione: chiedergliela per prima cosa (prima leggere
`SPECIFICHE.md` e `memoria/decisioni-utente.md`).

**Stato: versione 0.2.0 pubblicata** (etichetta `v0.2.0`, release con
`Phonestra-0.2.0-x86_64.AppImage`, copia anche in `~`). Primo collegamento
**senza cavo**: finestra «Prepara il telefono» (`src/prepara.rs`, SPECIFICHE
§5.2) con 4 voci (stessa rete Wi-Fi, Opzioni sviluppatore, protezioni se ci
sono, Debug wireless con interruttore + codice a 6 cifre); per ogni voce la
parola da cercare («usa lo strumento di ricerca delle Impostazioni…») e
«Chiedi a Google ↗» (Modalità IA, `udm=50`); spunte automatiche via mDNS;
associazione nostra (`src/adb/abbina.rs`, SPAKE2 come `adb pair`). Provata
dall'utente (20 s) e dal beta-tester a 700 km (circa 5 minuti, nessun
blocco: prove §40). Procedura col cavo solo per Android 10 o precedente.

**Decisioni della sera** (dettagli in `decisioni-utente.md`): scelta
radicale «solo l'elenco, poi l'utente capisce come farlo»; niente cavo;
niente LLM dentro Phonestra; **niente disegni dell'interfaccia del telefono,
nemmeno «universali»** (la ricerca sui Samsung è una casella in basso).
Libreria di moduli e guide animate accantonate (`libreria-procedure.md`,
mockup `mockup/proposte/guida-*.html`).

**Aperti**:
- «rendere definitivo»: riquadro rapido «Debug wireless» aggiunto dalla shell
  (`cmd statusbar add-tile …DevelopmentTiles$WirelessDebugging`; il comando
  esiste, abilitare il riquadro da provare), perché senza cavo, quando il Debug
  wireless si spegne, basti un tocco;
- righe orizzontali nel drawer su Zorin (prove §36): chiedere al beta-tester
  versione di Zorin, se cambiano col mouse, ridimensionamento dello schermo;
- sul telefono dell'utente restano voci di prova in Debug wireless ›
  Dispositivi associati (adb di sistema, «Phonestra@…»): può toglierle.

### Storia della giornata (superata)

**La prossima sessione è dedicata** (decisione dell'utente, 27 set sera) a
**definire, progettare e realizzare i moduli della guida al primo
collegamento**. Punto di partenza: `memoria/libreria-procedure.md` (perché,
decisione dell'utente, architettura: Phonestra = motore che riconosce marca e
modello e avvia il modulo; moduli in ANDROLIN-DATA, solo dati e immagini,
firmati, con copia base nell'AppImage; ripieghi «Che telefono hai?» e «modello
simile»). Primo modulo: **Samsung Galaxy S26**, dal caso del beta-tester.
Ordine proposto: formato dei moduli e delle scene → mockup animato del
modulo S26 (in `mockup/proposte/`, animazioni al clic e finite) → motore in
Phonestra → firma e scaricamento da ANDROLIN-DATA → prova. Chiedere all'utente
se è arrivata la foto del riquadro «Cosa vede il PC» dal suo amico.

Stato a fine giornata: tutto committato; AppImage beta 5 in
`~/Phonestra-0.1.0-x86_64.AppImage` e nelle Release (v0.1.0-beta.1…5).

**Beta-tester, esito provvisorio** (sera del 27 set): Galaxy S26 su Zorin OS
(PC fisico), a 700 km: fermo al passo 1 anche con «Trasferimento file» (il PC
non vede il telefono: cavo, porta o Restrizioni massime del Blocco
automatico). Mandata la beta 5 col riquadro «Cosa vede il PC»: aspettiamo la
foto. **Prossimo lavoro deciso con l'utente**: libreria modulare di procedure
guidate (`memoria/libreria-procedure.md`): Phonestra riconosce il telefono e
avvia il modulo; primo modulo **Samsung Galaxy S26**, da un mockup animato.

**Metodo concordato** (l'utente: «si aggiusta una cosa e se ne rompe
un'altra»): un cambiamento alla volta, provato col monitoraggio (registro con
l'ora + Monitor) su YouTube **e** Facebook prima del successivo.

## Regole pratiche

- **Prove**: usare la versione ottimizzata (`cargo build --release`,
  `target/release/phonestra`), con `PHONESTRA_DEBUG=1` per le misure e
  `PHONESTRA_FOTO=<cartella>` per le immagini delle finestre. Il banco di prova
  `phonestra-prova banco …` misura i fotogrammi senza l'utente (§20).
  La finestra «Aggiungi un telefono» si prova senza cavo con
  `PHONESTRA_PROVA_PASSO=debug|consenti|xiaomi|wifi|fatto phonestra-prova procedura`.
- **Riavvii**: chiedere prima all'utente. Ogni chiusura di Phonestra rimette a
  posto il telefono e **lo blocca**; non lanciare istanze di prova mentre la
  sua è aperta. Phonestra si chiude bene con SIGTERM (cercare il processo con
  `pgrep -x phonestra`: `pkill -f` colpisce anche il proprio terminale).
- **Etichetta** `v0.1.0` sulla prima tappa interna; il lavoro è sul ramo `main`.

## Stato

Il 26–27 set è stata scelta una **nuova direzione, stile «vetro»** (mockup in
`mockup/proposte/`, decisioni e perché in `memoria/interfaccia.md`) ed è
entrata nel programma. L'utente ha provato e confermato («mi sembra che sia
tutto OK»): drawer nuovo con tutte le voci attive, **schermo vero del
telefono nel drawer** (interattivo: WhatsApp, YouTube), Preferenze, avvisi a
comparsa, invio file. Dettagli in prove-collegamento §23–25. Creato il
repository pubblico `nic-fio/ANDROLIN-DATA` (§5.2).

Funzionava già prima: app in finestre (più insieme), ridimensionamento,
ricollegamento automatico; mouse, rotellina, zoom, tastiera, clic destro =
pressione lunga; appunti nei due sensi (password escluse); audio dal PC in
sincronia; video fluidi.

Non ancora provati col telefono: la procedura «Aggiungi telefono» col cavo
vero; il passaggio fra due telefoni (ne serve un secondo); il rispegnimento
del pannello al primo clic dal PC dopo uno sblocco a mano.

## In sospeso

- **Righe orizzontali nel drawer** sul PC del beta-tester (Zorin): prove §36,
  in attesa delle sue risposte.

- **Finestre delle app**: Screenshot e Registra verificati il 27 set
  (`~/Immagini/Phonestra/LibreLink … .png`, `~/Video/Phonestra/Facebook … .mp4`,
  14 s, 1120×1992, 24 fps); dal 27 set sera anche **con l'audio** del
  telefono (Opus senza ricodifica, allineato: provato dall'utente).

## Dopo

- Tema scuro da rifinire; «Mostra sul telefono» e
  «App aperta sul telefono» nelle finestre (da verificare se Android lo
  permette dalla shell).
- Tempo di spegnimento
  normale mentre il telefono è in mano (oggi resta a 30 min).
- AppImage (podman, glibc 2.35); icona di Phonestra (area di notifica / dock).
- Da verificare: messaggio delle app protette con Bitwarden in finestra; un
  avvio che si è chiuso da solo dopo 2 s (non ripetuto).
