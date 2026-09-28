# Componente per il telefono

`scrcpy-server-v4.1` è il server ufficiale di scrcpy 4.1 (Genymobile,
licenza Apache 2.0, testo in `LICENZA-scrcpy.txt`), scaricato da
https://github.com/Genymobile/scrcpy/releases/tag/v4.1 e verificato:

    sha256 deacb991ed2509715160ffdc7907e47b4160eb30d1566217e9047fd5b8850cae

Phonestra lo include nell'eseguibile, lo copia a ogni collegamento in
`/data/local/tmp/phonestra-server.jar` e lo avvia con i permessi della shell:
sul telefono non si installa niente. Più avanti sarà sostituito da una
versione modificata (elenco app con icone, notifiche, pausa dei media alla
caduta del collegamento), sempre con la licenza Apache 2.0 e la citazione.

## Aiutante di Phonestra

`phonestra-aiuto.jar` (sorgenti in `aiuto/`, licenza del progetto) elenca le
app del launcher con nome e icona PNG (le icone adattive con la forma usata
dal telefono). Phonestra lo copia in `/data/local/tmp`, lo esegue con
`app_process` e lo cancella subito. Si ricompila con `aiuto/costruisci.sh`
(servono `javac` e D8 di R8 9.4.26 in `strumenti/r8.jar`: indirizzo e impronta
nello script). Accorgimenti necessari sul Samsung (vedi i commenti nel codice):
`ConfigurationController` come in scrcpy e un carattere predefinito creato a
mano, altrimenti le icone con testo (Calendario) fanno abortire il processo.
