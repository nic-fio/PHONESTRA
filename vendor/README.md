# Crate di terzi modificate

Copie di crate di crates.io che Phonestra usa al posto degli originali
(`[patch.crates-io]` in `Cargo.toml`). Restano dei loro autori, con le loro
licenze (file `LICENSE` in ogni cartella); le modifiche sono solo quelle
scritte qui.

| Cartella | Originale | Modifica | Perché |
|---|---|---|---|
| `rusb/` | rusb 0.9.4 (MIT) | la funzione `vendored` non attiva più `libusb1-sys/vendored`; in `src/lib.rs` spenti gli avvisi dei compilatori recenti; tolti `examples/` e `Cargo.lock` | `adb_client` chiede `rusb` con `vendored`, che compila libusb (LGPL-2.1) **dentro** `phonestra`. Con sorgenti chiusi la LGPL chiede che la libreria resti sostituibile: così libusb è quella del sistema (`libusb-1.0-0-dev` per compilare), collegata dinamicamente, e nell'AppImage è un file a parte. `packaging/collect.sh` si ferma se `phonestra` non chiede `libusb-1.0.so`. |

Per aggiornare `adb_client` o `rusb`: rifare la copia dalla nuova versione
(`~/.cargo/registry/src/*/rusb-<versione>`) e ripetere la modifica.
