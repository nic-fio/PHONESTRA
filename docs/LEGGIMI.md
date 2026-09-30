# Documentazione di Phonestra

| File | Per chi |
|---|---|
| `Phonestra_Manuale_Utente.html` | Chi usa Phonestra: installazione, primo collegamento, uso di tutti i giorni, problemi. |
| `Phonestra_Manuale_Tecnico.html` | Chi sviluppa: architettura, client ADB, componente sul telefono, video, audio, input, interfaccia, AppImage, prove. |

Sono file unici, senza script: stile, logo e schemi SVG sono dentro la pagina.
Si possono scaricare da soli e aprire nel browser, anche senza internet. (Su
GitHub, cliccandoci sopra, si vede il codice sorgente: GitHub non mostra i file
`.html` come pagine.) Nomi, stile e struttura sono quelli dei manuali di
IR_Service, come in AMS.

I manuali sono **generati**: non si modificano a mano.

| Sorgente | Cosa |
|---|---|
| `sorgenti/build.py` | Il generatore: funzioni per testo, tabelle, avvisi e figure SVG; mappa dei file e numeri contati dai sorgenti; controlli. |
| `sorgenti/stile.css` | Lo stile, lo stesso di IR_Service e AMS. |
| `sorgenti/tecnico/chNN_*.py`, `sorgenti/utente/chNN_*.py` | Un file per capitolo: `CHAPTER = (titolo, [(sezione, html), …])`. |

```
python3 docs/sorgenti/build.py              # rigenera i due manuali
python3 docs/sorgenti/build.py --controlla  # li controlla (lo fa cargo test)
```

Serve `python3-pygments`. Il perché delle decisioni non sta qui ma in `memoria/`.
