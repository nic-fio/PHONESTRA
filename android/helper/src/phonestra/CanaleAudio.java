package phonestra;

import android.media.MediaCodec;
import android.net.LocalSocket;

import java.io.BufferedOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;

/**
 * Il canale «audio» del servizio (notes/component.md, «Audio»): l'audio del
 * telefono verso il PC, con la ricetta scelta dalle misure (prove §42–43):
 * cattura <b>loopback</b> (AudioPolicy con {@code ROUTE_FLAG_LOOP_BACK}: il
 * telefono intanto tace), <b>AAC-LC 192 kbit/s</b> (PCM come riserva), orari
 * dal conteggio dei campioni, lettura a priorità −19. Cattura, lettura e
 * codifica sono quelle dello strumento di misura ({@link Audio}), già provate.
 *
 * <p>Tipo del canale: {@code audio} o {@code audio:aac} (AAC), {@code audio:pcm}
 * (PCM 16 bit, 48 kHz, stereo). Un canale solo alla volta: uno nuovo prende il
 * posto del vecchio (che si chiude e toglie la politica prima che l'altro la
 * registri).
 *
 * <p>Pacchetti (servizio → PC), come quelli dello strumento: 8 byte di orario
 * in µs big-endian con due bandiere, 4 byte di lunghezza, i dati.
 * <ul>
 *   <li>bit 61: testo UTF-8 {@code tipo chiave=valore …}. Il primo pacchetto
 *       del canale è sempre un testo: {@code inizio formato=aac|pcm …} se la
 *       cattura è partita, {@code errore …} se no (e il canale si chiude);
 *       poi {@code lettura}, {@code misura} (una al secondo), {@code avviso};
 *   <li>bit 62: configurazione del codec (AAC: l'AudioSpecificConfig, 2 byte),
 *       prima di qualsiasi pacchetto di dati;
 *   <li>senza bandiere: dati (AAC: un frame di 1024 campioni; PCM: 1024
 *       campioni), orario = campioni letti × 10⁶ / 48000.
 * </ul>
 * Il PC non manda niente su questo canale: chiuderlo ferma la cattura.
 *
 * <p>Thread: lettura (−19), codifica, spedizione, e una sentinella che si
 * accorge della chiusura del canale. Se il Wi-Fi rallenta la coda si riempie
 * e si perdono i pacchetti più vecchi (contati in {@code persi}), ma la
 * lettura non si ferma mai.
 *
 * <p>Politica audio: si toglie alla chiusura del canale e, con
 * {@code System.exit} (fine del servizio), nel gancio di chiusura. Se il
 * processo muore di colpo ({@code kill -9}) la toglie Android: la politica è
 * legata al binder del processo e {@code AudioService} la rilascia quando il
 * binder muore. Il custode non può farlo (non c'è un comando di shell che tolga
 * la politica di un altro processo) e non serve.
 */
final class CanaleAudio {
    /** Attesa massima perché il canale precedente si chiuda. */
    private static final long ATTESA_CHIUSURA_MS = 3000;

    private static CanaleAudio attivo;
    private static boolean gancioInstallato;

    private final LocalSocket socket;
    private final boolean aac;
    /** Scatta quando il canale deve fermarsi (PC che chiude, errore, canale nuovo). */
    private final CountDownLatch ferma = new CountDownLatch(1);
    /** Scatta quando il canale ha finito di chiudersi (politica tolta). */
    private final CountDownLatch chiuso = new CountDownLatch(1);
    private Audio.Cattura cattura;
    private boolean catturaChiusa;

    private CanaleAudio(LocalSocket socket, boolean aac) {
        this.socket = socket;
        this.aac = aac;
    }

    /** Il gestore del tipo «audio» ({@code Servizio.TIPI}). */
    static void gestisci(LocalSocket socket, String tipo) throws Exception {
        Boolean aac = formato(tipo);
        if (aac == null) {
            OutputStream o = socket.getOutputStream();
            o.write(Audio.testo(0, "errore formato sconosciuto: " + tipo));
            o.flush();
            socket.close();
            return;
        }
        CanaleAudio c = new CanaleAudio(socket, aac);
        CanaleAudio vecchio;
        synchronized (CanaleAudio.class) {
            vecchio = attivo;
            attivo = c;
            if (!gancioInstallato) {
                // System.exit (fine del servizio, anche per battito mancato): la
                // politica si toglie qui; con kill -9 la toglie Android.
                Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                    CanaleAudio a;
                    synchronized (CanaleAudio.class) {
                        a = attivo;
                    }
                    if (a != null) {
                        a.chiudiCattura();
                    }
                }, "audio-fine"));
                gancioInstallato = true;
            }
        }
        if (vecchio != null) {
            vecchio.ferma.countDown();
            if (!vecchio.chiuso.await(ATTESA_CHIUSURA_MS, TimeUnit.MILLISECONDS)) {
                log("il canale audio precedente non si è chiuso entro " + ATTESA_CHIUSURA_MS + " ms");
            }
        }
        try {
            c.esegui();
        } finally {
            synchronized (CanaleAudio.class) {
                if (attivo == c) {
                    attivo = null;
                }
            }
        }
    }

    /**
     * Il formato dal tipo del canale: {@code true} = AAC, {@code false} = PCM,
     * {@code null} = sconosciuto.
     */
    static Boolean formato(String tipo) {
        switch (tipo) {
            case "audio":
            case "audio:aac":
                return Boolean.TRUE;
            case "audio:pcm":
                return Boolean.FALSE;
            default:
                return null;
        }
    }

    private void esegui() throws Exception {
        BlockingQueue<byte[]> coda = new ArrayBlockingQueue<>(Audio.CODA);
        AtomicLong persi = new AtomicLong();
        OutputStream uscita = new BufferedOutputStream(socket.getOutputStream(), 1 << 16);
        Thread spedizione = avvia("audio-spedizione", () -> spedisci(coda, uscita));
        InputStream ingresso = socket.getInputStream();
        avvia("audio-sentinella", () -> sorveglia(ingresso));

        MediaCodec codificatore = null;
        Audio.Lettura lettura = null;
        Thread lettore = null;
        Thread codifica = null;
        final Throwable[] erroreCodifica = new Throwable[1];
        try {
            Audio.Opzioni opzioni = new Audio.Opzioni(
                    new String[] {"sorgente=loopback", "formato=" + (aac ? "aac" : "pcm")}, 0);
            Audio.Cattura aperta = Audio.apri(Contesto.shell(), opzioni);
            synchronized (this) {
                cattura = aperta;
            }
            BlockingQueue<Audio.Blocco> blocchi = null;
            if (aac) {
                blocchi = new ArrayBlockingQueue<>(Audio.CODA);
                codificatore = Audio.codificatore();
                codificatore.start();
            }
            lettura = new Audio.Lettura(aperta.registratore, opzioni, coda, blocchi, persi);
            Audio.metti(coda, Audio.testo(0, "inizio formato=" + (aac ? "aac" : "pcm")
                    + " frequenza=" + Audio.FREQUENZA
                    + " canali=" + Audio.CANALI
                    + (aac ? " bitrate=" + Audio.BIT_RATE : "")
                    + " sorgente=loopback"
                    + " buffer_ms=" + aperta.registratore.getBufferSizeInFrames() * 1000 / Audio.FREQUENZA
                    + " registrazione=" + aperta.registrazione), persi);
            aperta.registratore.startRecording();
            lettore = avvia("audio-lettura", lettura);
            if (aac) {
                MediaCodec c = codificatore;
                BlockingQueue<Audio.Blocco> b = blocchi;
                Thread l = lettore;
                codifica = avvia("audio-codifica", () -> {
                    try {
                        Audio.codifica(c, b, coda, persi, spedizione, l);
                    } catch (Throwable e) {
                        erroreCodifica[0] = e;
                    } finally {
                        ferma.countDown();
                    }
                });
            }
            // Si resta qui finché il PC tiene aperto il canale e tutto gira.
            while (!ferma.await(500, TimeUnit.MILLISECONDS)) {
                if (!spedizione.isAlive() || !lettore.isAlive() || (codifica != null && !codifica.isAlive())) {
                    break;
                }
            }
            Throwable errore = lettura.errore != null ? lettura.errore : erroreCodifica[0];
            if (errore != null) {
                segnala(coda, persi, spedizione, errore);
            }
        } catch (Throwable e) {
            // Anche gli Error (API nascoste cambiate): il servizio deve restare vivo.
            segnala(coda, persi, spedizione, e);
        } finally {
            if (lettura != null) {
                lettura.attiva = false;
            }
            chiudiCattura();
            unisci(lettore);
            unisci(codifica);
            if (codificatore != null) {
                try {
                    codificatore.stop();
                } catch (RuntimeException e) {
                    // già fermo
                }
                codificatore.release();
            }
            spedizione.interrupt();
            try {
                socket.close();
            } catch (IOException e) {
                // già chiuso
            }
            chiuso.countDown();
        }
    }

    /** Ferma il registratore e toglie la politica, una volta sola (anche dal gancio di chiusura). */
    private synchronized void chiudiCattura() {
        if (cattura != null && !catturaChiusa) {
            catturaChiusa = true;
            cattura.chiudi();
        }
    }

    /** Un errore arriva al PC come testo, se il canale c'è ancora; poi si chiude. */
    private static void segnala(BlockingQueue<byte[]> coda, AtomicLong persi, Thread spedizione, Throwable e) {
        log("canale audio: " + Audio.descrivi(e));
        if (!spedizione.isAlive()) {
            return;
        }
        Audio.metti(coda, Audio.testo(0, "errore " + Audio.descrivi(e)), persi);
        for (int i = 0; i < 10 && !coda.isEmpty(); i++) {
            Sistema.attendi(50);
        }
        Sistema.attendi(100);
    }

    /** Scrive i pacchetti sul canale; finisce quando il PC chiude o il canale si ferma. */
    private void spedisci(BlockingQueue<byte[]> coda, OutputStream uscita) {
        try {
            while (true) {
                uscita.write(coda.take());
                if (coda.isEmpty()) {
                    uscita.flush();
                }
            }
        } catch (IOException | InterruptedException e) {
            // Canale chiuso dal PC, o fermato da noi.
        } finally {
            ferma.countDown();
        }
    }

    /**
     * Il PC non manda niente sul canale audio: la lettura serve solo ad
     * accorgersi che l'ha chiuso (fine del file o errore), anche quando il
     * telefono tace e la spedizione non ha niente da scrivere.
     */
    private void sorveglia(InputStream ingresso) {
        byte[] scarto = new byte[256];
        try {
            while (ingresso.read(scarto) >= 0) {
                // Dati inattesi: si ignorano (riservati a usi futuri).
            }
        } catch (IOException e) {
            // Canale chiuso.
        } finally {
            ferma.countDown();
        }
    }

    /** Un thread di servizio che non porta giù il processo se qualcosa va storto. */
    private static Thread avvia(String nome, Runnable r) {
        Thread t = new Thread(() -> {
            try {
                r.run();
            } catch (Throwable e) {
                log("thread «" + nome + "»: " + Audio.descrivi(e));
            }
        }, nome);
        t.setDaemon(true);
        t.start();
        return t;
    }

    private static void unisci(Thread t) {
        if (t == null) {
            return;
        }
        try {
            t.join(1000);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }

    private static void log(String testo) {
        System.err.println("phonestra-servizio: " + testo);
        System.err.flush();
    }
}
