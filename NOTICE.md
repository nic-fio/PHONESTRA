# Copyright e componenti di terzi

Phonestra
Copyright © 2026 Nicola Fiorillo. Tutti i diritti riservati.

Phonestra è **freeware**, con la **Phonestra Freeware Licence** (in inglese:
[`LICENSE.md`](LICENSE.md)), dalla prima versione dopo la 1.0.0-rc.8.

| Si può, gratis | Non si può senza una licenza scritta |
|---|---|
| Usarlo per qualsiasi cosa, a casa o al lavoro, anche come strumento in un lavoro pagato | **Venderlo** o chiedere un compenso per averlo |
| Copiarlo e darlo a chiunque, intatto, con la licenza | **Includerlo** in un prodotto o servizio venduto o offerto da un'impresa |
| Eseguire l'AppImage come permette: renderla eseguibile, spostarla, estrarla, metterla nel menu | **Modificarlo**, o decompilarlo oltre quanto consente la legge (e la sezione 6: diritti delle librerie LGPL) |
| Metterlo in una raccolta di programmi data gratis, intatto e con la licenza | Togliere le note di copyright e di licenza, o usare il nome per far credere a un'approvazione |

**Per includere Phonestra in un prodotto, o comprare il progetto:**
**phonestra@nicfio.it**.

Le versioni fino alla 1.0.0-rc.8 compresa sono uscite con la loro licenza (uso
personale gratuito, `LICENSE-1.0.0-rc.8-and-earlier.md`), che le loro copie
conservano.

Tutto il codice del repository è di Phonestra, compreso il componente che gira
sul telefono (`android/`, `android/phonestra-helper.jar`): lì non ci sono
componenti di terzi.

## Componenti di terzi

Questi componenti mantengono le proprie licenze: quanto scritto sopra vale per
Phonestra, non per loro. Nessuno è copiato nel repository: le crate Rust le
scarica `cargo` (versioni fissate in `Cargo.lock`), le librerie dell'AppImage le
raccoglie `packaging/collect.sh` dal contenitore di costruzione.

### Compilati dentro l'eseguibile `phonestra`

libusb non è più compilata dentro `phonestra`: dalla versione dopo la
1.0.0-rc.8 è quella del sistema, collegata **dinamicamente** (vedi sotto,
librerie dell'AppImage). `adb_client` chiede `rusb` con la funzione `vendored`,
che la incorporerebbe staticamente; Phonestra usa una copia di `rusb` 0.9.4
senza quell'effetto (`vendor/rusb/`, `vendor/README.md`). Fino alla 1.0.0-rc.8
libusb 1.0.27 (LGPL-2.1-or-later) era collegata staticamente.

Le 142 crate Rust (dipendenze normali per Linux x86-64, senza le macro
procedurali, che servono solo a compilare; elenco ricavato con
`cargo tree -e normal,no-proc-macro`). Il testo di ogni licenza è nella crate,
su crates.io; nell'AppImage stanno tutti in
`usr/share/doc/phonestra/third-party/rust-crates.txt`, generato a ogni
costruzione da `packaging/rust-licenses.py` con lo stesso elenco.

| Crate | Versione | Licenza |
|---|---|---|
| `adb_client` | 3.2.3 | MIT |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 |
| `async-channel` | 2.5.0 | Apache-2.0 OR MIT |
| `atomic_refcell` | 0.1.14 | Apache-2.0 OR MIT |
| `base64ct` | 1.8.3 | Apache-2.0 OR MIT |
| `base64` | 0.22.1 | MIT OR Apache-2.0 |
| `base64` | 0.23.1 | MIT OR Apache-2.0 |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 |
| `byteorder` | 1.5.0 | Unlicense OR MIT |
| `bytes` | 1.12.1 | MIT |
| `cairo-rs` | 0.22.9 | MIT |
| `cairo-sys-rs` | 0.22.9 | MIT |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 |
| `chacha20` | 0.10.2 | MIT OR Apache-2.0 |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT |
| `const-oid` | 0.9.6 | Apache-2.0 OR MIT |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 |
| `cpufeatures` | 0.3.1 | MIT OR Apache-2.0 |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 |
| `curve25519-dalek` | 4.1.3 | BSD-3-Clause |
| `deranged` | 0.5.8 | MIT OR Apache-2.0 |
| `der` | 0.7.10 | Apache-2.0 OR MIT |
| `digest` | 0.10.7 | MIT OR Apache-2.0 |
| `either` | 1.18.0 | MIT OR Apache-2.0 |
| `equivalent` | 1.0.2 | Apache-2.0 OR MIT |
| `errno` | 0.3.14 | MIT OR Apache-2.0 |
| `event-listener-strategy` | 0.5.4 | Apache-2.0 OR MIT |
| `event-listener` | 5.4.2 | Apache-2.0 OR MIT |
| `field-offset` | 0.3.6 | MIT OR Apache-2.0 |
| `futures-channel` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-executor` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-io` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 |
| `gdk4-sys` | 0.11.5 | MIT |
| `gdk4` | 0.11.5 | MIT |
| `gdk-pixbuf-sys` | 0.22.9 | MIT |
| `gdk-pixbuf` | 0.22.0 | MIT |
| `generic-array` | 0.14.7 | MIT |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 |
| `gio-sys` | 0.22.9 | MIT |
| `gio` | 0.22.10 | MIT |
| `glib-sys` | 0.22.9 | MIT |
| `glib` | 0.22.10 | MIT |
| `gobject-sys` | 0.22.9 | MIT |
| `graphene-rs` | 0.22.8 | MIT |
| `graphene-sys` | 0.22.9 | MIT |
| `gsk4-sys` | 0.11.5 | MIT |
| `gsk4` | 0.11.5 | MIT |
| `gstreamer-app-sys` | 0.25.4 | MIT |
| `gstreamer-app` | 0.25.2 | MIT OR Apache-2.0 |
| `gstreamer-base-sys` | 0.25.4 | MIT |
| `gstreamer-base` | 0.25.4 | MIT OR Apache-2.0 |
| `gstreamer-sys` | 0.25.4 | MIT |
| `gstreamer` | 0.25.4 | MIT OR Apache-2.0 |
| `gtk4-sys` | 0.11.5 | MIT |
| `gtk4` | 0.11.5 | MIT |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT |
| `itertools` | 0.15.0 | MIT OR Apache-2.0 |
| `kstring` | 2.0.5 | MIT OR Apache-2.0 |
| `lazy_static` | 1.5.0 | MIT OR Apache-2.0 |
| `libadwaita-sys` | 0.9.2 | MIT |
| `libadwaita` | 0.9.2 | MIT |
| `libc` | 0.2.189 | MIT OR Apache-2.0 |
| `libm` | 0.2.16 | MIT |
| `libusb1-sys` | 0.7.0 | MIT |
| `log` | 0.4.34 | MIT OR Apache-2.0 |
| `memchr` | 2.8.3 | Unlicense OR MIT |
| `memoffset` | 0.9.1 | MIT |
| `mio` | 1.2.3 | MIT |
| `muldiv` | 1.0.1 | MIT |
| `num-bigint-dig` | 0.8.6 | MIT OR Apache-2.0 |
| `num-conv` | 0.2.2 | MIT OR Apache-2.0 |
| `num_enum` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| `num-integer` | 0.1.47 | MIT OR Apache-2.0 |
| `num-iter` | 0.1.46 | MIT OR Apache-2.0 |
| `num-rational` | 0.4.2 | MIT OR Apache-2.0 |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 |
| `option-operations` | 0.6.1 | MIT OR Apache-2.0 |
| `pango-sys` | 0.22.9 | MIT |
| `pango` | 0.22.9 | MIT |
| `parking` | 2.2.1 | Apache-2.0 OR MIT |
| `pem-rfc7468` | 0.7.0 | Apache-2.0 OR MIT |
| `pem` | 4.0.0 | MIT |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT |
| `pkcs1` | 0.7.5 | Apache-2.0 OR MIT |
| `pkcs8` | 0.10.2 | Apache-2.0 OR MIT |
| `powerfmt` | 0.2.0 | MIT OR Apache-2.0 |
| `ppv-lite86` | 0.2.21 | MIT OR Apache-2.0 |
| `quick-protobuf` | 0.8.1 | MIT |
| `rand_chacha` | 0.3.1 | MIT OR Apache-2.0 |
| `rand_core` | 0.10.1 | MIT OR Apache-2.0 |
| `rand_core` | 0.6.4 | MIT OR Apache-2.0 |
| `rand` | 0.10.3 | MIT OR Apache-2.0 |
| `rand` | 0.8.8 | MIT OR Apache-2.0 |
| `rcgen` | 0.14.10 | MIT OR Apache-2.0 |
| `regex-automata` | 0.4.18 | MIT OR Apache-2.0 |
| `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 |
| `regex` | 1.13.1 | MIT OR Apache-2.0 |
| `ring` | 0.17.14 | Apache-2.0 AND ISC |
| `rsa` | 0.9.10 | MIT OR Apache-2.0 |
| `rusb` | 0.9.4 | MIT |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 |
| `rustls` | 0.23.45 | Apache-2.0 OR ISC OR MIT |
| `rustls-webpki` | 0.103.15 | ISC |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 |
| `serde` | 1.0.229 | MIT OR Apache-2.0 |
| `sha1` | 0.10.7 | MIT OR Apache-2.0 |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 |
| `signature` | 2.2.0 | Apache-2.0 OR MIT |
| `slab` | 0.4.12 | MIT |
| `smallvec` | 1.16.2 | MIT OR Apache-2.0 |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 |
| `spin` | 0.9.9 | MIT |
| `spki` | 0.7.3 | Apache-2.0 OR MIT |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 |
| `subtle` | 2.6.1 | BSD-3-Clause |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 |
| `time-core` | 0.1.9 | MIT OR Apache-2.0 |
| `time` | 0.3.55 | MIT OR Apache-2.0 |
| `tokio-rustls` | 0.26.5 | MIT OR Apache-2.0 |
| `tokio` | 1.53.1 | MIT |
| `toml_datetime` | 0.6.11 | MIT OR Apache-2.0 |
| `toml_edit` | 0.22.27 | MIT OR Apache-2.0 |
| `toml` | 0.8.23 | MIT OR Apache-2.0 |
| `toml_write` | 0.1.2 | MIT OR Apache-2.0 |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 |
| `untrusted` | 0.9.0 | ISC |
| `winnow` | 0.7.15 | MIT |
| `yasna` | 0.6.0 | MIT OR Apache-2.0 |
| `zerocopy` | 0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT |
| `zeroize` | 1.9.0 | Apache-2.0 OR MIT |

### Inclusi nell'AppImage (Release su GitHub)

L'AppImage porta con sé le librerie che l'eseguibile usa e che non si possono
dare per presenti su ogni distribuzione (elenco preciso: quello che
`packaging/collect.sh` copia con `ldd`; restano fuori glibc, driver grafici,
X11, fontconfig, freetype, PipeWire). Le principali:

| Componente | Versione | Licenza |
|---|---|---|
| GTK | 4.14.5 (compilata nel contenitore) | LGPL-2.1-or-later |
| libadwaita | 1.5.4 (compilata nel contenitore) | LGPL-2.1-or-later |
| GLib, GIO | 2.80.5 (compilata nel contenitore) | LGPL-2.1-or-later |
| graphene | 1.10.8 (compilata nel contenitore) | MIT |
| libtiff | 4.6.0 (compilata nel contenitore senza JBIG, dalla 1.3.0) | libtiff (BSD-like) |
| Wayland (libwayland-*) | 1.22.0 (compilata nel contenitore, «di riserva») | MIT |
| gst-plugin-gtk4 (`gtk4paintablesink`, da gst-plugins-rs) | 0.13.5 | MPL-2.0 |
| libusb | quella di Ubuntu 22.04 | LGPL-2.1-or-later |
| GStreamer e i plugin base, good, bad (con `va`, `nvcodec`, `openh264`), vaapi | quelle di Ubuntu 22.04 | LGPL-2.1-or-later |
| OpenH264 (libopenh264, decodifica H.264 in software, usata dal plugin `openh264`) | 2.2.0 (Ubuntu 22.04) | BSD-2-Clause |
| libopus (decodifica dell'audio, usata dal plugin `opus`) | 1.3.1 (Ubuntu 22.04) | BSD-3-Clause |
| Pango, cairo, HarfBuzz, FriBidi, gdk-pixbuf, librsvg, libepoxy, libxkbcommon, dconf e le loro dipendenze | quelle di Ubuntu 22.04 | LGPL-2.1-or-later, MIT o simili (vedi `/usr/share/doc/<pacchetto>/copyright` in Ubuntu 22.04) |
| Tema di icone Adwaita | quello di Ubuntu 22.04 | LGPL-3.0 / CC-BY-SA-3.0 |
| Runtime di AppImage (da appimagetool) | continuous | MIT |

Le librerie LGPL dell'AppImage, libusb compresa, sono collegate dinamicamente
e stanno come file separati (`usr/lib/`): chi vuole può sostituirle con altre
versioni compatibili (la sezione 6 di `LICENSE.md` lo permette).

**Niente FFmpeg dalla versione 1.2.0.** Fino alla 1.1.1 l'AppImage conteneva
FFmpeg 4.4.2 di Ubuntu 22.04, compilato con `--enable-gpl` (quindi
GPL-2.0-or-later, con x264 e x265), caricato da GStreamer col plugin `libav`
per decodificare l'audio AAC e il video H.264 senza scheda video. Dalla 1.2.0
il telefono manda l'audio in Opus (decodificato da libopus) e il video H.264 si
decodifica con la scheda video (VA-API, NVDEC) o con OpenH264: nell'AppImage
non ci sono più FFmpeg, x264 e x265.

**Niente libjbig e liblzo2 dalla versione 1.3.0.** Fino alla 1.2.0 GTK portava
con sé due librerie GPL-2.0-or-later di Ubuntu 22.04: **libjbig** (usata dalla
libtiff di Ubuntu) e **liblzo2** (usata da `cairo-script-interpreter`). Dalla
1.3.0 libtiff è compilata nel contenitore senza JBIG e GTK è compilata senza
l'interprete di script di cairo (facoltativo: legge solo i file di debug del
disegno), quindi nell'AppImage non c'è nessuna libreria sotto la sola GPL
(verificato l'8 ott 2026 sui file di copyright: dove compare la GPL riguarda i
file di Debian, gli esempi o gli strumenti a riga di comando dei pacchetti,
non le librerie incluse). L'unico file GPL-2.0-or-later rimasto è un file di
dati, `usr/share/icons/hicolor/index.theme` del tema di icone hicolor. Le librerie di NVIDIA (CUDA, NVDEC) non sono
incluse: le carica il plugin `nvcodec` dal driver del sistema, se c'è.

### Testi delle licenze e sorgenti

Dentro l'AppImage, in `usr/share/doc/phonestra/`: `LICENSE.md`, questo
`NOTICE.md` e la cartella `third-party/`, che `packaging/collect.sh` riempie a
ogni costruzione:

- `ubuntu/<pacchetto>/copyright`: il file di copyright e licenza di ogni
  pacchetto Ubuntu da cui viene almeno un file copiato nell'AppImage, e in
  `common-licenses/` i testi completi (LGPL, GPL, Apache…) a cui quei file
  rimandano;
- `built/<componente>/`: le licenze delle librerie compilate nel contenitore
  (GTK, libadwaita, GLib, graphene, libtiff, Wayland, wayland-protocols) e di
  gst-plugin-gtk4, con le licenze delle crate compilate dentro di lui
  (`rust-crates.txt`);
- `rust-crates.txt`: le licenze delle crate compilate dentro `phonestra`;
- `appimage-runtime/LICENSE`: il runtime di AppImage e le librerie che contiene;
- `README.txt`: l'indice, con la versione di ogni pacchetto Ubuntu e i file che
  ne vengono.

I sorgenti delle versioni incluse: per i pacchetti Ubuntu, quelli di Ubuntu
22.04 (`apt-get source <pacchetto>=<versione>`, o
`https://launchpad.net/ubuntu/+source/<pacchetto>`); per le librerie compilate
nel contenitore, i tarball e il ramo indicati in `packaging/Containerfile`.
