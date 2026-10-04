# Da dove ripartire (aggiornato il 4 ottobre 2026)

## ▶ Ripartire esattamente da qui

**4 ottobre 2026: verso la 1.0.0-rc.9 e il sito `phonestra.nicfio.it`.**
Il 3 ott sera il tablet si è bloccato (memoria piena: 7,5 GB senza swap, e
`/tmp` è un tmpfs nella RAM): niente era perso, i lavori a metà sono stati
ripresi e chiusi. Regola: al massimo due aiutanti alla volta, niente
compilazioni in `/tmp`, `make dist` da solo.

Fatto (tutto committato):
- **libusb collegata dinamicamente** (`vendor/rusb/`, NOTICE): nell'AppImage
  è `usr/lib/libusb-1.0.so.0`, provata su Debian 12, Fedora, Arch.
- **Interfaccia in italiano e inglese** (SPECIFICATION §15.1): tutti i moduli
  tradotti (`data/en/*.toml`), scheda «Lingua» nelle Preferenze, prove in tutte
  e due le lingue.
- **Nome dei telefoni nella lingua dell'interfaccia** (§6): «Telefono»/«Tablet»
  (in inglese «Phone»), col modello accanto se sono più d'uno; il nome del
  dispositivo scelto sul telefono non si mostra più; sezione «I miei telefoni».
- **Casella `phonestra@nicfio.it`**: alias di `nesh@nicfio.it` in Zimbra (OVH),
  come `efipm@`.
- **Pagina iniziale**: scelto il mockup 12 «Vivid» → `site/landing/index.html`
  (`@VERSION@` riempito da `publish.sh`), `site/og.png`.

Da fare, in ordine:
1. Controllo a occhio dell'utente del programma in inglese.
2. **Manuali**: con l'interfaccia bilingue le etichette si citano in inglese,
   con l'italiano tra parentesi (decisione del 4 ott); allineare «Aggiorna
   ora» (oggi tradotto in due modi). Nomi inglesi delle impostazioni scritti a
   memoria per Xiaomi, Oppo/OnePlus, Honor, Pixel: da verificare.
3. **Sito in inglese**: le 90 scritte dei disegni e le 20 etichette del testo
   con le traduzioni vere (`data/en/`), il telefono dei disegni «Phone», via la
   frase «The interface is in Italian…» (notes/user-decisions.md, sito).
4. **1.0.0-rc.9**: versione, `make dist` (da solo), `packaging/test-distributions.sh`.
5. `site/publish.sh` (pagina, manuali, licenza, AppImage).
6. Solo dopo: repository privato.

---

**30 settembre 2026, sera: struttura standard dei sette progetti.** Solo nomi
di file e cartelle, in inglese (il contenuto resta in italiano): `memoria/` →
`notes/` (questo file era `prossima-sessione.md`), `SPECIFICHE.md` →
`SPECIFICATION.md`, `telefono/aiuto/` → `android/helper/`
(`phonestra-helper.jar`), `costruzione/` → `packaging/`, `dati/` → `data/`,
`grafica/` → `logos/`, `prove/` → `experiments/`, `docs/sorgenti/` →
`docs/sources/`, `mockup/proposte/` → `mockup/proposals/`. Nuovi: `Makefile`,
`NOTICE.md`, `tools/`, `docs/index.html`, CI su GitHub. I moduli Rust e le
classi Java restano coi loro nomi.

**Aggiornato il 29 settembre 2026.** **Phonestra 1.0.0-rc.4 funziona
interamente col componente nostro** (audio, video, input, appunti): scrcpy è
stato tolto dal codice, dall'AppImage e dalla licenza (fase 3, `component.md`
§15). Problemi e soluzioni della giornata: `issue-log.md`; misure:
`connection-tests.md` §41–51.

Regole scoperte oggi da non perdere:
- la cattura audio parte **5 s dopo lo specchio del drawer** e riparte se lo
  specchio si ricrea (§49);
- blocchi ADB da **64 KiB** (audio in sincrono col video, §50);
- il custode riaccende il pannello **in Java** (niente blocco del telefono, §51).

**Fatto**: AppImage 1.0.0-rc.1 costruita (contenitore `phonestra-appimage`,
comando in `packaging/collect.sh`) e pubblicata come pre-release
`v1.0.0-rc.1` su GitHub, poi sostituita dalla **1.0.0-rc.2** (correzione
dell'audio dopo un ricollegamento, prove §53) e dalla **1.0.0-rc.3** (logo
ufficiale nella finestra «Informazioni» e nell'icona dell'AppImage) e dalla
**1.0.0-rc.4** (pannello che non si spegneva più dopo mezz'ora, tempo di
spegnimento dell'utente preservato alla chiusura: prove §55–56). La 1.0
definitiva dopo la conferma del beta-tester.
Dal 29 set sera nella **1.0.0-rc.5** (pubblicata come pre-release `v1.0.0-rc.5`): display che restava
acceso dopo una caduta di rete (prove §57), provato anche con blocco e
sblocco a mano. Poi (§58): pannello rispento dopo 2 min senza tocchi (provato)
e acceso alla chiamata in arrivo (da provare con una chiamata vera).
Scritto il **manuale tecnico**, ora `docs/Technical Manual.html`, con
accanto `docs/User Manual.html` (stile dei manuali di IR_Service, decisione
dell'utente del 30 set; tradotti in inglese e rinominati lo stesso giorno). Dal 30 set sera sono
**generati** da `docs/sources/` (un capitolo per file, stile dei manuali di
AMS, decisione dell'utente) e allineato al codice di nuovo: mancavano il
pannello (§57–60: chiamate, «in mano», spegnimento senza tocchi, cadute),
preferenze, più telefoni, comandi di prova e varie correzioni. Si rigenera con
`python3 docs/sources/build.py`; `cargo test` controlla mappa, simboli, file,
variabili e comandi citati, non i comportamenti: quando cambia un comportamento
si aggiorna il capitolo nello stesso commit.
Trovati rileggendo il codice per il manuale, **da verificare con l'utente**:
avvisi del sistema per tutte le notifiche già presenti alla prima lettura
(`avvisa_nuove` fissa le «già viste» con l'elenco ancora vuoto); col drawer
aperto la chiusura dell'ultima finestra non riaccende il pannello (lo specchio
conta come sessione); `packaging/test-distributions.sh` cerca
`procedura.png` ma con configurazione vuota si apre `prepara`.
Scrivendo il manuale utente (30 set) risultano in `SPECIFICATION.md` ma non nel
codice: QR, passaggio automatico al cavo, «Spegni il Debug wireless alla
chiusura» («In arrivo»), blocco del telefono alla chiusura (§5.9), schermata
«Telefono non raggiungibile» (§5.1), avviso «App aperta sul telefono» (§7.6),
rilevamento dei firewall (§5.4), icona nell'area di notifica (§7.1), scheda di
conferma dell'apk (§11), pulsanti «Passa a…»/«Nascondi» negli avvisi (§8),
notifiche dei telefoni non attivi (§6), «Connessione debole» (§14), AppImage
aarch64. Anche: il pulsante «Android 10 o precedente? Collega col cavo» contro
il requisito Android 14+, e la preferenza «Elenco delle app: si aggiorna da
solo» (si rilegge solo al collegamento e dopo installazioni). Da decidere con
l'utente: SPECIFICATION da correggere o funzioni da fare.
Per l'AppImage servono **sempre** `cargo build --release` nel contenitore e
poi `collect.sh` (comando nel manuale, capitolo AppImage): il 28 set
`collect.sh` da solo ha impacchettato un eseguibile vecchio.

Dal 29 set notte nella **1.0.0-rc.6** (pubblicata come pre-release `v1.0.0-rc.6`, copia in `~/`): micro-interruzioni
dei reel risolte (prove §59): allo spegnimento del pannello il display dei Samsung
restava a 24 Hz; ora `Pannello.java` lo porta a 60 Hz per un istante
(`min_refresh_rate`) prima di spegnere. Da provare dentro Phonestra lo
spegnimento col display a riposo (riga «frequenza del display a 60 Hz» nel registro).

Dal 30 set nella **1.0.0-rc.7** (pubblicata come pre-release `v1.0.0-rc.7`, copia in `~/`; dal 30 set è l'unica release su GitHub, le rc.3–rc.6 tolte, etichette git tenute): pannello acceso per sempre dopo una
chiamata risposta con un clic dal PC, e con nessuna finestra aperta (prove
§60, da provare con una chiamata vera). Il registro di Phonestra è nel
journal del PC: `journalctl --user --since today | grep -i phonestra`.

Dal 30 set sera nella **1.0.0-rc.8** (pubblicata come pre-release `v1.0.0-rc.8`, copia in `~/`; dal 30 set sera è l'unica release su GitHub, la rc.7 tolta, etichetta git tenuta) **«Ricevi file…»**:
file dal telefono al PC (SPECIFICATION §11.1, mockup `receive-files.html`, prove
§61), provato dall'utente («trasferimento da telefono a PC OK»). Da provare:
scheda SD, annullamento, cartelle grandi, tema scuro;
in prova per un paio di giorni con l'utente e il beta-tester.

**Prossimi passi**: risposta del beta-tester; prove manuali di
`phase2-tests-todo.md`. Poi (il volume che resta al massimo è
accettato dall'utente, `user-decisions.md`): il margine audio che cresce ma non scende; *delayed ack*
(`adb.md`); memoria del servizio ~145 MB; caduta del Wi-Fi del PC da provare;
registrazione AAC da provare.

### Storia della giornata (superata)

**La prossima sessione è dedicata** (decisione dell'utente, 27 set sera) a
**definire, progettare e realizzare i moduli della guida al primo
collegamento**. Punto di partenza: `notes/procedure-library.md` (perché,
decisione dell'utente, architettura: Phonestra = motore che riconosce marca e
modello e avvia il modulo; moduli nel repository dei dati, solo dati e immagini,
firmati, con copia base nell'AppImage; ripieghi «Che telefono hai?» e «modello
simile»). Primo modulo: **Samsung Galaxy S26**, dal caso del beta-tester.
Ordine proposto: formato dei moduli e delle scene → mockup animato del
modulo S26 (in `mockup/proposals/`, animazioni al clic e finite) → motore in
Phonestra → firma e scaricamento dal repository dei dati → prova. Chiedere all'utente
se è arrivata la foto del riquadro «Cosa vede il PC» dal suo amico.

Stato a fine giornata: tutto committato; AppImage beta 5 in
`~/Phonestra-0.1.0-x86_64.AppImage` e nelle Release (v0.1.0-beta.1…5).

**Beta-tester, esito provvisorio** (sera del 27 set): Galaxy S26 su Zorin OS
(PC fisico), a 700 km: fermo al passo 1 anche con «Trasferimento file» (il PC
non vede il telefono: cavo, porta o Restrizioni massime del Blocco
automatico). Mandata la beta 5 col riquadro «Cosa vede il PC»: aspettiamo la
foto. **Prossimo lavoro deciso con l'utente**: libreria modulare di procedure
guidate (`notes/procedure-library.md`): Phonestra riconosce il telefono e
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
`mockup/proposals/`, decisioni e perché in `notes/interface.md`) ed è
entrata nel programma. L'utente ha provato e confermato («mi sembra che sia
tutto OK»): drawer nuovo con tutte le voci attive, **schermo vero del
telefono nel drawer** (interattivo: WhatsApp, YouTube), Preferenze, avvisi a
comparsa, invio file. Dettagli in prove-collegamento §23–25. Creato il
repository pubblico dei dati (§5.2, poi eliminato).

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
  normale mentre il telefono è in mano (dal 29 set resta al massimo, prove §55).
- AppImage (podman, glibc 2.35); icona di Phonestra (area di notifica / dock).
- Da verificare: messaggio delle app protette con Bitwarden in finestra; un
  avvio che si è chiuso da solo dopo 2 s (non ripetuto).
