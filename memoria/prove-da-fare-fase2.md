# Prove da fare: video e input dal componente nostro (fase 2)

Cosa è cambiato: finestre delle app e schermo del telefono nel drawer usano il
componente nostro per video, tocchi e tasti (`componente.md` §14); un solo
servizio per collegamento, anche per l'audio. Gli appunti (copie dal telefono
al PC) passano ancora da scrcpy. **Ci si aspetta lo stesso comportamento di
prima**, solo più fluido.

Avvio: Phonestra vero (release) con `PHONESTRA_DEBUG=1`, telefono sbloccato.
Nel terminale all'inizio **non** deve comparire «[componente] non avviato»:
vorrebbe dire che si sta usando scrcpy.

## Prima: i pezzi da soli (nessuna finestra di Phonestra aperta)

```
phonestra-prova video-componente app
phonestra-prova video-componente schermo --secondi 20
phonestra-prova input-componente tutte
phonestra-prova audio-componente 20 aac
```
Attesi: «prova riuscita» in tutte (come in prove §45–46).

## Poi Phonestra, voce per voce

| # | Cosa fare | Cosa aspettarsi |
|---|---|---|
| 1 | Aprire il drawer | Schermo del telefono visibile e aggiornato; pannello del telefono nero |
| 2 | Clic e trascinamento sullo schermo del telefono nel drawer; tasto Esc | Il telefono risponde; Esc = indietro |
| 3 | Aprire un'app (es. Chrome) dal drawer | Finestra con l'app in ~1 s, come prima |
| 4 | Clic, trascinamento, rotellina, Pagina giù, frecce | Come prima; scorrimento fluido |
| 5 | Clic destro su una parola; tasto «indietro» del mouse | Pressione lunga (selezione, menu); indietro |
| 6 | Ctrl + rotellina, Ctrl +/−, pizzico sul touchpad (Maps) | Zoom |
| 7 | Scrivere in un campo di testo: «Ciao àèìòù €», Invio, Backspace, Ctrl+A | Testo giusto, accentate comprese |
| 8 | Copiare un testo sul PC, Ctrl+V nell'app | Incollato; **non** torna indietro come copia nel PC |
| 9 | Copiare un testo nell'app sul telefono | Arriva negli appunti del PC (ancora scrcpy) |
| 10 | Allargare, stringere, massimizzare la finestra | L'app segue la finestra; a tutto schermo si ricrea con la densità giusta |
| 11 | Aprire Facebook, poi allargare la finestra | Finestra a misura fissa verticale (app solo verticale) |
| 12 | Aprire YouTube e mettere un video a schermo intero | Video nella finestra, non ruotato a striscia |
| 13 | Aprire un'app con schermata protetta (banca, Bitwarden) | Riquadro «Schermata protetta» al posto del nero; sparisce uscendo |
| 14 | Nel drawer, clic destro sull'icona di un'app › «Informazioni sull'app» | Pagina delle impostazioni dell'app nella finestra |
| 15 | Screenshot e Registra (10 s su YouTube) | PNG salvato; MP4 con video e audio in sincrono, parte subito |
| 16 | Due o tre app aperte insieme, poi chiuderne una | Le altre continuano; pannello del telefono resta nero |
| 17 | Chiudere l'ultima app (drawer aperto) e poi il drawer | App tolte dalle recenti del telefono; pannello riacceso alla fine |
| 18 | Bloccare il telefono col tasto, poi sbloccarlo | Fascia «Riconnessione…», poi l'app torna dov'era; pannello acceso (in mano) finché non si usa dal PC |
| 19 | Spegnere il Wi-Fi del telefono 10 s e riaccenderlo | Come 18: le finestre ripartono da sole |
| 20 | Chiudere Phonestra con un'app aperta | Telefono come prima: pannello acceso, volume e spegnimento dello schermo rimessi |
| 21 | Riaprire con `PHONESTRA_COMPONENTE=scrcpy` e ripetere 3, 4, 7, 17 | Tutto come prima (riserva scrcpy) |

## Dopo le prove

```
phonestra-prova shell 'ps -A | grep [p]honestra; ls /data/local/tmp'
phonestra-prova shell 'dumpsys display | grep -c "\"phonestra-"'
```
Attesi: nessun processo `phonestra-servizio`/`phonestra-custode`, nessun
`phonestra-servizio-*.jar`, 0 schermi `phonestra-`.

Nel log di `PHONESTRA_DEBUG=1` da guardare: `[servizio] il telefono
risponde: …` (errori di comandi mandati senza attesa), `[finestra] il
telefono ha chiuso la sessione`, `[componente] il servizio sul telefono si è
fermato`, e i fotogrammi/s di `[video]`.
