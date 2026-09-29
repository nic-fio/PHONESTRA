# Documentazione di Phonestra

`manuale-tecnico.html` è un file unico: stile, script, logo e diagrammi sono
dentro la pagina. Si può scaricare da solo e aprire nel browser, anche senza
internet. (Su GitHub, cliccandoci sopra, si vede il codice sorgente: GitHub non
mostra i file `.html` come pagine.)

## File

| File | Cosa |
|---|---|
| `manuale-tecnico.html` | Il manuale tecnico: architettura, client ADB, componente sul telefono, video, audio, input, interfaccia, AppImage, prove, convenzioni. Stile e script (indice laterale, ricerca, indice analitico) sono ripresi dal manuale di NESH e tradotti. |
| `aggiorna-numeri.py` | Riscrive le righe della mappa dei file e della tabella dei numeri. `cargo test` (`tests/manuale.rs`) fallisce quando non corrispondono più ai sorgenti. |
| `disegna-diagrammi.py` | Disegna i diagrammi: ognuno ha la sorgente Mermaid in un `<pre class="mermaid-sorgente" hidden>` e, subito sotto, il disegno SVG. Si modifica la sorgente e si lancia lo script (serve Google Chrome o Chromium). |
| `assets/vendor/mermaid.min.js` | Mermaid, usato solo da `disegna-diagrammi.py` (licenza MIT, `assets/vendor/mermaid.LICENSE`). La pagina non lo carica. |

Il perché delle decisioni non sta qui ma in `memoria/`.
