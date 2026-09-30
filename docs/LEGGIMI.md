# Documentazione di Phonestra

`manuale-tecnico.html` è un file unico, senza script: stile, logo e schemi SVG
sono dentro la pagina. Si può scaricare da solo e aprire nel browser, anche
senza internet. (Su GitHub, cliccandoci sopra, si vede il codice sorgente:
GitHub non mostra i file `.html` come pagine.)

Il manuale è **generato**: non si modifica a mano.

## File

| File | Cosa |
|---|---|
| `manuale-tecnico.html` | Il manuale tecnico, generato. Stile e struttura sono quelli dei manuali di AMS (a loro volta sul modello del Manuale Tecnico di IR_Service). |
| `sorgenti/build.py` | Il generatore: funzioni per testo, tabelle, avvisi e figure SVG; mappa dei file e numeri contati dai sorgenti; controlli. |
| `sorgenti/stile.css` | Lo stile, lo stesso di AMS. |
| `sorgenti/capitoli/chNN_*.py` | Un file per capitolo: `CHAPTER = (titolo, [(sezione, html), …])`. |

```
python3 docs/sorgenti/build.py              # rigenera il manuale
python3 docs/sorgenti/build.py --controlla  # lo controlla (lo fa cargo test)
```

Serve `python3-pygments`. Il perché delle decisioni non sta qui ma in `memoria/`.
