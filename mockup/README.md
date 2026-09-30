# Mockup dell'interfaccia

Canvas pubblicato (privato, account dell'utente):
**https://claude.ai/artifact/5Zg31Zy9vSvDive8RZmYyc**

| Stile | File |
|---|---|
| 1 · Adwaita nativo (scelto per ora) | `canvas/Main.dc.html` (drawer), `canvas/d1-finestra.dc.html`, `canvas/d1-notifiche.dc.html`, `canvas/d1-procedura.dc.html` |
| 2 · Barra laterale | `canvas/d3-*.dc.html` |
| 3 · Ricerca prima di tutto | `canvas/d5-*.dc.html` |

I nomi `d3`/`d5` vengono dalla prima versione a 5 stili (2 e 4 erano gli stili
scuri, eliminati). `canvas/canvas.json` descrive disposizione e titoli.

## Formato

Ogni `.dc.html` è una schermata nel formato del tipo «Design» di Claude
(artboard). Non si aprono direttamente nel browser: hanno bisogno del runtime
del canvas (`support.js`).

## Icone

Le icone delle app sono i file in `icone/`. Nei `.dc.html` sono richiamate con
indirizzi `/_blob/<id>` che esistono **solo dentro il canvas pubblicato**.

## Ripristino se il canvas è perso

Chiedere a Claude Code, dalla cartella del repository:

> Crea un nuovo canvas Design «Phonestra – mockup interfaccia» con i file di
> `mockup/canvas/`; carica prima le icone di `mockup/icons/` come asset e
> sostituisci nei `.dc.html` gli indirizzi `/_blob/...` con quelli nuovi.

Poi aggiornare il link in questo file e in `SPECIFICATION.md`.
