// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

package phonestra;

import android.net.Credentials;
import android.net.LocalServerSocket;
import android.net.LocalSocket;
import android.os.Build;
import android.os.Process;

import java.io.BufferedInputStream;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.security.SecureRandom;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Il servizio di Phonestra sul telefono: un solo processo per collegamento, di
 * lunga durata (notes/component.md). Lo scheletro: canali, segreto, battito,
 * custode, autotest. Audio, video e input si aggiungeranno come tipi di canale
 * ({@link #TIPI}) e messaggi del canale comandi.
 *
 * <p>Ciclo di vita:
 * <ol>
 *   <li>il PC lo avvia con {@code shell,v2,raw:} e gli manda il segreto
 *       (32 cifre esadecimali e a capo) sull'ingresso: così non compare fra gli
 *       argomenti del processo;
 *   <li>il servizio avvia il {@link Custode}, cancella il proprio jar, prepara il
 *       contesto ed esegue l'{@link Autotest};
 *   <li>apre un {@code LocalServerSocket} astratto dal nome casuale e stampa
 *       sull'uscita la riga di pronto:
 *       {@code phonestra-servizio pronto protocollo=1 socket=phonestra_<32 hex> pid=<pid>};
 *   <li>ogni canale ({@code localabstract:} dal PC) deve venire dall'uid 2000 e
 *       cominciare col preambolo (segreto e tipo, {@link Protocollo#leggiPreambolo}),
 *       altrimenti si chiude subito;
 *   <li>il canale {@code comandi} (uno solo) riceve il {@code CIAO}, poi il
 *       battito nei due sensi ogni secondo;
 *   <li>il servizio esce ({@code System.exit}) se il PC chiede {@code FINE}, se
 *       il canale comandi si chiude, se per 5 s non arriva niente dal PC, o se
 *       nessun PC si fa vivo entro 10 s. Il custode rimette a posto.
 * </ol>
 */
final class Servizio {
    static final String PRONTO = "phonestra-servizio pronto";
    static final String ERRORE_AVVIO = "phonestra-servizio errore";

    // Codici d'uscita (il PC li legge dal canale shell,v2).
    static final int USCITA_FINE = 0;
    static final int USCITA_ERRORE = 1;
    static final int USCITA_BATTITO = 3;
    static final int USCITA_COMANDI_CHIUSO = 4;
    static final int USCITA_NESSUN_PC = 5;

    static final long BATTITO_MS = 1000;
    static final long LIMITE_MS = 5000;
    static final long ATTESA_PC_MS = 10000;

    /** File della prova del custode: lo crea il servizio, lo toglie solo il custode. */
    static final String FILE_PROVA_CUSTODE = "/data/local/tmp/phonestra-prova-custode";

    /** Uid della shell: adbd gira con questo uid, e ogni canale ADB arriva da lui. */
    private static final int UID_SHELL = 2000;

    /** Un tipo di canale: riceve il socket già controllato e il tipo completo (per esempio «video:3»). */
    interface Gestore {
        void gestisci(LocalSocket socket, String tipo) throws Exception;
    }

    /** Tipi di canale per la parte prima dei due punti; i pezzi futuri si aggiungono qui. */
    private static final Map<String, Gestore> TIPI = new HashMap<>();

    static {
        TIPI.put("comandi", Servizio::comandi);
        TIPI.put("audio", CanaleAudio::gestisci);
        TIPI.put("video", Video::canale);
    }

    private static final long INIZIO = System.nanoTime();
    private static volatile byte[] segreto;
    private static volatile long pronto;
    private static volatile long ultimoMessaggio;
    private static volatile boolean comandiAperto;
    private static final AtomicBoolean TERMINATO = new AtomicBoolean();
    private static final Object SCRITTURA = new Object();
    private static OutputStream uscitaComandi;
    private static Custode custode;
    private static byte[] ciao;

    private Servizio() {
    }

    static void main(String[] argomenti) {
        Thread.setDefaultUncaughtExceptionHandler((t, e) -> {
            log("eccezione nel thread «" + t.getName() + "»: " + Nascoste.causa(e));
            termina(USCITA_ERRORE, "eccezione non gestita");
        });
        avviaGuardiano();
        try {
            avvia();
        } catch (Throwable e) {
            System.out.println(ERRORE_AVVIO + " " + Nascoste.causa(e));
            System.out.flush();
            termina(USCITA_ERRORE, "avvio non riuscito: " + Nascoste.causa(e));
        }
    }

    private static void avvia() throws Exception {
        segreto = leggiSegreto(System.in);
        String jar = System.getenv("CLASSPATH");
        // Una copia del jar resta al custode: gli serve per riaccendere il
        // pannello in Java (senza bloccare il telefono); la cancella lui alla fine.
        jarCustode = copiaPerCustode(jar);
        custode = Custode.avvia(jar, jarCustode);
        cancellaJar(jar);
        long inizioTest = System.nanoTime();
        Map<String, String> esiti = Autotest.esegui();
        esiti.put("custode", custode.stato());
        long durataTest = (System.nanoTime() - inizioTest) / 1_000_000;
        String nome = "phonestra_" + esadecimale(casuali(16));
        LocalServerSocket server = new LocalServerSocket(nome);
        ciao = componiCiao(esiti, durataTest).getBytes(StandardCharsets.UTF_8);
        pronto = System.nanoTime();
        System.out.println(PRONTO + " protocollo=" + Protocollo.VERSIONE + " socket=" + nome + " pid=" + Process.myPid());
        System.out.flush();
        while (true) {
            LocalSocket s = server.accept();
            Thread t = new Thread(() -> canale(s), "canale");
            t.setDaemon(true);
            t.start();
        }
    }

    /** Il segreto: una riga di 32 cifre esadecimali sull'ingresso del processo. */
    static byte[] leggiSegreto(InputStream in) throws IOException {
        ByteArrayOutputStream riga = new ByteArrayOutputStream();
        int b;
        while ((b = in.read()) >= 0 && b != '\n') {
            if (riga.size() > 128) {
                throw new IOException("segreto troppo lungo");
            }
            if (b != '\r') {
                riga.write(b);
            }
        }
        String testo = new String(riga.toByteArray(), StandardCharsets.US_ASCII).trim();
        if (!testo.matches("[0-9a-f]{" + 2 * Protocollo.LUNGHEZZA_SEGRETO + "}")) {
            throw new IOException("segreto non valido o assente");
        }
        byte[] s = new byte[Protocollo.LUNGHEZZA_SEGRETO];
        for (int i = 0; i < s.length; i++) {
            s[i] = (byte) Integer.parseInt(testo.substring(2 * i, 2 * i + 2), 16);
        }
        return s;
    }

    /**
     * Il jar non serve più una volta partiti (il dex è già aperto): si cancella
     * subito, così non resta anche se il processo viene ucciso. Solo se ha il
     * nome che gli dà il PC; il custode ci riprova alla fine.
     */
    /** Copia del jar per il custode, o {@code null} se non riesce. */
    static volatile String jarCustode;

    private static String copiaPerCustode(String jar) {
        if (jar == null) {
            return null;
        }
        String copia = "/data/local/tmp/phonestra-custode-" + Process.myPid() + ".jar";
        try {
            java.nio.file.Files.copy(new File(jar).toPath(), new File(copia).toPath(),
                    java.nio.file.StandardCopyOption.REPLACE_EXISTING);
            return copia;
        } catch (Exception e) {
            log("copia del jar per il custode non riuscita: " + Nascoste.causa(e));
            return null;
        }
    }

    private static void cancellaJar(String jar) {
        if (jar == null || !jar.startsWith("/data/local/tmp/phonestra-servizio-") || !jar.endsWith(".jar")) {
            log("jar dal nome inatteso, non lo cancello: " + jar);
            return;
        }
        if (!new File(jar).delete()) {
            log("jar non cancellato: " + jar);
        }
    }

    private static String componiCiao(Map<String, String> esiti, long durataTest) {
        StringBuilder s = new StringBuilder();
        voce(s, "protocollo", String.valueOf(Protocollo.VERSIONE));
        voce(s, "android", Build.VERSION.RELEASE);
        voce(s, "sdk", String.valueOf(Build.VERSION.SDK_INT));
        voce(s, "produttore", Build.MANUFACTURER);
        voce(s, "modello", Build.MODEL);
        voce(s, "pid", String.valueOf(Process.myPid()));
        voce(s, "avvio_ms", String.valueOf((System.nanoTime() - INIZIO) / 1_000_000));
        voce(s, "autotest_ms", String.valueOf(durataTest));
        for (Map.Entry<String, String> e : esiti.entrySet()) {
            voce(s, "autotest." + e.getKey(), e.getValue());
        }
        return s.toString();
    }

    private static void voce(StringBuilder s, String chiave, String valore) {
        s.append(chiave).append('=').append(String.valueOf(valore).replace('\n', ' ').replace('\r', ' ')).append('\n');
    }

    /** Un canale appena accettato: chi è, segreto, tipo; poi al suo gestore. */
    private static void canale(LocalSocket s) {
        try {
            Credentials c = s.getPeerCredentials();
            if (c.getUid() != UID_SHELL) {
                log("canale rifiutato: uid " + c.getUid() + " (pid " + c.getPid() + ")");
                s.close();
                return;
            }
            s.setSoTimeout(3000);
            String tipo = Protocollo.leggiPreambolo(s.getInputStream(), segreto);
            if (tipo == null) {
                log("canale rifiutato: segreto sbagliato");
                s.close();
                return;
            }
            s.setSoTimeout(0);
            Gestore g = TIPI.get(tipo.split(":", 2)[0]);
            if (g == null) {
                log("canale rifiutato: tipo sconosciuto «" + tipo + "»");
                s.close();
                return;
            }
            g.gestisci(s, tipo);
        } catch (Exception e) {
            log("canale chiuso: " + Nascoste.causa(e));
            try {
                s.close();
            } catch (IOException ignorato) {
                // già chiuso
            }
        }
    }

    /** Il canale comandi: CIAO, battito, e i messaggi del PC finché resta aperto. */
    private static void comandi(LocalSocket s, String tipo) throws Exception {
        synchronized (Servizio.class) {
            if (comandiAperto) {
                log("secondo canale comandi rifiutato");
                s.close();
                return;
            }
            uscitaComandi = s.getOutputStream();
            ultimoMessaggio = System.nanoTime();
            comandiAperto = true;
        }
        manda(Protocollo.CIAO, 0, 0, ciao);
        Thread battito = new Thread(() -> {
            while (true) {
                Sistema.attendi(BATTITO_MS);
                manda(Protocollo.BATTITO, 0, 0, new byte[0]);
            }
        }, "battito");
        battito.setDaemon(true);
        battito.start();
        DataInputStream in = new DataInputStream(new BufferedInputStream(s.getInputStream()));
        while (true) {
            Protocollo.Messaggio m;
            try {
                m = Protocollo.leggi(in);
            } catch (IOException e) {
                termina(USCITA_COMANDI_CHIUSO, "canale comandi chiuso dal PC (" + Nascoste.causa(e) + ")");
                return;
            }
            ultimoMessaggio = System.nanoTime();
            if (m.risposta()) {
                continue;
            }
            switch (m.tipo) {
                case Protocollo.BATTITO:
                    break;
                case Protocollo.FINE:
                    manda(Protocollo.FINE, Protocollo.RISPOSTA, m.id, new byte[0]);
                    termina(USCITA_FINE, "fine chiesta dal PC");
                    return;
                case Protocollo.PROVA_CUSTODE:
                    provaCustode(m);
                    break;
                case Video.APRI:
                case Video.CHIUDI:
                case Video.AVVIA_APP:
                case Video.RIDIMENSIONA:
                case Video.CHIAVE:
                case Video.PANNELLO:
                    Video.comando(m);
                    break;
                default:
                    if (Input.nostro(m.tipo)) {
                        Input.ricevi(m);
                        break;
                    }
                    manda(Protocollo.ERRORE, Protocollo.RISPOSTA, m.id,
                            String.format("tipo sconosciuto 0x%02x", m.tipo).getBytes(StandardCharsets.UTF_8));
            }
        }
    }

    /**
     * Prova innocua del custode: crea un file in /data/local/tmp e affida al
     * custode il compito di toglierlo. Se dopo la fine del servizio il file non
     * c'è più, il custode ha fatto il suo lavoro.
     */
    private static void provaCustode(Protocollo.Messaggio m) {
        try {
            try (FileOutputStream o = new FileOutputStream(FILE_PROVA_CUSTODE)) {
                o.write("prova del custode di Phonestra\n".getBytes(StandardCharsets.UTF_8));
            }
            custode.imposta("prova-custode", 900, "rm -f " + FILE_PROVA_CUSTODE);
            manda(Protocollo.PROVA_CUSTODE, Protocollo.RISPOSTA, m.id, FILE_PROVA_CUSTODE.getBytes(StandardCharsets.UTF_8));
        } catch (Exception e) {
            manda(Protocollo.ERRORE, Protocollo.RISPOSTA, m.id, Nascoste.causa(e).getBytes(StandardCharsets.UTF_8));
        }
    }

    /** Il custode, per i pezzi che gli affidano azioni di ripristino ({@code null} prima dell'avvio). */
    static Custode custode() {
        return custode;
    }

    /** Manda un messaggio sul canale comandi; se non si può, il PC non c'è più. */
    static void manda(int tipo, int bandiere, int id, byte[] dati) {
        try {
            synchronized (SCRITTURA) {
                Protocollo.scrivi(uscitaComandi, tipo, bandiere, id, dati);
            }
        } catch (IOException e) {
            termina(USCITA_COMANDI_CHIUSO, "scrittura sul canale comandi non riuscita (" + Nascoste.causa(e) + ")");
        }
    }

    /**
     * Guardiano: senza segreto o senza canale comandi entro 10 s, o senza
     * messaggi dal PC per 5 s (adbd sul Wi-Fi non si accorge del PC sparito,
     * study/system.md §1.5), il servizio esce.
     */
    private static void avviaGuardiano() {
        Thread t = new Thread(() -> {
            while (true) {
                Sistema.attendi(200);
                long adesso = System.nanoTime();
                if (segreto == null && adesso - INIZIO > ATTESA_PC_MS * 1_000_000) {
                    termina(USCITA_NESSUN_PC, "segreto non arrivato");
                } else if (!comandiAperto && pronto != 0 && adesso - pronto > ATTESA_PC_MS * 1_000_000) {
                    termina(USCITA_NESSUN_PC, "nessun canale comandi");
                } else if (comandiAperto && adesso - ultimoMessaggio > LIMITE_MS * 1_000_000) {
                    termina(USCITA_BATTITO, "nessun messaggio dal PC da " + LIMITE_MS / 1000 + " s");
                }
            }
        }, "guardiano");
        t.setDaemon(true);
        t.start();
    }

    /**
     * Fine del processo. Il ripristino lo fa il custode, che si accorge della
     * morte del servizio: qui non si chiude niente a mano, così la strada è la
     * stessa per la fine normale e per quella improvvisa. Se {@code System.exit}
     * si bloccasse, dopo 2 s si termina di forza.
     */
    static void termina(int codice, String motivo) {
        if (!TERMINATO.compareAndSet(false, true)) {
            return;
        }
        log("fine (" + codice + "): " + motivo);
        Thread forza = new Thread(() -> {
            Sistema.attendi(2000);
            Runtime.getRuntime().halt(codice);
        }, "fine forzata");
        forza.setDaemon(true);
        forza.start();
        System.exit(codice);
    }

    private static void log(String testo) {
        System.err.println("phonestra-servizio: " + testo);
        System.err.flush();
    }

    private static byte[] casuali(int n) {
        byte[] b = new byte[n];
        new SecureRandom().nextBytes(b);
        return b;
    }

    private static String esadecimale(byte[] b) {
        StringBuilder s = new StringBuilder();
        for (byte x : b) {
            s.append(String.format("%02x", x & 0xff));
        }
        return s.toString();
    }
}
