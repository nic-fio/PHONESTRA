# Componente per il telefono

Tutto quello che gira sul telefono è nostro: `phonestra-aiuto.jar` (sorgenti
in `aiuto/`, licenza del progetto). Phonestra lo include nell'eseguibile, lo
copia a ogni collegamento in `/data/local/tmp` e lo avvia con `app_process` e
i permessi della shell; le copie si cancellano alla fine (le toglie il
custode, vedi sotto): sul telefono non si installa niente. Dal 28 set 2026
non ci sono più componenti di terzi (`memoria/componente.md`, «Fase 3»).

## Aiutante di Phonestra

L'aiutante elenca le app del launcher con nome e icona PNG (le icone adattive
con la forma usata dal telefono). Si ricompila con `aiuto/costruisci.sh`
(servono `javac` e D8 di R8 9.4.26 in `strumenti/r8.jar`: indirizzo e impronta
nello script). Accorgimenti necessari sul Samsung (vedi i commenti nel codice):
il `ConfigurationController` del contesto di sistema e un carattere
predefinito creato a mano, altrimenti le icone con testo (Calendario) fanno
abortire il processo.

Altri comandi dell'aiutante (primo argomento): `sfondo <larghezza>`,
`codificatori` (elenco dei codificatori audio e video) e
`audio [sorgente=submix|loopback|render] [formato=pcm|aac] [priorita=si|no]
[voce=si|no]`, lo strumento di misura dell'audio (`memoria/api-android.md`
§1.1), che resta attivo finché il PC legge. Gli stub in `aiuto/stub/`
coprono solo le API pubbliche; quelle nascoste (`AudioPolicy`) sono chiamate
per riflessione.

## Servizio (componente nostro)

`servizio` è il comando di lunga durata (`memoria/componente.md`): un
processo per collegamento, avviato dal PC con `shell,v2,raw:` e
`--nice-name=phonestra-servizio`, che riceve il segreto sull'ingresso, apre un
socket astratto `phonestra_<casuale>` per i canali (`comandi`, `audio`,
`video:<id>`), manda il `CIAO` con l'autotest delle API nascoste e tiene il
battito col PC. Fa audio, video delle finestre e dello schermo, tocchi e
tasti, appunti e pannello. Un custode (`sh` con `setsid`, nome
`phonestra-custode`) rimette a posto il telefono e cancella le copie del jar
quando il servizio muore, anche di colpo. Classi principali: `Servizio`,
`Protocollo`, `Custode`, `Autotest`, `Nascoste` (adattatori delle API
nascoste), `Contesto` (contesto di sistema e della shell), `Audio`, `Video`,
`Input`, `Appunti`, `Pannello`. Prova dal PC: `phonestra-prova servizio
[secondi] [--sparisci]`.
