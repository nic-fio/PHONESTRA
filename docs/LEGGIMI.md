# Documentazione di Phonestra

GitHub mostra i file `.html` come codice sorgente, non come pagine. Per leggere
il manuale: clona il repository e apri `docs/manuale-tecnico.html` nel
browser. Tutto quello che serve alla pagina (stile, script, diagrammi) è in
`docs/assets`, quindi funziona anche senza internet.

## File

| File | Cosa |
|---|---|
| `manuale-tecnico.html` | Il manuale tecnico: architettura, client ADB, componente sul telefono, video, audio, input, interfaccia, AppImage, prove, convenzioni. |
| `aggiorna-numeri.py` | Riscrive le righe della mappa dei file e della tabella dei numeri. `cargo test` (`tests/manuale.rs`) fallisce quando non corrispondono più ai sorgenti. |
| `assets/docs.css`, `assets/docs.js` | Stile, indice laterale, ricerca, indice analitico: ripresi dal manuale di NESH e tradotti. |
| `assets/vendor/mermaid.min.js` | Mermaid, per i diagrammi (licenza MIT, `assets/vendor/mermaid.LICENSE`). |

Il perché delle decisioni non sta qui ma in `memoria/`.
