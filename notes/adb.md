# Il nostro client ADB: delayed ack, blocchi e finestre

> 28 settembre 2026. Fase 1 dello studio (`study/system.md` §1.3–1.4,
> `study/README.md`). Codice in `src/adb/` (`mod.rs`, `flusso.rs`,
> `misura.rs`, `prove.rs`); comando `phonestra-prova throughput`.
> **Legenda**: ✅ verificato sul sorgente di adbd; 🔶 ipotesi; 🧪 da misurare
> sul telefono. **Niente di questo è ancora provato sul telefono.**

## Cosa è cambiato

1. **CNXN**: annunciamo `host::features=shell_v2,cmd,stat_v2,delayed_ack` e un
   `max_payload` di **64 KiB** (prima 1 MiB, senza `delayed_ack`).
2. Il *delayed ack* si usa **solo se anche il telefono lo annuncia** nel suo
   CNXN (`features=…delayed_ack`); altrimenti tutto resta come prima: `OPEN`
   con `arg1 = 0`, `OKAY` vuoti, un `WRTE` alla volta per canale.
3. Col *delayed ack*, per ogni canale:
   - `OPEN(locale, finestra, "servizio\0")`: `arg1` = byte che il telefono può
     mandarci senza conferma (predefinito **256 KiB**);
   - l'`OKAY` di risposta porta il saldo che adbd ci concede (32 MiB); noi
     partiamo dal **minimo tra quello e la nostra finestra**, così un invio
     grande (per esempio `sync:`) non riempie la connessione davanti ai
     comandi degli altri canali;
   - `Canale::scrivi` manda `WRTE` finché il saldo è positivo, toglie i byte
     mandati, aggiunge quelli di ogni `OKAY` (anche 0 o negativi) e aspetta
     solo quando il saldo è finito. Torna appena l'ultimo blocco è partito,
     non quando il telefono l'ha ricevuto;
   - a ogni `WRTE` del telefono rispondiamo `OKAY` con 4 byte (int32 LE) =
     lunghezza del blocco, **appena arriva** (come oggi: la coda sul PC non ha
     limite). A richiesta (`OpzioniCanale::conferma_alla_lettura`) l'`OKAY`
     parte invece quando `Canale::leggi` consegna il blocco: la coda sul PC
     resta dentro la finestra e la contropressione arriva fino al processo
     sul telefono. Oggi nessun canale di Phonestra la usa (vedi Rischi).
4. **Configurabile**: `Trasporto { delayed_ack, max_payload, finestra }`
   (`Adb::wifi_con`) e `OpzioniCanale { finestra, conferma_alla_lettura }`
   (`Adb::apri_con`). `Adb::wifi` usa i predefiniti, cambiabili per le prove
   con variabili d'ambiente senza ricompilare:
   `PHONESTRA_ADB_DELAYED_ACK=0`, `PHONESTRA_ADB_PAYLOAD=1m`,
   `PHONESTRA_ADB_FINESTRA=512k`. **Il comportamento di prima** è
   `PHONESTRA_ADB_DELAYED_ACK=0 PHONESTRA_ADB_PAYLOAD=1m`.
5. `Canale::leggi` resta annullabile (timeout, `select!`): la conferma alla
   lettura non aspetta la rete, va in una coda scritta da un compito a parte.
6. `Adb::apri` che fallisce (rifiuto, tempo scaduto) toglie il canale dal
   registro (prima restava lì).

## Fatti dal sorgente di adbd ✅

Sorgenti di `packages/modules/adb`, ramo `main` (android.googlesource.com,
28 set 2026): `adb.cpp`, `sockets.cpp`, `transport.cpp`, `adb.h`,
`docs/dev/delayed_ack.md`. (`protocol.txt` e `docs/dev/asocket.md` oggi davano
503/404; non servono: il codice è più preciso.)

- **Negoziazione**: `atransport::SetFeatures` → `delayed_ack_ =
  CanUseFeature(features_, "delayed_ack")`, cioè entrambe le parti.
  adbd (`!ADB_HOST`) lo mette **sempre** in `supported_features()`; `adb` sul
  PC solo con `ADB_BURST_MODE=1`. Il telefono dell'utente (S23+, Android 16)
  lo annuncia (misurato).
- **Le nostre funzioni le legge dal primo CNXN**, quello prima di `STLS`:
  `handle_new_connection` chiama `update_version` e `parse_banner` e poi, se
  TLS, `send_tls_request`. Anche il `max_payload` è quello del primo CNXN:
  `max_payload = min(nostro, MAX_PAYLOAD = 1 MiB)`; il CNXN che adbd manda
  dopo il TLS ha in `arg1` quel minimo (`send_connect`: `arg1 =
  t->get_max_payload()`).
- **A_OPEN** (`adb.cpp`): `if (t->SupportsDelayedAck() !=
  static_cast<bool>(arg1)) → send_close` («unexpected value of A_OPEN
  arg1»). Col *delayed ack* `s->available_send_bytes = arg1` (il saldo con cui
  adbd **ci manda** dati) e risponde `send_ready(…, INITIAL_DELAYED_ACK_BYTES)`
  = 32 MiB (`adb.h`). Senza, `send_ready(…, 0)` e l'`OKAY` è vuoto.
- **send_ready**: col *delayed ack* l'`OKAY` ha sempre 4 byte (uint32
  copiato così com'è, letto come int32 LE).
- **A_OKAY ricevuto**: contenuto di 4 byte → `acked_bytes` (può essere
  negativo); vuoto → nessuno; altre lunghezze → messaggio scartato.
  `local_socket_ack`: se il canale ha il *delayed ack* e l'`OKAY` no (o
  viceversa) **l'OKAY viene ignorato** («delayed ack mismatch»): un nostro
  `OKAY` vuoto col *delayed ack* attivo bloccherebbe il canale.
  Poi `available_send_bytes += acked` e, se > 0, riprende a leggere dal fd.
- **Lato invio di adbd** (`local_socket_flush_outgoing`): legge dal fd fino a
  `max_payload` byte, toglie la lunghezza dal saldo, manda il `WRTE`; smette
  di leggere quando il saldo è ≤ 0. Quindi **può sforare la nostra finestra
  di un blocco** (fino a finestra + max_payload − 1 byte in volo).
- **Lato ricezione di adbd** (`local_socket_flush_incoming`): dopo ogni
  scrittura sul fd del processo manda `OKAY(byte_scritti)` — anche 0 —
  cioè conferma i byte **consegnati al processo**, non solo ricevuti.
  Senza *delayed ack* manda `OKAY` quando ha scritto qualcosa e in coda resta
  meno di un blocco (può quindi mandarne due per un `WRTE` scritto a metà:
  comportamento vecchio che il nostro client già tollerava).
- Un canale in chiusura con dati in coda viene svuotato prima di essere
  distrutto (`s->closing`): un `CLSE` subito dopo `WRTE` senza conferma non
  perde dati.

## Scelte e valori predefiniti

| Valore | Predefinito | Perché |
|---|---|---|
| `delayed_ack` | annunciato | adbd lo offre sempre (Android 14+ è il nostro minimo); Google misura +70 % (USB 3); via Wi-Fi conta la regolarità 🔶 |
| `max_payload` | 64 KiB | una sola connessione TCP+TLS per tutti i canali, adbd non dà precedenze (una sola coda di `send_packet`): un `WRTE` da 1 MiB a 5 MB/s occupa ~200 ms, uno da 64 KiB ~13 ms. Intestazione: 24 byte ogni 64 KiB (0,04 %) |
| finestra per canale | 256 KiB | 4 blocchi in volo: a 100 ms di RTT ~2,5 MB/s per canale, più del video di Phonestra 🔶; a 5 MB/s un canale pieno fa aspettare gli altri ≤ ~50 ms. 32 MiB (adbd) lascerebbero accumulare secondi di video (SPECIFICATION §14) |
| conferma | all'arrivo | identica a oggi per chi usa i canali: nessun canale non letto può fermare il telefono |

Il `max_payload` si dichiara prima di sapere se il telefono offre il *delayed
ack*: su un telefono senza, 64 KiB limitano ogni canale a 64 KiB per andata e
ritorno (a 50 ms, ~1,3 MB/s). Dato il minimo Android 14 è un caso teorico;
se capita, `PHONESTRA_ADB_PAYLOAD=1m`.

## Misure da fare sul telefono 🧪

Tutte via Wi-Fi, telefono sbloccato, schermo acceso, una alla volta.
`phonestra-prova throughput [MB] [--senza-delayed-ack] [--payload N]
[--finestra N] [--latenza] [--exec] [--alla-lettura]` (MB predefiniti: 100).
Il carico è `head -c N /dev/zero` avviato con `shell,v2,raw:` (niente PTY);
`--exec` usa `exec:` (PTY raw, com'è avviato oggi scrcpy). La latenza è
l'andata e ritorno di 8 byte attraverso `cat` su un secondo canale: 20 misure
a riposo, poi una ogni 50 ms durante il carico.

1. Come prima: `phonestra-prova throughput 100 --senza-delayed-ack --payload 1m --latenza`
2. Nuovo predefinito: `phonestra-prova throughput 100 --latenza`
3. Solo blocchi piccoli: `phonestra-prova throughput 100 --senza-delayed-ack --latenza`
4. Solo delayed ack: `phonestra-prova throughput 100 --payload 1m --latenza`
5. Finestra: `phonestra-prova throughput 100 --finestra 64k --latenza`,
   poi `--finestra 1m`
6. Conferma alla lettura: `phonestra-prova throughput 100 --alla-lettura --latenza`
7. PTY: `phonestra-prova throughput 50 --exec` e `… 50 --exec --senza-delayed-ack --payload 1m`
8. Phonestra vero col nuovo predefinito (video + audio + tocchi, 5 minuti); se
   qualcosa non va, lo stesso con
   `PHONESTRA_ADB_DELAYED_ACK=0 PHONESTRA_ADB_PAYLOAD=1m phonestra` per
   capire se è il trasporto.

Da guardare: MB/s, «pausa massima tra blocchi» e pause ≥ 50 ms, latenza sotto
carico (mediana e massimo) rispetto a quella a riposo. Ipotesi da confermare:

- 🔶 col *delayed ack* il throughput di un canale sale e le pause tra blocchi
  si accorciano (il telefono non aspetta un RTT per blocco);
- 🔶 con blocchi da 64 KiB la latenza di un piccolo messaggio sotto carico
  scende rispetto a 1 MiB (meno attesa dietro un `WRTE` grande), a patto
  che la finestra sia piccola: con finestra grande la coda si sposta nei
  buffer TCP e la latenza torna a salire;
- 🔶 256 KiB bastano a non limitare il throughput con l'RTT del Wi-Fi di casa;
  se la misura 5 mostra che 1 MiB va molto più veloce senza peggiorare la
  latenza, alzare `Trasporto::FINESTRA`;
- 🔶 `exec:` (PTY) limita da solo il throughput: se sì, è un motivo in più per
  avviare il componente con `shell,v2,raw:` e passare i dati su
  `localabstract:` (studio §1.2).

## Rischi

- **Regressioni sul collegamento di oggi**: ora tutti i canali (scrcpy video
  e comandi, aiutante, custode, `sync:`) usano il *delayed ack* e blocchi da
  64 KiB. Le parti delicate: `scrivi` non aspetta più la conferma (nessun
  codice di Phonestra la usava come «il telefono ha ricevuto», ma è da
  osservare con il custode e gli appunti); il video scrcpy arriva in più
  `WRTE` piccoli invece che in uno grande (Phonestra ricompone già i
  pacchetti con `leggi_esatti`). Ritorno immediato al comportamento di prima
  con le variabili d'ambiente sopra, senza ricompilare.
- **Conferma alla lettura**: un canale che nessuno legge (per esempio l'uscita
  di un processo lasciata lì) ferma chi scrive sul telefono appena la finestra
  è piena. Per questo non è il predefinito; è pensata per il video del
  componente nostro, letto sempre da un compito dedicato.
- **Canale abbandonato** (`Canale` buttato senza `chiudi`): come prima, i
  dati successivi non vengono confermati e il telefono si ferma su quel
  canale; ora dopo una finestra (256 KiB) invece che dopo un blocco.
- La finestra non limita la memoria sul PC con la conferma all'arrivo: la
  coda di ricezione resta senza limite, come oggi.

## Prima prova sul telefono (28 set 2026, S23+ Android 16)

`phonestra-prova throughput 100 --latenza` (100 MB, latenza di 8 byte su un
secondo canale ogni 50 ms durante il carico):

| Trasporto | MB/s | Latenza sotto carico (mediana / 95 % / max) | Note |
|---|---|---|---|
| come prima (niente delayed ack, blocchi 1 MiB) | 6,7 | 22 / 55 / 115 ms | 51 pause ≥ 50 ms |
| niente delayed ack, blocchi 64 KiB | 4,6 | 12 / 20 / 65 ms | **collegamento chiuso dal telefono a 73 MB** («connection reset»): da capire |
| delayed ack attivo | — | — | **adbd rifiuta ogni OPEN** (anche `exec:echo`) |

Nel codice di adbd un `OPEN` con `arg1` ≠ 0 viene chiuso se il telefono non
considera attivo il delayed ack per il trasporto: il nostro annuncio nel primo
CNXN (prima del TLS) non viene registrato, benché il telefono offra la
funzione. Da indagare (dove adbd legge le funzioni dell'host col TLS;
confronto con l'adb ufficiale). **Predefinito riportato al comportamento di
prima** (delayed ack spento, blocchi 1 MiB); prove con
`PHONESTRA_ADB_DELAYED_ACK=1 PHONESTRA_ADB_PAYLOAD=64k`.
