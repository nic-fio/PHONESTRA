#!/usr/bin/env python3
# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
"""Testi delle licenze delle crate Rust compilate dentro un programma.

    rust-licenses.py [--package NOME] [--manifest-path Cargo.toml] [-- OPZIONI DI CARGO…] > FILE

Parte dal pacchetto indicato (quello principale se manca) e prende, con
`cargo tree` (versioni di Cargo.lock), solo le dipendenze normali per Linux
x86-64: niente macro procedurali, dipendenze di compilazione (build.rs) o di
prova, che nel programma finito non entrano (lo stesso elenco di NOTICE.md).
Per ogni crate scrive nome, versione, licenza dichiarata e il testo dei file di
licenza che la crate porta con sé (LICENSE*, LICENCE*, COPYING*, NOTICE*,
UNLICENSE*, COPYRIGHT*, anche nelle sottocartelle dirette); un testo
identico a uno già scritto si scrive una volta sola. Nessuno strumento da
installare: bastano cargo e Python.
Per una crate MIT senza file di licenza scrive il testo MIT standard con gli
autori del suo Cargo.toml; esce con 1 se una crate di altra licenza non ne ha.
"""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys

PIATTAFORMA = "x86_64-unknown-linux-gnu"
# Testo della licenza MIT (https://spdx.org/licenses/MIT.html), per le crate
# MIT che non ne portano una copia: il copyright è quello degli autori
# dichiarati nel loro Cargo.toml.
MIT = """Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""
PREFISSI = ("license", "licence", "copying", "notice", "unlicense", "copyright")


def licenza(p):
    nome = p.name.lower()
    return p.is_file() and nome.startswith(PREFISSI) and not nome.endswith((".rs", ".toml"))


def file_di_licenza(cartella):
    """I file di licenza della crate, in ordine: nella radice, nella cartella
    LICENSES/ e nelle sottocartelle dirette, dove stanno i sorgenti C che la
    crate porta con sé (es. libusb/COPYING in libusb1-sys)."""
    trovati = []
    for p in sorted(cartella.iterdir()):
        if licenza(p):
            trovati.append(p)
        elif p.is_dir() and p.name.lower() == "licenses":
            trovati += sorted(q for q in p.iterdir() if q.is_file())
        elif p.is_dir() and p.name not in ("tests", "benches", "examples", "src", "target"):
            trovati += sorted(q for q in p.iterdir() if licenza(q))
    return trovati


def main():
    arg = argparse.ArgumentParser()
    arg.add_argument("--package")
    arg.add_argument("--manifest-path")
    arg.add_argument("cargo", nargs="*", help="altre opzioni per cargo tree (es. --features)")
    a = arg.parse_args()

    # L'elenco delle crate lo dà cargo tree, che applica le feature come la
    # compilazione vera (cargo metadata elenca anche dipendenze facoltative
    # che non si compilano); cargo metadata, con tutte le feature perché
    # nessuna crate dell'elenco manchi, dà dove stanno i loro sorgenti.
    comune = ["--manifest-path", a.manifest_path] if a.manifest_path else []
    albero = ["cargo", "tree", "-e", "normal,no-proc-macro", "--target", PIATTAFORMA,
              "--prefix", "none", "-f", "{p}"] + comune + (["-p", a.package] if a.package else []) + a.cargo
    righe = subprocess.run(albero, check=True, capture_output=True, text=True).stdout.splitlines()
    nomi = {tuple(r.split()[:2]) for r in righe if r.strip()}
    meta = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1", "--all-features"] + comune,
                                     check=True, capture_output=True, text=True).stdout)
    radice = righe[0].split()[0]
    pacchetti = {(p["name"], "v" + p["version"]): p for p in meta["packages"]}
    crate = sorted((pacchetti[n] for n in nomi if n[0] != radice), key=lambda p: (p["name"], p["version"]))

    nome_radice = radice
    out = [f"Crate Rust compilate dentro {nome_radice}: {len(crate)}",
           f"Generato da packaging/rust-licenses.py (cargo tree, Cargo.lock, {PIATTAFORMA});",
           "dipendenze normali, senza macro procedurali, build.rs e prove.",
           "Per ogni crate: licenza dichiarata e testo dei file di licenza che porta con sé.", ""]
    scritti, senza = {}, []
    for p in crate:
        cartella = pathlib.Path(p["manifest_path"]).parent
        out += ["=" * 78, f"{p['name']} {p['version']}",
                f"Licenza: {p.get('license') or '(file ' + str(p.get('license_file')) + ')'}"]
        if p.get("repository"):
            out.append(f"Sorgenti: {p['repository']}")
        file = file_di_licenza(cartella)
        if not file and p.get("license") == "MIT":
            autori = ", ".join(p.get("authors") or []) or "gli autori della crate"
            out += ["", "--- (nessun file di licenza nella crate: testo MIT standard) ---",
                    f"Copyright (c) {autori}", "", MIT]
        elif not file:
            senza.append(f"{p['name']} {p['version']}")
            out.append("(la crate non porta nessun file di licenza)")
        for f in file:
            testo = f.read_text(encoding="utf-8", errors="replace").rstrip() + "\n"
            impronta = hashlib.sha256(testo.encode()).hexdigest()
            out += ["", f"--- {f.relative_to(cartella)} ---"]
            if impronta in scritti:
                out.append(f"(testo identico a quello di {scritti[impronta]}, scritto sopra)")
            else:
                scritti[impronta] = f"{f.relative_to(cartella)} di {p['name']} {p['version']}"
                out.append(testo)
        out.append("")
    sys.stdout.write("\n".join(out))
    if senza:
        print("crate senza file di licenza: " + ", ".join(senza), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
