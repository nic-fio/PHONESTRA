# Componente nostro al posto di scrcpy: sintesi dello studio (28 set 2026)

Decisioni dell'utente che fanno da confine (`../decisioni-utente.md`):
componente tutto nostro, Java solo dove Android lo impone, **solo Android 14+**,
**stesse funzioni di oggi**: si correggono solo i difetti di prestazioni
(audio che si interrompe, video che scende di fotogrammi o riparte).

Studi completi, con fonti e distinzione tra fatti verificati e ipotesi:
[video.md](video.md) · [audio.md](audio.md) · [input.md](input.md) ·
[sistema.md](sistema.md).

## Cosa abbiamo capito dei difetti di oggi

| Difetto | Causa più probabile | Fonte |
|---|---|---|
| Micro-interruzioni dell'audio | Le catture «playback» e «output» passano dal *remote submix*, che non ha un orologio vero: ritmo irregolare, zeri quando mancano dati. Il registratore Samsung (audio perfetto) probabilmente usa `LOOP_BACK_RENDER`, col ritmo dell'altoparlante vero | audio.md §1, prove §41–42 |
| Orari audio a raffica | scrcpy con Opus usa l'ora di uscita dal codificatore; contare i campioni (come fa già la nostra bozza) è corretto | audio.md §4 |
| Ripartenze del video di 1–2 s | a ogni «ricomincia video» scrcpy ricrea il codificatore; basta chiedere un fotogramma chiave | video.md §3 |
| Traffico lento o a scatti | il nostro ADB non attiva il *delayed ack* che il telefono offre (un solo blocco in viaggio per canale; Google misura +70 %) e un blocco video grande può far attendere audio e comandi | sistema.md §1 |
| Audio disturbato dal lavoro del PC sul telefono | `dumpsys` ripetuti (~100 ms) e video nello stesso processo; eventi del sistema al posto dei `dumpsys`, thread audio ad alta priorità | video.md §2, audio.md §6 |

## Architettura proposta

- **Un solo processo per collegamento** sul telefono (`app_process`, l'aiutante
  esteso), invece di uno scrcpy per finestra più l'aiutante a ogni richiesta.
- **Canali**: il processo si avvia con `shell,v2` (codice d'uscita, errori
  separati); i dati passano su socket `localabstract:` con nome casuale e un
  segreto: uno per i comandi, uno per l'audio, uno per il video di ogni finestra.
- **Formato dei messaggi**: quello che Phonestra già legge (8 byte di orario e
  bandiere, 4 di lunghezza, dati) per audio e video; messaggi di comando
  numerati e con lunghezza.
- **Custode separato** (`setsid`): se il processo muore o il PC sparisce
  rimette tutto (pausa dei media, volume, tempo di spegnimento, pannello, task
  degli schermi virtuali). **Battito** ogni secondo: senza risposta per 5 s il
  processo si chiude (adbd sul Wi-Fi non se ne accorge da solo).
- **API nascoste** chiuse in piccoli adattatori con ripieghi per le varianti
  di Android e Samsung, e un **autotest all'avvio** che dice al PC cosa
  funziona su quel telefono.
- **Audio**: sorgente scelta dopo le misure (sotto); orari dal conteggio dei
  campioni; thread ad alta priorità; PCM o AAC dopo le misure.
- **Video**: schermi virtuali `TRUSTED`/`OWN_FOCUS` (servono anche ai tasti in
  più finestre), fotogramma chiave a comando, codifica sospesa per le finestre
  nascoste, eventi dei task al posto dei `dumpsys`.
- **Input**: stesse funzioni di oggi (tocchi, più dita, rotellina, tasti,
  testo, incolla, indietro), iniezione asincrona indirizzata allo schermo giusto;
  appunti letti dal componente.
- **PC**: *delayed ack* e blocchi più piccoli nel nostro ADB in Rust.

## Decisioni che servono all'utente

1. **Codice di scrcpy**: due pezzi piccoli e delicati (preparazione del
   contesto di sistema, accorgimenti per Samsung) si possono **copiare**
   citando la licenza Apache, oppure **riscrivere** prendendo scrcpy solo come
   documentazione. Proposta: riscriverli (codice nostro, come deciso),
   annotando da quale problema noto proteggono.
2. **Prova del gesto «indietro»** (video.md, prova 3): un'impostazione degli
   schermi virtuali, su S23+ e S26 con Android 16, può bloccare il gesto fino
   al riavvio del telefono. La prova può richiedere un riavvio.
3. **Audio col telefono che suona** (`LOOP_BACK_RENDER`): da decidere solo dopo
   aver misurato se la copia è presa prima del volume (in quel caso basta il
   telefono al minimo).

## Ordine di lavoro

**Fase 0 — misure sul telefono** (in serie, una alla volta; decidono
l'architettura):
1. il banner del telefono annuncia `delayed_ack`? (1 minuto);
2. audio: le tre sorgenti a confronto in PCM, con contatore degli zeri sul
   telefono, fotogrammi/s del video e `dumpsys` durante il registratore Samsung;
3. video: tempo del fotogramma chiave a comando, quanti codificatori insieme;
4. gesto «indietro» (con il consenso dell'utente);
5. caduta del Wi-Fi senza chiusura: cosa resta sul telefono.

**Fase 1 — sviluppo in parallelo** (agenti in copie separate del repository,
ognuno con compilazione, test e clippy puliti; prove sul telefono e unione in
serie):
- nostro ADB: *delayed ack*, blocchi più piccoli, `shell,v2`;
- scheletro del componente: processo, canali, segreto, battito, custode,
  adattatori con autotest (prima degli altri tre, che ci si appoggiano);
- poi in parallelo: audio, input, video.

**Fase 2** — Phonestra usa il componente nostro un pezzo alla volta (prima
l'audio), con scrcpy come riserva.

**Fase 3** — scrcpy tolto dall'AppImage e dalla licenza.

## Rimandato (novità, non stesse funzioni)

Tastiera italiana UHID, notifiche in tempo reale (la shell avrebbe il permesso
di SystemUI), ripetizione dei tasti, controller da gioco, voce di WhatsApp al
PC, webcam e microfono (già tolti il 27 set). Dettagli nei rispettivi studi.
