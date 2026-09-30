# Phonestra

Programma per Linux che fa **usare le app del telefono Android in finestre del
desktop**, via Wi-Fi e senza installare niente sul telefono. GTK4 + libadwaita,
distribuito come AppImage. Telefoni con **Android 14 o successivo**.

**Stato:** versione **1.0.0-rc.8**, candidata alla 1.0 (Release su GitHub).
`phonestra` apre il drawer con le app, le notifiche e lo schermo del telefono;
ogni app si apre nella sua finestra, con audio, mouse, tastiera, appunti,
screenshot e registrazione. I file passano nei due sensi: si trascinano sul
telefono (un `.apk` si installa) e si ricevono dal telefono con «Ricevi file…». Tutto quello che gira sul telefono è codice
nostro (`android/`). `phonestra-prova` fa prove e diagnosi da riga di comando.

## Documentazione

I due manuali sono **in inglese** (il resto del progetto, interfaccia compresa, è
in italiano):

- **Manuale utente**: [`docs/User Manual.html`](docs/User%20Manual.html): installazione,
  primo collegamento, uso di tutti i giorni, risoluzione dei problemi.
- **Manuale tecnico**: [`docs/Technical Manual.html`](docs/Technical%20Manual.html) (aprirlo
  nel browser dopo aver clonato il repository; vedi [`docs/README.md`](docs/README.md)):
  architettura, client ADB, componente sul telefono, video, pannello, audio, input,
  interfaccia, AppImage, prove, convenzioni. Sono generati da `docs/sources/`
  (`python3 docs/sources/build.py`).
- `SPECIFICATION.md`: cosa fa Phonestra. `notes/`: il perché delle decisioni.

## Compilare e provare

Serve Rust (installato nella cartella utente con `rustup`, vedi sotto) e, per
l'interfaccia, i pacchetti di sviluppo di GTK4, libadwaita e GStreamer
(`libgtk-4-dev libadwaita-1-dev libgstreamer1.0-dev
libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-bad gstreamer1.0-libav
gstreamer1.0-gtk4`). L'aiutante per il telefono è già compilato in
`android/phonestra-helper.jar` (per ricompilarlo: `android/helper/build.sh`).

```bash
cargo build
./target/debug/phonestra-prova usb       # stato del telefono col cavo
./target/debug/phonestra-prova cerca     # telefoni col Debug wireless in rete
./target/debug/phonestra-prova prepara   # via cavo: Wi-Fi acceso, telefono salvato
./target/debug/phonestra-prova collega   # via Wi-Fi, al telefono salvato
./target/debug/phonestra-prova file /sdcard/Download   # una cartella del telefono
./target/debug/phonestra                 # drawer e app in finestre
cargo test && cargo clippy
```

Gli stessi comandi con `make`: `make` (compila), `make test` (prove e controllo
dei manuali), `make clippy`, `make docs` (rigenera i manuali). Su una macchina
nuova `tools/setup-dev.sh` dice quali pacchetti mancano (`--install` li installa).

Phonestra tiene chiave e telefoni in `~/.config/Phonestra` (cancellarla = ripartire
da zero; il telefono andrà autorizzato di nuovo).

## Contenuto

| Percorso | Cosa |
|---|---|
| [`SPECIFICATION.md`](SPECIFICATION.md) | Il documento di specifiche: tutte le decisioni, prove fatte e da fare |
| [`notes/`](notes/) | Il perché delle decisioni, i dettagli delle prove, la storia dell'interfaccia. **Da leggere prima di cambiare direzione.** |
| [`notes/issue-log.md`](notes/issue-log.md) | Problemi riscontrati, cause, soluzioni e stato |
| [`notes/component.md`](notes/component.md) | Il componente nostro per il telefono: architettura, messaggi, prove |
| [`notes/study/`](notes/study/) | Studio di Android 14+ (video, audio, input, sistema) prima del componente |
| [`mockup/`](mockup/) | I mockup dell'interfaccia (sorgenti del canvas e icone) e il link al canvas pubblicato |
| [`logos/`](logos/) | Il logo di Phonestra |
| [`experiments/`](experiments/) | Strumenti usati nelle prove (ricerca del telefono in rete via mDNS) |
| [`android/`](android/) | Il componente del telefono: sorgenti Java e `phonestra-helper.jar` già compilato |
| [`packaging/`](packaging/) | Contenitore e script dell'AppImage |
| [`tools/`](tools/) | `setup-dev.sh` (cosa manca su una macchina nuova), `backup.sh` (tutto il repository in un file) |
| [`Makefile`](Makefile) | `make`, `make test`, `make clippy`, `make docs`, `make docs-check`, `make dist`, `make clean` |
| [`NOTICE.md`](NOTICE.md) | Copyright e componenti di terzi (crate Rust, librerie dell'AppImage) |
| [`CLAUDE.md`](CLAUDE.md) | Istruzioni per le sessioni di Claude Code su questo progetto |

## Licenza

Gratis per uso personale; uso in aziende o per lavoro, modifica,
redistribuzione e uso commerciale vietati senza accordo scritto con l'autore:
vedi [`LICENSE.md`](LICENSE.md). Anche il componente che gira sul telefono è
di Phonestra, con la stessa licenza: non ci sono componenti di terzi.
Il programma per il PC usa librerie di terzi con le loro licenze: vedi
[`NOTICE.md`](NOTICE.md).

## Ripristino da zero

Questo repository è l'unica copia completa del progetto. Su una macchina nuova:

```bash
git clone https://github.com/nic-fio/PHONESTRA.git ~/Documenti/PHONESTRA
cd ~/Documenti/PHONESTRA
```

Per compilare serve Rust, installato solo nella cartella utente:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal -c clippy -c rustfmt
. ~/.cargo/env
```

(serve anche `build-essential` per compilare libusb).

Fuori dal repository, da ricreare (nessuno contiene dati da salvare):

- `strumenti/r8.jar` (D8 di Google, per l'aiutante del telefono): indirizzo e
  impronta sha256 in `android/helper/build.sh`; serve anche `javac`
  (`sudo apt install default-jdk-headless`). Poi `sh android/helper/build.sh`.
- `target/`: la crea `cargo build`.
- AppImage: contenitore `podman build -t phonestra-appimage packaging`, poi il
  comando in cima a `packaging/collect.sh` (o tutto insieme: `make dist`).
- `~/.config/Phonestra` (chiave ADB e telefoni associati): è personale e non va
  salvata; su un PC nuovo si riassocia il telefono con «Aggiungi telefono»
  (codice a 6 cifre del Debug wireless, `SPECIFICATION.md` §5.2).

Per ripetere le prove manuali servono inoltre `python3` (per
`experiments/mdns-find-phone.py`) e, solo per le diagnosi, il pacchetto `adb`.

Se il canvas dei mockup non fosse più raggiungibile, `mockup/README.md` spiega
come ricrearlo dai file del repository.
