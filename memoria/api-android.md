# API di Android per il componente nostro

Il componente per il telefono (`telefono/aiuto`) sostituisce scrcpy un pezzo
alla volta (decisione del 28 set 2026, `decisioni-utente.md`). Qui, **prima di
scrivere il codice di ogni pezzo**, le API scelte, i permessi della shell che
servono e le differenze tra Android 14, 15 e 16. Solo Android 14+ (API 34+).

Il componente gira come la shell (uid 2000, `app_process`), non è un'app
installata: niente finestre di consenso, ma solo i permessi che la shell ha.

## 1. Audio (bozza, da rimisurare)

> Lo studio del 28 set (`studio/audio.md`) mostra che anche REMOTE_SUBMIX passa
> dal *remote submix*, senza orologio vero: la sorgente si sceglie solo dopo il
> confronto in PCM delle tre sorgenti (REMOTE_SUBMIX, LOOP_BACK,
> LOOP_BACK_RENDER). Sintesi e ordine di lavoro in `studio/README.md`. La bozza
> `telefono/aiuto/src/phonestra/Audio.java` serve da base per quelle misure.

- **Sorgente**: `AudioRecord` con `MediaRecorder.AudioSource.REMOTE_SUBMIX`
  (l'uscita intera del telefono, dopo il mixer). Richiede
  `CAPTURE_AUDIO_OUTPUT`, che la shell ha. Mentre la cattura è attiva Android
  non manda l'audio agli altoparlanti (è quello che vogliamo).
  Scartata la cattura per singola app (`AudioPlaybackCaptureConfiguration`):
  nelle prove del 28 set lasciava vuoti di 50–120 ms nei lettori di Facebook e
  Chrome (§41–42 delle prove).
- **Contesto**: `AudioRecord.Builder.setContext()` (API 31) con un contesto del
  pacchetto `com.android.shell`, così l'attribuzione del permesso corrisponde
  all'uid della shell.
- **Formato**: PCM 16 bit, 48 kHz, stereo; letto a blocchi di 1024 campioni.
- **Compressione**: `MediaCodec` AAC-LC (`audio/mp4a-latm`), 192 kbit/s: la
  stessa scelta del registratore dello schermo Samsung, che nelle prove ha dato
  audio senza interruzioni. Opus (quello di scrcpy) è un codificatore software.
- **Orari**: dal numero di campioni letti, non dall'orologio: regolari per
  costruzione (gli orari irregolari di scrcpy causavano micro-interruzioni).
- **Trasporto**: sull'uscita del processo (`exec:`), pacchetti con 8 byte di
  orario e bandiere (bit 62 = configurazione) e 4 byte di lunghezza, come quelli
  che Phonestra già legge. Lettura e spedizione su due thread: il Wi-Fi lento
  non ferma la cattura.

## 2. Video e finestre delle app (da studiare)

## 3. Tocchi, tasti, appunti, comandi (da studiare)
