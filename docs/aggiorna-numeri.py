#!/usr/bin/env python3
"""Riscrive i numeri del manuale tecnico (docs/manuale-tecnico.html): la mappa
dei file con le righe di ogni sorgente, la tabella «Il progetto in numeri» e la
dimensione nella testata. Le descrizioni della mappa restano quelle scritte a
mano; un file nuovo riceve «DA DESCRIVERE» (e `cargo test` fallisce finché
qualcuno non la scrive), un file sparito perde la sua riga.

    python3 docs/aggiorna-numeri.py

`tests/manuale.rs` controlla che i numeri corrispondano ai sorgenti.
"""
import html
import pathlib
import re

RADICE = pathlib.Path(__file__).resolve().parent.parent
MANUALE = RADICE / "docs" / "manuale-tecnico.html"
DA_DESCRIVERE = "DA DESCRIVERE"

# File della mappa: stessi criteri di tests/manuale.rs.
def sorgenti():
    file = []
    file += sorted((RADICE / "src").rglob("*.rs"))
    file += sorted((RADICE / "telefono" / "aiuto" / "src").rglob("*.java"))
    file += sorted(p for p in (RADICE / "costruzione").iterdir() if p.is_file() and p.suffix != ".svg")
    file += [RADICE / "telefono" / "aiuto" / "costruisci.sh", RADICE / "docs" / "aggiorna-numeri.py",
             RADICE / "docs" / "disegna-diagrammi.py"]
    file += sorted((RADICE / "tests").glob("*.rs"))
    return [p.relative_to(RADICE).as_posix() for p in file]

def righe(percorso):
    return (RADICE / percorso).read_bytes().count(b"\n")

# Parti del progetto per la tabella dei numeri: la prima regola che corrisponde vince.
PARTI = [
    ("Prove e misure (PC)", "src/bin/prova.rs, src/bin/prova/, prova_input.rs, misura_audio.rs, video_nostro/prova.rs, adb/misura.rs, adb/prove.rs, tests/",
     "lo strumento phonestra-prova, le misure dello studio, i test d'integrazione",
     lambda p: p.startswith(("src/bin/prova", "tests/")) or p in (
         "src/prova_input.rs", "src/misura_audio.rs", "src/video_nostro/prova.rs", "src/adb/misura.rs", "src/adb/prove.rs")),
    ("Prove e misure (telefono)", "VideoProva.java, InputProva.java, Codificatori.java",
     "strumenti di misura e comandi di prova del componente",
     lambda p: p.endswith(("/VideoProva.java", "/InputProva.java", "/Codificatori.java"))),
    ("Interfaccia", "cassetto.rs, finestra.rs, prepara.rs, procedura.rs, avvisi.rs, foto.rs, bin/phonestra.rs",
     "drawer, finestre delle app, primo collegamento, avvisi",
     lambda p: p in ("src/cassetto.rs", "src/finestra.rs", "src/prepara.rs", "src/procedura.rs", "src/avvisi.rs",
                     "src/foto.rs", "src/bin/phonestra.rs")),
    ("Client ADB", "src/adb/", "messaggi, TLS, canali, shell,v2, sync:, associazione",
     lambda p: p.startswith("src/adb/")),
    ("Componente, lato PC", "componente.rs, audio_nostro.rs, video_nostro/, input_nostro.rs, appunti.rs",
     "servizio, smistamento, audio, video, input, appunti",
     lambda p: p in ("src/componente.rs", "src/audio_nostro.rs", "src/input_nostro.rs", "src/appunti.rs")
     or p.startswith("src/video_nostro/")),
    ("Collegamento e dati", "collegamento.rs, rete.rs, configurazione.rs, telefono.rs, usb.rs, notifiche.rs, azioni.rs, app.rs, lib.rs",
     "collegamento, mDNS, configurazione, cavo, notifiche, azioni, elenco delle app",
     lambda p: p.startswith("src/")),
    ("Componente sul telefono", "telefono/aiuto/src/phonestra/", "servizio, custode, audio, video, input, appunti, pannello",
     lambda p: p.startswith("telefono/aiuto/src/")),
    ("Costruzione e strumenti", "costruzione/, costruisci.sh, docs/*.py", "contenitore, AppImage, jar, numeri e diagrammi del manuale",
     lambda p: True),
]

# Ordine in cui le parti compaiono nel manuale.
ORDINE = ["Interfaccia", "Collegamento e dati", "Client ADB", "Componente, lato PC", "Componente sul telefono",
          "Prove e misure (PC)", "Prove e misure (telefono)", "Costruzione e strumenti"]

def parte(percorso):
    return next(i for i, (_, _, _, regola) in enumerate(PARTI) if regola(percorso))

def migliaia(n):
    return f"{n:,}".replace(",", ".")

def main():
    testo = MANUALE.read_text(encoding="utf-8")
    inizio, fine = "<!-- mappa:inizio -->", "<!-- mappa:fine -->"
    a, b = testo.index(inizio) + len(inizio), testo.index(fine)
    descrizioni = {
        html.unescape(m.group(1)): m.group(2)
        for m in re.finditer(r'<tr data-file="([^"]+)"[^>]*><td><code>[^<]*</code></td><td>\d+</td><td>(.*?)</td></tr>', testo[a:b])
    }
    file = sorgenti()
    conteggi = {f: righe(f) for f in file}

    gruppi = {}
    for f in file:
        gruppi.setdefault(parte(f), []).append(f)
    mappa = ["", "<table>",
             "  <thead><tr><th>File</th><th>Righe</th><th>Contenuto</th></tr></thead>", "  <tbody>"]
    for i in sorted(gruppi, key=lambda i: ORDINE.index(PARTI[i][0])):
        mappa.append(f'    <tr class="gruppo"><td colspan="3"><b>{PARTI[i][0]}</b></td></tr>')
        for f in gruppi[i]:
            d = descrizioni.get(f, DA_DESCRIVERE)
            mappa.append(f'    <tr data-file="{html.escape(f)}" data-righe="{conteggi[f]}" data-index="{html.escape(f.rsplit("/", 1)[-1])}" data-kind="file">'
                         f'<td><code>{html.escape(f)}</code></td><td>{conteggi[f]}</td><td>{d}</td></tr>')
    mappa += ["  </tbody>", "</table>", ""]
    testo = testo[:a] + "\n".join(mappa) + testo[b:]

    inizio, fine = "<!-- numeri:inizio -->", "<!-- numeri:fine -->"
    a, b = testo.index(inizio) + len(inizio), testo.index(fine)
    tabella = ["", "<table>", "  <thead><tr><th>Parte</th><th>Dove</th><th>Righe</th><th>Contenuto</th></tr></thead>", "  <tbody>"]
    for i in sorted(range(len(PARTI)), key=lambda i: ORDINE.index(PARTI[i][0])):
        nome, dove, contenuto, _ = PARTI[i]
        n = sum(conteggi[f] for f in gruppi.get(i, []))
        tabella.append(f'    <tr data-parte="{i}" data-righe="{n}"><td>{nome}</td><td><code>{html.escape(dove)}</code></td><td>{migliaia(n)}</td><td>{contenuto}</td></tr>')
    totale = sum(conteggi.values())
    tabella.append(f'    <tr data-parte="totale" data-righe="{totale}"><td><b>Totale</b></td><td></td><td><b>{migliaia(totale)}</b></td><td></td></tr>')
    tabella += ["  </tbody>", "</table>", ""]
    testo = testo[:a] + "\n".join(tabella) + testo[b:]

    rust = sum(n for f, n in conteggi.items() if f.endswith(".rs"))
    java = sum(n for f, n in conteggi.items() if f.endswith(".java"))
    arrotonda = lambda n: migliaia(round(n, -2))
    inizio, fine = "<!-- numeri:dimensione -->", "<!-- /numeri:dimensione -->"
    a, b = testo.index(inizio) + len(inizio), testo.index(fine)
    testo = testo[:a] + f"~{arrotonda(rust)} righe di Rust, ~{arrotonda(java)} di Java" + testo[b:]

    MANUALE.write_text(testo, encoding="utf-8")
    mancanti = [f for f in file if f not in descrizioni]
    print(f"{len(file)} file, {migliaia(totale)} righe")
    for f in mancanti:
        print(f"  da descrivere nella mappa: {f}")

if __name__ == "__main__":
    main()
