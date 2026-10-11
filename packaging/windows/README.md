# Lo zip portatile per Windows

Come è nato `Phonestra-1.3.0-Windows-Portable.zip` (11 ottobre 2026), per
poterlo rifare. Il perché delle scelte: `NOTICE.md` («Inclusi nello zip per
Windows») e `notes/user-decisions.md`.

## Da dove parte

Una cartella preparata su Windows con MSYS2 UCRT64: `phonestra.exe`
compilato per `x86_64-pc-windows-gnu` (stesse crate di `Cargo.lock`), le DLL
che usa e i dati di GTK (`share/`, `etc/fonts/`, `lib/gstreamer-1.0/`,
`lib/gdk-pixbuf-2.0/`), più lo script `Avvia-Phonestra-Portable.cmd`, che
imposta `XDG_CONFIG_HOME` e `HOME` sulla cartella `Dati` accanto
(`src/configurazione.rs` legge una di queste due variabili, e Windows non
imposta `HOME`).

## I passi, su Linux

1. **Togliere** `adb.exe` e `AdbWinApi.DLL` (Phonestra non li usa), i `.dll.a`,
   i loader `libpixbufloader-heif.dll` e `io-wmf.dll` (con le loro righe in
   `loaders.cache`), `share/fonts/gsfonts/` e i file `69-urw-*` di fontconfig
   (AGPL), le icone `*djvu*` (GPL); poi `gtk4-update-icon-cache -f -t
   share/icons/hicolor`.
2. **libtiff senza JBIG**: libtiff della stessa versione di MSYS2 (4.7.2,
   SHA-256 del `.tar.gz` uguale a quello del PKGBUILD di MSYS2), compilata con
   [llvm-mingw](https://github.com/mstorsjo/llvm-mingw) UCRT, con intestazioni e
   librerie d'importazione dei pacchetti MSYS2 di zlib-ng-compat,
   libjpeg-turbo, libdeflate, xz, zstd, libwebp e lerc:

       ../tiff-4.7.2/configure --host=x86_64-w64-mingw32 CC=x86_64-w64-mingw32-clang \
         CPPFLAGS=-I<sysroot>/include LDFLAGS="-L<sysroot>/lib -Wl,--no-insert-timestamp" \
         --disable-static --enable-shared --disable-cxx --disable-jbig --enable-lerc \
         --enable-libdeflate --enable-webp --enable-zstd --enable-lzma \
         --disable-docs --disable-tests --disable-tools --disable-contrib PKG_CONFIG=false

   La nuova `libtiff-6.dll` esporta tutte le funzioni della vecchia; poi via
   `libjbig-0.dll`.
3. **Il sostituto dell'interprete di cairo**: `csi-stub.c` →
   `libcairo-script-interpreter-2.dll`
   (`x86_64-w64-mingw32-clang -O2 -shared -Wl,--no-insert-timestamp`); poi via
   `liblzo2-2.dll`. GTK 4.24 importa solo le quattro funzioni del file
   (`gsk/gskrendernodeparser.c`).
4. **Le DLL rimaste senza uso**: `orfane.py CARTELLA`, da ripetere finché non
   stampa nulla (la prima volta: x264, x265, i codificatori AV1, abseil,
   protobuf…).
5. **Nessun percorso personale**: i percorsi di `cargo` in `phonestra.exe`
   contengono il nome dell'utente di Windows; si sostituisce con una parola
   della stessa lunghezza (`C:\Users\builder\`). Meglio: compilare con
   `--remap-path-prefix`.
6. **Le licenze**, in `doc/`: `LICENSE*.md`, `NOTICE.md`; in `third-party/` le
   cartelle `share/licenses/<nome>` dei pacchetti MSYS2 che `mappa.py` trova
   (per i pacchetti di GStreamer, che non ne hanno, il `COPYING` della
   versione dal monorepo di GStreamer), `built/libtiff/LICENSE.md`,
   `built/cairo-script-interpreter-stub/csi-stub.c`,
   `llvm-mingw/LICENSE.TXT`, `rust-crates.txt` (`packaging/rust-licenses.py`
   con `PIATTAFORMA = "x86_64-pc-windows-gnu"`) e un `README.txt` con
   l'elenco dei pacchetti.
7. **Lo zip**: `zippa.py CARTELLA target/release-v<versione>/Phonestra-<versione>-Windows-Portable.zip`;
   `site/publish.sh` lo prende da lì.

Prova rapida senza Windows: Wine in un contenitore (Debian, `wine`,
`xvfb`, senza rete, `GSK_RENDERER=cairo`): la finestra si apre, i caratteri
sono storpiati perché lì mancano i font di Windows. La prova vera è su
Windows.
