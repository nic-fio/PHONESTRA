# Suite di prove della sincronia audio-video (progetto, 5 ottobre 2026)

**Il difetto.** Su YouTube e Facebook, dentro Phonestra, l'audio e il video si
disallineano di poco, e non sempre. Riprodotti sul telefono, gli stessi video
sono a posto. Finora il giudizio è stato a orecchio («leggermente in ritardo»,
«qualche millisecondo»: §50, §52, §59 di `connection-tests.md`) e ogni
correzione ha spostato il problema. Le prove servono a dare **un numero a ogni
punto della catena**, così si vede dove nasce lo scarto invece di indovinarlo.

## 1. Come funziona oggi (dal codice, 5 ott)

| Tratto | Audio | Video |
|---|---|---|
| Orario sul telefono | campioni letti × 10⁶ / 48 000 (`Audio.java:459`): orologio del submix | µs dal primo fotogramma (`SessioneVideo.java`): orologio monotono |
| Legame fra i due | **nessuno**: nessun orologio comune, nessun riferimento scambiato | |
| Trasporto | stesso TCP ADB, blocchi da 64 KiB (§50) | |
| PC, decodifica | `avdec_aac` | `h264parse ! decodebin` |
| PC, orario di uscita | orario del telefono + scarto fissato dal **primo pacchetto** + margine 80→300 ms che sale a ogni ritardo e quasi mai scende (§52) | **nessuno**: `do-timestamp=true`, `gtk4paintablesink sync=false` (`finestra.rs:274`), cioè si mostra appena decodificato |
| PC, uscita | `autoaudiosink` (latenza di PipeWire/Pulse non misurata) | GTK, frame clock del compositore |

Quindi lo scarto A/V visto dall'utente è, a grandi linee:

```
scarto = (cattura audio − cattura video sul telefono)
       + (trasporto audio − trasporto video)
       + margine audio (80…300 ms, variabile)
       + latenza dell'uscita audio − (decodifica + disegno del video)
```

Nessuno di questi termini è misurato oggi.

## 2. Ipotesi da confermare o scartare

| # | Ipotesi | Cosa si vedrebbe | Prova che decide |
|---|---|---|---|
| H1 | **Margine audio** senza contrappeso sul video: l'audio arriva in ritardo per costruzione, e a scatti di +40 ms | scarto ≈ margine + costante, e salta quando sale il margine | P4 contro registro del margine (T1, T2) |
| H2 | **Due orologi diversi**: lo scarto si fissa al primo pacchetto, poi il quarzo del PC e l'orologio del telefono si allontanano (decine di ppm = decine di ms in pochi minuti); si corregge solo oltre 200 ms o con un ritardo | scarto che cresce o cala in linea retta nel tempo, poi salta | pendenza in ms/min su 30 min (T3) |
| H3 | **Video senza coda**: le pause del Wi-Fi (150–500 ms, §18) fanno arrivare il video in ritardo e poi a raffica, mentre l'audio è protetto dal margine | scarto che oscilla con l'arrivo dei fotogrammi | P2 contro P4; cavo USB contro Wi-Fi (T4) |
| H4 | **App e loopback**: YouTube e Facebook (ExoPlayer) agganciano l'immagine alla posizione dell'`AudioTrack`; col loopback il submix ha una latenza diversa da quella dell'altoparlante, e l'app compensa male | scarto già presente **all'origine** (P1), diverso tra app e lettore locale | P1 con YouTube/Facebook contro il lettore video del telefono (T0, T1, T2) |
| H5 | **Ritardo di avvio dell'AAC** (priming del codificatore, ~2 048 campioni ≈ 43 ms) non tolto dal decodificatore | scarto costante di ~40 ms in AAC, assente in PCM | AAC contro PCM (T8) |
| H6 | **Decodifica video a fili** (`avdec_h264` con più thread trattiene alcuni fotogrammi) | P3 video − P2 video = alcuni fotogrammi | P2/P3 per fotogramma |
| H7 | **Uscita audio variabile**: il quantum di PipeWire cambia se sul PC suona altro (browser) | scarto che cambia aprendo o chiudendo un'altra fonte audio | T9 |
| H8 | **Ripartenze**: la cattura audio riparte 5 s dopo lo specchio e a ogni ricreazione (§49); lo scarto si rifissa a un valore diverso | scarto che cambia a gradino dopo un ricollegamento o uno specchio ricreato | T6 |
| H9 | **Display a 24 Hz** a pannello spento (§59): fotogrammi composti a 24 Hz, audio a tempo pieno | scarto a dente di sega fino a ~40 ms | pannello acceso contro spento (T7) |

Le ipotesi non si escludono: è probabile che lo scarto sia una somma di più
termini. Le prove li separano.

## 3. Lo strumento: video «lampo + bip» già pubblici

Su YouTube ci sono molti video di prova della sincronia (lampo bianco e bip
nello stesso istante), in risoluzioni da Full HD a 8K e a 24, 25, 30, 50 e
60 fps: si usano quelli, senza caricare niente (scelta dell'utente, 5 ott).

Come sceglierli:
- **risoluzione**: Phonestra cattura lo schermo del telefono alla sua
  risoluzione, quindi 4K o 8K non danno una misura migliore. Caricano però il
  decodificatore del telefono, e questo diventa una variabile in più. Si usa
  **1080p**; l'8K solo come prova di carico a parte;
- **fps**: una versione a 24, una a 30 e una a 60 (H9 e H6 dipendono dagli
  fps);
- **lo stesso video anche come file**: per T0 (lettore del telefono) serve
  lo stesso contenuto fuori da YouTube. Le prove di PhotoJoseph
  (https://photojoseph.com/AVsyncTest) sono su YouTube e scaricabili in H.264
  a 23,98, 24, 25, 29,97, 50 e 59,94 fps: candidate da guardare per prime.

Cosa cambia rispetto a un video nostro:
- intervalli **regolari** (di solito 1 s): un marcatore si abbina a quello
  giusto finché lo scarto resta sotto mezzo periodo (±500 ms), molto oltre i
  300 ms del margine audio massimo;
- **nessun numero** sui marcatori: un marcatore perso si riconosce dal salto
  di un periodo nei tempi;
- rivelatori **generici**: il lampo è un salto di luminosità nel riquadro
  indicato in configurazione; la frequenza del bip si ricava dalle prime
  battute (picco dello spettro), poi Goertzel su quella.

**Facebook (T2) resta scoperto**: senza caricare un video, lì non c'è un
marcatore noto. Da decidere più avanti, dopo le prime misure su YouTube.

## 4. Dove si misura

Un modo di misura dentro il programma vero (`PHONESTRA_AVSYNC=<file.csv>`),
non in un banco a parte: deve passare per la stessa pipeline che usa l'utente.
Due rivelatori:
- **lampo**: luminosità media del centro del fotogramma decodificato (sonda
  sul pad dopo il decodificatore, solo il piano Y, pochi µs);
- **bip**: energia a 1 kHz con Goertzel sul PCM decodificato, a blocchi di
  1 ms.

| Punto | Video | Audio | Cosa dice |
|---|---|---|---|
| **P1** telefono, all'origine | orario del fotogramma (monotono) | campione del bip convertito in orario monotono con `AudioRecord.getTimestamp` (già letto per la deriva, `Audio.java:534`) | lo scarto **creato dal telefono** (app, submix, composizione): H4, H9 |
| **P2** arrivo sul PC | `Instant` d'arrivo del pacchetto | `Instant` d'arrivo del pacchetto | effetto del **trasporto** (Wi-Fi, ADB): H3 |
| **P3** dopo la decodifica | uscita del decodificatore | uscita di `avdec_aac` | decodifica: H5, H6 |
| **P4** presentazione | ora di presentazione del frame clock di GTK (`GdkFrameTimings`) per il fotogramma col lampo | orario del buffer nella pipeline + latenza dichiarata dall'uscita (query `latency`), riportato all'orologio monotono | **quello che vede e sente l'utente** |
| **P5** verità esterna (facoltativa) | fotocamera frontale del tablet che guarda lo schermo in uno specchio (30 fps) e microfono del tablet per le casse: un solo fotogramma vale ±17 ms, la media su qualche centinaio di marcatori a intervalli irregolari scende a pochi ms | | conferma che P4 non mente; una volta sola |

Per P1 il telefono deve mandare, una volta al secondo e per ogni flusso, la
coppia «orario del flusso ↔ orologio monotono»: è la sola aggiunta al
componente (righe di testo sui canali esistenti).

Insieme ai marcatori, nel CSV: margine audio, ritardi, riallineamenti,
fotogrammi al secondo arrivati, pausa massima, segnale Wi-Fi del telefono
(`cmd wifi status`, ogni 30 s come oggi), stato del pannello, Hz del display.

Uscita a fine prova: `phonestra-prova avsync-analizza <file.csv>` scrive per
ogni punto media, deviazione, minimo/massimo, **pendenza in ms/min** e salti,
e una pagina con il grafico dello scarto nel tempo.

## 5. Le prove

Ogni prova: 5 minuti (T3: 30), video lampo + bip, telefono e PC nelle stesse
condizioni, **una variabile alla volta** (metodo concordato, `next-session.md`).

| Prova | Variabile | Ipotesi |
|---|---|---|
| T0 | lettore video del telefono, in finestra | taratura; H4 (riferimento) |
| T1 | YouTube, in finestra | H1, H4 |
| T2 | Facebook, in finestra | H1, H4 |
| T3 | YouTube, 30 minuti | H2 |
| T4 | Wi-Fi vicino al router / lontano / cavo USB | H3 |
| T5 | finestra dell'app contro specchio nel drawer | H3, H6 |
| T6 | ricollegamento e specchio ricreato a metà prova | H8 |
| T7 | pannello acceso contro spento | H9 |
| T8 | AAC contro PCM (`PHONESTRA_AUDIO_CODEC=pcm`) | H5 |
| T9 | un'altra fonte audio sul PC accesa a metà | H7 |
| T10 | carico: drawer + due finestre aperte | H3, H6 |

**Obiettivo** (EBU R37): l'audio al massimo 40 ms in anticipo o 60 ms in
ritardo sul video, **e stabile** (deviazione sotto 15 ms, pendenza zero).

## 6. Prove automatiche, senza telefono (`cargo test`)

- **Rivelatori**: lampo e bip su un video sintetico (`videotestsrc`,
  `audiotestsrc`) con scarti noti (0, +80, −40 ms): devono ritrovarli entro
  ±2 ms.
- **Orologi che si allontanano**: `Orari` e `Margine` con un telefono più
  lento o più veloce di 50 ppm per 30 minuti simulati: oggi lo scarto deve
  risultare deriva + salti (H2); servirà a provare il rimedio.
- **Tracce registrate e rigiocate**: il modo di misura salva gli arrivi veri
  (orario del telefono, `Instant` d'arrivo, dimensione) di audio e video;
  una prova li rigioca nella logica del PC. Così una cura si confronta con
  quella di oggi **sulle stesse tracce**, senza disturbare l'utente.

## 7. Ordine dei lavori

1. Scelta dei video su YouTube (1080p a 24, 30, 60 fps) e formato del CSV;
   rivelatori con le loro prove automatiche.
2. P2, P3, P4 sul PC (nessun cambio al telefono).
3. P1: righe dell'orologio dal componente (`CanaleAudio`, `SessioneVideo`).
4. Analisi e grafico.
5. T0, T1, T2 con l'utente; poi le altre secondo quanto dicono le prime.
6. Solo allora la cura, provata prima sulle tracce rigiocate e poi su YouTube
   **e** Facebook.

Vincoli del tablet (7,5 GB di RAM e 4 GB di swap): misure leggere (solo luminosità e
Goertzel), niente compilazioni in parallelo alle prove.

## 8. Da decidere con l'utente

- ~~Caricare il video di prova su YouTube e Facebook~~: no per YouTube, si
  usano i video pubblici (5 ott); Facebook da decidere dopo le prime misure.
- ~~Secondo telefono per la ripresa a 240 fps (P5)~~: non c'è (5 ott); P5 si fa
  con fotocamera e microfono del tablet e uno specchio.

## 9. Prima prova, 5 ottobre sera (30 min, solo registro)

Phonestra 1.0.0 con `PHONESTRA_DEBUG=1`, YouTube col video a 59,94 fps di
PhotoJoseph (https://youtu.be/YxJHqN_zlv4) per 30 minuti, Wi-Fi, telefono
fermo. Nessuna misura dello scarto: solo il registro dell'audio.

**Margine audio** (ritardo aggiunto all'audio, nessun ritardo uguale sul video):

| min | 0 | 3,7 | 6,2 | 11 | 12,3 | 14,8 | 18,3 | 19,4 | 20,3 | 26,3 | 30 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| ms | 80 | 120 | 138 | 157 | 197 | 216 | 173 | 293 | 272 | 229 | 229 |

- sale di +40 ms a ogni pacchetto in ritardo (8 in 30 min, uno o due alla
  volta) e scende di ~20 ms solo dopo minuti tranquilli: in 30 minuti è
  andato da 80 a 293 ms e ha chiuso a 229;
- due **riallineamenti degli orologi** (min 11 e 19,2): il secondo è seguito
  subito dal salto a 253 e 293 ms;
- `deriva_ms` del telefono (submix contro orologio monotono) fermo a
  51,3–53,4 ms per tutta la prova: dal lato del telefono i due orologi non si
  allontanano.

**Cosa dice**: H1 è quasi certa. Lo scarto audio-video cambia di 100–200 ms
nella stessa sessione, a gradini, e dipende da quanti pacchetti sono arrivati
in ritardo **fino a quel momento**: spiega il «non sempre». Lo conferma la
misura vera (P4), che resta da fare.

**Il video di PhotoJoseph non va bene**: l'autore dichiara fuori sincronia la
copia su YouTube («OUT OF SYNC — USE LINK BELOW TO DOWNLOAD») e il
riferimento non è un lampo ma un pallino che scorre su una scala di −30…+30
fotogrammi, 60 posizioni al secondo: a occhio non si legge (l'utente vedeva
sempre il primo pallino, −30) e il rivelatore dovrebbe seguirne la posizione.
Servono video con un **lampo a tutto schermo**; per T0 il file scaricato o un
video nostro.

Note: la fotocamera del tablet non funziona su questo kernel (progetto
INTEL-CAMERA), quindi P5 con lo specchio per ora non si può fare. Il registro
è in `target/avsync-2026-10-05.log` (fuori da git: contiene i nomi delle app
del telefono).

## 10. Prima misura vera, 6 ottobre mattina (30 min, P4 approssimato)

Misura dentro Phonestra (`PHONESTRA_AVSYNC`, `src/avsync.rs`): luminosità di
ogni fotogramma decodificato (prima di GTK: il disegno non è contato) e, per
ogni blocco di audio decodificato, l'istante in cui esce (orario del blocco +
latenza dichiarata dall'uscita). Analisi: `experiments/avsync-analizza.py`.
Build di debug, Wi-Fi, drawer col telefono, YouTube.

**Video**: «1080p50 Audio/video sync tester - 30 minute with timecode»
(https://www.youtube.com/watch?v=4S5KBlieT0I). Non ha un lampo: un cerchio
bianco si svuota in un secondo e torna pieno di colpo (+7 di luminosità media
nel drawer). **Nel file di YouTube il bip arriva 30–60 ms (≈50) dopo il
cerchio pieno** (tracce scaricate e misurate con ffmpeg): è la taratura.
Un bip al secondo: lo scarto si legge modulo 1 s (in −200…+800 ms); seguirlo
«per continuità» sbaglia di un periodo dopo un disturbo.

| min | 3–5 | 6–10 | 11–21 | 22–31 |
|---|---|---|---|---|
| audio dopo il video, mediana | +300…+334 | +395…+547 | +464…+579 | +425…+586 ms |

- Tolti i ~50 ms dell'origine: **Phonestra mette l'audio 250–530 ms dopo il
  video**. La lettura a occhio di ieri (pallino a −30 fotogrammi, ~500 ms)
  era giusta.
- **Il margine non basta a spiegarlo**: con lo stesso margine (96 ms) lo
  scarto passa da ~310 (min 3–5) a ~520 ms (min 15–21). C'è un ritardo che
  si accumula da sé nei primi 10 minuti, poi resta intorno a 450–550 ms: H2
  (orologi) o una coda che cresce fra `appsrc` e l'uscita. Da separare con P2
  e P3 (arrivo e decodifica).
- Uscita audio: HDMI del monitor, PipeWire quantum 2048 sul dispositivo e
  4320 (90 ms) sul flusso di Phonestra.
- Minuti 7–8 disturbati da me (download e ffmpeg sul tablet, catture dello
  schermo del telefono): durante le prove niente lavori pesanti, niente
  `screencap`.
- Riavviare Phonestra fa addormentare e bloccare il telefono (il custode
  rimette il tempo di spegnimento): serve l'utente per sbloccarlo.

Dati in `target/avsync/prova-20261006-0918.*` (fuori da git).

## 11. Seconda misura, 6 ottobre 10:08–10:33: dove nasce lo scarto

Stessa prova del §10 con due dati in più: arrivo di ogni pacchetto dal
telefono con il suo orario (`RA`, `RV`: P2) e, per ogni blocco audio,
l'attesa fra la decodifica e l'uscita.

| Tratto | Ritardo dell'audio sul video |
|---|---|
| Trasporto (arrivo − orario del telefono) | costante: +3 ms in 25 min, audio e video |
| Già alla decodifica sul PC, tolti i ~50 ms del video | **160–355 ms**, variabile (min 4: 158, min 8: 355, min 20: 324) |
| PC: dalla decodifica all'uscita (margine + latenza dell'uscita, 110 ms dichiarati) | **130–160 ms**, stabile |
| Totale | 320–500 ms (uscita 365–547 ms, tolti i 50 del video) |

- **H2 scartata**: nessuna deriva fra telefono e PC, nessuna coda che cresce
  nel trasporto.
- **La parte grande e variabile nasce sul telefono**: l'audio parte già
  160–355 ms dopo il fotogramma corrispondente. Candidati: latenza della
  cattura in loopback e del suo buffer, ritardo del codificatore AAC (H5),
  YouTube che aggancia l'immagine alla posizione dell'`AudioTrack` mentre il
  submix ha un'altra latenza (H4). Prossimo: P1 nel componente (orario di
  cattura del campione con `AudioRecord.getTimestamp`, orario del
  fotogramma) e T0 col lettore del telefono contro YouTube.
- Il PC aggiunge ~150 ms fissi senza un ritardo uguale sul video: anche da
  solo è oltre il limite EBU (60 ms).
- Fra le due prove una chiamata: prima di ogni comando al telefono
  controllare `mCallState`, non solo prima dei riavvii.

## 12. Prima cura: un orologio comune (6 ottobre, 10:39)

- **Helper**: l'audio ha come orario l'istante monotono di cattura
  (`AudioRecord.getTimestamp` per il campione 0 + conteggio dei campioni), i
  fotogrammi il loro orario monotono assoluto (prima: µs dal primo
  fotogramma). Verificato: `pts_us` dei fotogrammi = monotono − 81 ms.
- **PC** (`src/sincronia.rs`): a ogni pacchetto l'audio annuncia a che ora del
  PC suonerà quell'istante del telefono (orario + latenza dell'uscita); ogni
  fotogramma riceve l'orario corrispondente e `gtk4paintablesink` ha `sync`
  acceso. Senza annunci da più di 1 s: fotogrammi subito, `sync` spento.
- Trasporto sullo stesso orologio: arrivo − orario = audio 26 ms più tardi
  del video. **Lo scarto di 160–355 ms «sul telefono» del §11 non era nel
  trasporto né nella cattura**: veniva dai due orologi slegati (il PC
  mostrava il video appena arrivato e l'audio dopo il suo margine).

| min (dalle 10:40) | 1 | 1,5 | 2 | 2,5 | 3 | 3,5 | 4 | 4,5 |
|---|---|---|---|---|---|---|---|---|
| audio dopo il video, mediana (ms) | 153 | 150 | 151 | 154 | 156 | 155 | 157 | 149 |

Variazione ±20 ms, a dente di sega (scende ~1 ms/s, poi risale di ~30).
Tolti i ~50 ms del video originale: **resta ~100 ms costante** (prima
250–530 e variabile). Da capire: H4 (l'app col loopback), latenza
dell'uscita dichiarata male, o disegno del fotogramma non contato. Se è
costante su YouTube, Facebook e lettore del telefono, si toglie con un
valore fisso.

**Effetto collaterale da decidere**: mentre c'è audio (anche silenzio: il
loopback manda pacchetti di continuo) il video aspetta l'audio, quindi la
risposta ai clic arriva più tardi.
