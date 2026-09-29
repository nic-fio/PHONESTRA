#!/usr/bin/env python3
"""Disegna i diagrammi del manuale tecnico (docs/manuale-tecnico.html).

Il manuale è un file unico: deve funzionare anche scaricato da solo, senza la
cartella docs/assets. Per questo i diagrammi non si disegnano nel browser di chi
legge ma qui, una volta sola, con Mermaid (docs/assets/vendor/mermaid.min.js)
dentro Google Chrome senza finestra; il disegno (SVG) si scrive nella pagina.

Ogni diagramma nel manuale è fatto così:

    <pre class="mermaid-sorgente" hidden>flowchart LR ...</pre>
    <div class="mermaid">... SVG disegnato da questo script ...</div>

Si modifica solo la sorgente, poi si lancia: python3 docs/disegna-diagrammi.py
"""

import html
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

RADICE = Path(__file__).resolve().parent.parent
MANUALE = RADICE / "docs" / "manuale-tecnico.html"
MERMAID = RADICE / "docs" / "assets" / "vendor" / "mermaid.min.js"

DIAGRAMMA = re.compile(
    r'(<pre class="mermaid-sorgente" hidden>)(.*?)(</pre>\s*<div class="mermaid">)(.*?)(</div><!-- /mermaid -->)',
    re.S,
)

# Gli stessi colori del manuale (variabili di docs.css).
CONFIGURAZIONE = {
    "startOnLoad": False,
    "securityLevel": "strict",
    "theme": "base",
    "fontFamily": "system-ui, -apple-system, Segoe UI, Roboto, sans-serif",
    "themeVariables": {
        "background": "#ffffff", "primaryColor": "#eef5ff", "primaryTextColor": "#1b2430",
        "primaryBorderColor": "#14427c", "lineColor": "#4a6076", "secondaryColor": "#fff0e0",
        "tertiaryColor": "#f6f9ff", "noteBkgColor": "#fff5ea", "noteTextColor": "#1b2430",
        "actorBkg": "#eef5ff", "actorTextColor": "#1b2430", "actorBorder": "#14427c",
        "signalColor": "#1b2430", "signalTextColor": "#1b2430", "clusterBkg": "#f8fbff",
        "clusterBorder": "#d5e3f7", "edgeLabelBackground": "#ffffff", "nodeTextColor": "#1b2430",
    },
}

PAGINA = """<!DOCTYPE html>
<html><head><meta charset="utf-8"></head><body>
<textarea id="uscita"></textarea>
<script src="{mermaid}"></script>
<script>
(async function () {{
  var sorgenti = {sorgenti};
  var disegni = [];
  mermaid.initialize({configurazione});
  for (var i = 0; i < sorgenti.length; i++) {{
    try {{
      var r = await mermaid.render("diagramma-" + (i + 1), sorgenti[i]);
      disegni.push(r.svg);
    }} catch (e) {{
      disegni.push("ERRORE: " + e);
    }}
  }}
  document.getElementById("uscita").textContent = JSON.stringify(disegni);
  document.body.setAttribute("data-fatto", "1");
}})();
</script>
</body></html>
"""


def chrome():
    for nome in ("google-chrome", "chromium", "chromium-browser"):
        if shutil.which(nome):
            return nome
    sys.exit("Serve Google Chrome o Chromium per disegnare i diagrammi.")


def main():
    testo = MANUALE.read_text(encoding="utf-8")
    trovati = list(DIAGRAMMA.finditer(testo))
    if not trovati:
        sys.exit("Nessun diagramma trovato nel manuale.")
    sorgenti = [html.unescape(m.group(2)).strip() for m in trovati]

    with tempfile.TemporaryDirectory() as cartella:
        pagina = Path(cartella) / "disegna.html"
        pagina.write_text(PAGINA.format(
            mermaid=MERMAID.as_uri(),
            sorgenti=json.dumps(sorgenti),
            configurazione=json.dumps(CONFIGURAZIONE),
        ), encoding="utf-8")
        dom = subprocess.run(
            [chrome(), "--headless=new", "--disable-gpu", "--allow-file-access-from-files",
             f"--user-data-dir={cartella}/profilo", "--virtual-time-budget=20000",
             "--dump-dom", pagina.as_uri()],
            capture_output=True, text=True, check=True,
        ).stdout

    m = re.search(r'<textarea id="uscita">(.*?)</textarea>', dom, re.S)
    if not m or 'data-fatto="1"' not in dom:
        sys.exit("Chrome non ha finito di disegnare i diagrammi.")
    disegni = json.loads(html.unescape(m.group(1)))
    errori = [f"diagramma {i + 1}: {d}" for i, d in enumerate(disegni) if d.startswith("ERRORE")]
    if errori:
        sys.exit("\n".join(errori))

    pezzi, ultimo = [], 0
    for trovato, svg in zip(trovati, disegni):
        pezzi.append(testo[ultimo:trovato.start(4)])
        pezzi.append(svg)
        ultimo = trovato.end(4)
    pezzi.append(testo[ultimo:])
    MANUALE.write_text("".join(pezzi), encoding="utf-8")
    print(f"Disegnati {len(disegni)} diagrammi.")


if __name__ == "__main__":
    main()
