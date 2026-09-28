# Phonestra

*(nome provvisorio)*

Programma per Linux che fa **usare le app del telefono Android in finestre del
desktop**, via Wi-Fi e senza installare niente sul telefono. GTK4 + libadwaita,
distribuito come AppImage.

**Stato:** tappe 1–3 in prova — `phonestra` apre il drawer con le app del
telefono (icone vere, ricerca); ogni app si apre nella sua finestra, con
mouse, rotellina, tastiera, ridimensionamento e ricollegamento automatico.
`phonestra-prova` prepara il telefono via cavo (prima volta) e fa diagnosi.

## Compilare e provare

Serve Rust (installato nella cartella utente con `rustup`, vedi sotto) e, per
l'interfaccia, i pacchetti di sviluppo di GTK4, libadwaita e GStreamer
(`libgtk-4-dev libadwaita-1-dev libgstreamer1.0-dev
libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-bad gstreamer1.0-libav
gstreamer1.0-gtk4`). L'aiutante per il telefono è già compilato in
`telefono/phonestra-aiuto.jar` (per ricompilarlo: `telefono/aiuto/costruisci.sh`).

```bash
cargo build
./target/debug/phonestra-prova usb       # stato del telefono col cavo
./target/debug/phonestra-prova cerca     # telefoni col Debug wireless in rete
./target/debug/phonestra-prova prepara   # via cavo: Wi-Fi acceso, telefono salvato
./target/debug/phonestra-prova collega   # via Wi-Fi, al telefono salvato
./target/debug/phonestra                 # drawer e app in finestre
cargo test && cargo clippy
```

Phonestra tiene chiave e telefoni in `~/.config/Phonestra` (cancellarla = ripartire
da zero; il telefono andrà autorizzato di nuovo).

## Contenuto

| Percorso | Cosa |
|---|---|
| [`SPECIFICHE.md`](SPECIFICHE.md) | Il documento di specifiche: tutte le decisioni, prove fatte e da fare |
| [`memoria/`](memoria/) | Il perché delle decisioni, i dettagli delle prove, la storia dell'interfaccia. **Da leggere prima di cambiare direzione.** |
| [`mockup/`](mockup/) | I mockup dell'interfaccia (sorgenti del canvas e icone) e il link al canvas pubblicato |
| [`grafica/`](grafica/) | Il logo di Phonestra |
| [`prove/`](prove/) | Strumenti usati nelle prove (ricerca del telefono in rete via mDNS) |
| [`CLAUDE.md`](CLAUDE.md) | Istruzioni per le sessioni di Claude Code su questo progetto |

## Licenza

Gratis per uso personale; uso in aziende o per lavoro, modifica,
redistribuzione e uso commerciale vietati senza accordo scritto con l'autore:
vedi [`LICENZA.md`](LICENZA.md). Il
componente scrcpy sul telefono ha la sua licenza Apache 2.0.

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
  impronta sha256 in `telefono/aiuto/costruisci.sh`; serve anche `javac`
  (`sudo apt install default-jdk-headless`). Poi `sh telefono/aiuto/costruisci.sh`.
- `target/`: la crea `cargo build`.
- AppImage: contenitore `podman build -t phonestra-appimage costruzione`, poi il
  comando in cima a `costruzione/raccogli.sh`.
- `~/.config/Phonestra` (chiave ADB e telefoni associati): è personale e non va
  salvata; su un PC nuovo si riassocia il telefono con «Aggiungi telefono»
  (codice a 6 cifre del Debug wireless, `SPECIFICHE.md` §5.2).

Per ripetere le prove manuali servono inoltre `python3` (per
`prove/mdns-cerca-telefono.py`) e, solo per le diagnosi, il pacchetto `adb`.

Se il canvas dei mockup non fosse più raggiungibile, `mockup/README.md` spiega
come ricrearlo dai file del repository.
