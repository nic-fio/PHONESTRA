package phonestra;

import android.content.Context;
import android.media.AudioAttributes;
import android.media.AudioFormat;
import android.media.AudioRecord;
import android.media.AudioTimestamp;
import android.media.MediaCodec;
import android.media.MediaFormat;
import android.media.MediaRecorder;
import android.os.Process;

import java.io.BufferedOutputStream;
import java.io.DataOutputStream;
import java.io.FileDescriptor;
import java.io.FileOutputStream;
import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.Locale;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;

/**
 * Audio del telefono per Phonestra, per ora come strumento di misura (fase 0:
 * notes/study/audio.md, prove A2, A4, A5, A6; notes/api-android.md §1).
 * Cattura l'audio del telefono da una delle tre sorgenti e lo scrive
 * sull'uscita del processo finché il PC la legge.
 *
 * <p>Argomenti {@code chiave=valore}, tutti facoltativi:
 * <ul>
 * <li>{@code sorgente=submix} ({@code AudioRecord} con {@code REMOTE_SUBMIX},
 * predefinita), {@code loopback} ({@code AudioPolicy} con
 * {@code ROUTE_FLAG_LOOP_BACK}: il telefono tace) o {@code render}
 * ({@code ROUTE_FLAG_LOOP_BACK_RENDER}: il telefono continua a suonare);
 * <li>{@code formato=pcm} (campioni così come letti, predefinito) o {@code aac}
 * (AAC-LC 192 kbit/s);
 * <li>{@code priorita=si|no}: thread di lettura a {@code THREAD_PRIORITY_URGENT_AUDIO}
 * (predefinito sì; «no» per la prova A6 senza priorità);
 * <li>{@code voce=si|no}: con AudioPolicy cattura anche le chiamate VoIP
 * (predefinito no, prova A7).
 * </ul>
 *
 * <p>Ogni pacchetto: 8 byte di orario in µs (bit 62 = configurazione del
 * codec, bit 61 = misura in testo UTF-8), 4 byte di lunghezza, i dati. Gli
 * orari vengono dal numero di campioni letti, quindi sono regolari. Lettura,
 * codifica e spedizione hanno ciascuna un thread: se il Wi-Fi o il
 * codificatore rallentano, la lettura dell'audio non si ferma.
 *
 * <p>Le misure (una riga al secondo, {@code misura chiave=valore …}) sono
 * cumulative dall'inizio, tranne {@code lettura_max_ms} (ultimo secondo).
 *
 * <p>Cattura, lettura e codifica si usano anche dal canale «audio» del
 * servizio ({@link CanaleAudio}): stesse classi, provate con questo strumento.
 */
final class Audio {
    static final int FREQUENZA = 48000;
    static final int CANALI = 2;
    /** Campioni per canale in un blocco letto: quelli di un frame AAC e del tubo del submix. */
    private static final int BLOCCO = 1024;
    private static final int BYTE_PER_CAMPIONE = 2 * CANALI;
    static final int BIT_RATE = 192000;
    private static final long CONFIGURAZIONE = 1L << 62;
    private static final long MISURA = 1L << 61;
    /** Pacchetti in attesa: circa 5 s di audio. */
    static final int CODA = 256;
    /** Sequenza minima di zeri esatti da contare: 1 ms. */
    private static final int ZERI_MINIMI = 48;
    /** −40 dBFS in energia (campione² medio, fondo scala 32768). */
    private static final double SOGLIA_SUONO = 32768.0 * 32768.0 * 1e-4;
    /** Costante di tempo del livello prima degli zeri: 10 ms. */
    private static final double MEDIA_LIVELLO = 480;

    /**
     * Usi catturati con AudioPolicy: quelli che il motore di AOSP manda al
     * remote submix anche con REMOTE_SUBMIX (strategie MEDIA e ACCESSIBILITY),
     * così le sorgenti si confrontano sugli stessi suoni. Suonerie, sveglie,
     * notifiche e chiamate restano sul telefono.
     */
    private static final int[] USI = {
        AudioAttributes.USAGE_UNKNOWN,
        AudioAttributes.USAGE_MEDIA,
        AudioAttributes.USAGE_GAME,
        AudioAttributes.USAGE_ASSISTANT,
        AudioAttributes.USAGE_ASSISTANCE_ACCESSIBILITY,
        AudioAttributes.USAGE_ASSISTANCE_NAVIGATION_GUIDANCE,
        AudioAttributes.USAGE_ASSISTANCE_SONIFICATION,
    };

    private Audio() {
    }

    /** Le scelte passate dal PC. */
    static final class Opzioni {
        String sorgente = "submix";
        boolean aac;
        boolean priorita = true;
        boolean voce;

        Opzioni(String[] argomenti, int primo) {
            for (int i = primo; i < argomenti.length; i++) {
                String a = argomenti[i];
                int uguale = a.indexOf('=');
                if (uguale < 0) {
                    throw new IllegalArgumentException("argomento senza «=»: " + a);
                }
                String chiave = a.substring(0, uguale);
                String valore = a.substring(uguale + 1);
                switch (chiave) {
                    case "sorgente":
                        if (!valore.equals("submix") && !valore.equals("loopback") && !valore.equals("render")) {
                            throw new IllegalArgumentException("sorgente sconosciuta: " + valore);
                        }
                        sorgente = valore;
                        break;
                    case "formato":
                        if (!valore.equals("pcm") && !valore.equals("aac")) {
                            throw new IllegalArgumentException("formato sconosciuto: " + valore);
                        }
                        aac = valore.equals("aac");
                        break;
                    case "priorita":
                        priorita = valore.equals("si");
                        break;
                    case "voce":
                        voce = valore.equals("si");
                        break;
                    default:
                        throw new IllegalArgumentException("argomento sconosciuto: " + a);
                }
            }
        }
    }

    /** Il registratore e, con AudioPolicy, la politica da togliere alla fine. */
    static final class Cattura {
        final AudioRecord registratore;
        final Object gestore;
        final Object politica;
        final String registrazione;

        Cattura(AudioRecord registratore, Object gestore, Object politica, String registrazione) {
            this.registratore = registratore;
            this.gestore = gestore;
            this.politica = politica;
            this.registrazione = registrazione;
        }

        void chiudi() {
            try {
                registratore.stop();
            } catch (RuntimeException e) {
                // già fermo
            }
            registratore.release();
            if (politica != null) {
                deregistra(gestore, politica);
            }
        }
    }

    /** Un blocco PCM per il codificatore AAC. */
    static final class Blocco {
        final byte[] dati;
        final long orario;

        Blocco(byte[] dati, long orario) {
            this.dati = dati;
            this.orario = orario;
        }
    }

    /**
     * {@code argomenti[primo…]}: le opzioni. Gli errori arrivano al PC anche
     * come riga {@code errore …}, perché l'uscita d'errore non gli arriva.
     */
    static void cattura(Context shell, String[] argomenti, int primo) throws Exception {
        BlockingQueue<byte[]> coda = new ArrayBlockingQueue<>(CODA);
        AtomicLong persi = new AtomicLong();
        Thread spedizione = new Thread(() -> spedisci(coda), "spedizione");
        spedizione.setDaemon(true);
        spedizione.start();

        Cattura cattura = null;
        MediaCodec codificatore = null;
        Lettura lettura = null;
        try {
            Opzioni opzioni = new Opzioni(argomenti, primo);
            cattura = apri(shell, opzioni);
            BlockingQueue<Blocco> blocchi = null;
            if (opzioni.aac) {
                blocchi = new ArrayBlockingQueue<>(CODA);
                codificatore = codificatore();
                codificatore.start();
            }
            lettura = new Lettura(cattura.registratore, opzioni, coda, blocchi, persi);
            Thread lettore = new Thread(lettura, "lettura");
            lettore.setDaemon(true);
            metti(coda, testo(0, "inizio sorgente=" + opzioni.sorgente
                    + " formato=" + (opzioni.aac ? "aac" : "pcm")
                    + " priorita=" + (opzioni.priorita ? "si" : "no")
                    + " voce=" + (opzioni.voce ? "si" : "no")
                    + " buffer_ms=" + cattura.registratore.getBufferSizeInFrames() * 1000 / FREQUENZA
                    + " registrazione=" + cattura.registrazione), persi);
            cattura.registratore.startRecording();
            lettore.start();
            if (codificatore != null) {
                codifica(codificatore, blocchi, coda, persi, spedizione, lettore);
            }
            while (spedizione.isAlive() && lettore.isAlive()) {
                lettore.join(500);
            }
            if (lettura.errore != null) {
                throw lettura.errore;
            }
        } catch (Exception e) {
            if (spedizione.isAlive()) {
                metti(coda, testo(0, "errore " + descrivi(e)), persi);
                attendiSpedizione(coda);
            }
            throw e;
        } finally {
            if (lettura != null) {
                lettura.attiva = false;
            }
            if (cattura != null) {
                cattura.chiudi();
            }
            if (codificatore != null) {
                codificatore.stop();
                codificatore.release();
            }
        }
    }

    static Cattura apri(Context shell, Opzioni opzioni) throws Exception {
        if (opzioni.sorgente.equals("submix")) {
            return new Cattura(registratoreSubmix(shell), null, null, "-");
        }
        return conPolitica(shell, opzioni);
    }

    private static AudioFormat formato() {
        return new AudioFormat.Builder()
                .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                .setSampleRate(FREQUENZA)
                .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
                .build();
    }

    /** L'uscita intera del telefono, dopo il mixer. */
    private static AudioRecord registratoreSubmix(Context shell) throws Exception {
        int minimo = AudioRecord.getMinBufferSize(FREQUENZA, AudioFormat.CHANNEL_IN_STEREO, AudioFormat.ENCODING_PCM_16BIT);
        // Mezzo secondo di riserva: assorbe le pause del thread di lettura.
        int riserva = Math.max(minimo * 4, FREQUENZA / 2 * BYTE_PER_CAMPIONE);
        AudioRecord.Builder b = new AudioRecord.Builder()
                .setAudioSource(MediaRecorder.AudioSource.REMOTE_SUBMIX)
                .setAudioFormat(formato())
                .setBufferSizeInBytes(riserva);
        // Il permesso CAPTURE_AUDIO_OUTPUT è della shell: il contesto deve essere il suo.
        AudioRecord.Builder.class.getMethod("setContext", Context.class).invoke(b, shell);
        return inizializzato(b.build());
    }

    private static AudioRecord inizializzato(AudioRecord r) throws IOException {
        if (r.getState() != AudioRecord.STATE_INITIALIZED) {
            r.release();
            throw new IOException("cattura dell'audio non disponibile");
        }
        return r;
    }

    /**
     * Cattura dei lettori con una {@code AudioPolicy} (API nascoste, per
     * riflessione; serve {@code MODIFY_AUDIO_ROUTING}, che la shell ha): una
     * regola «ruolo lettori, questi usi», un {@code AudioMix} in loop-back
     * (con o senza l'altoparlante), la politica registrata presso
     * l'{@code AudioManager} e il registratore che legge il mix.
     */
    private static Cattura conPolitica(Context shell, Opzioni opzioni) throws Exception {
        Class<?> regola = Class.forName("android.media.audiopolicy.AudioMixingRule");
        Class<?> regolaB = Class.forName("android.media.audiopolicy.AudioMixingRule$Builder");
        Object rb = regolaB.getConstructor().newInstance();
        // Il ruolo va dato prima delle regole: ne decide la validità.
        regolaB.getMethod("setTargetMixRole", int.class).invoke(rb, regola.getField("MIX_ROLE_PLAYERS").getInt(null));
        int perUso = regola.getField("RULE_MATCH_ATTRIBUTE_USAGE").getInt(null);
        Method aggiungi = regolaB.getMethod("addMixRule", int.class, Object.class);
        for (int uso : USI) {
            aggiungi.invoke(rb, perUso, new AudioAttributes.Builder().setUsage(uso).build());
        }
        if (opzioni.voce) {
            aggiungi.invoke(rb, perUso, new AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_VOICE_COMMUNICATION).build());
            // Prima di build(): chiamato dopo non ha effetto.
            regolaB.getMethod("voiceCommunicationCaptureAllowed", boolean.class).invoke(rb, true);
        }
        Object laRegola = regolaB.getMethod("build").invoke(rb);

        Class<?> mix = Class.forName("android.media.audiopolicy.AudioMix");
        Class<?> mixB = Class.forName("android.media.audiopolicy.AudioMix$Builder");
        Object mb = mixB.getConstructor(regola).newInstance(laRegola);
        mixB.getMethod("setFormat", AudioFormat.class).invoke(mb, formato());
        String instradamento = opzioni.sorgente.equals("render") ? "ROUTE_FLAG_LOOP_BACK_RENDER" : "ROUTE_FLAG_LOOP_BACK";
        mixB.getMethod("setRouteFlags", int.class).invoke(mb, mix.getField(instradamento).getInt(null));
        Object ilMix = mixB.getMethod("build").invoke(mb);

        Class<?> politicaC = Class.forName("android.media.audiopolicy.AudioPolicy");
        Class<?> politicaB = Class.forName("android.media.audiopolicy.AudioPolicy$Builder");
        Object pb = politicaB.getConstructor(Context.class).newInstance(shell);
        politicaB.getMethod("addMix", mix).invoke(pb, ilMix);
        Object politica = politicaB.getMethod("build").invoke(pb);

        Object gestore = Context.class.getMethod("getSystemService", String.class).invoke(shell, "audio");
        String registrazione = registra(gestore, politicaC, politica);
        try {
            AudioRecord r = (AudioRecord) politicaC.getMethod("createAudioRecordSink", mix).invoke(politica, ilMix);
            if (r == null) {
                throw new IOException("createAudioRecordSink ha restituito null");
            }
            return new Cattura(inizializzato(r), gestore, politica, registrazione);
        } catch (Exception e) {
            deregistra(gestore, politica);
            throw e;
        }
    }

    /**
     * Registra la politica: prima col metodo pubblico di sistema
     * {@code AudioManager.registerAudioPolicy}, poi con quello statico interno
     * {@code registerAudioPolicyStatic}. Restituisce quale ha funzionato.
     */
    private static String registra(Object gestore, Class<?> politicaC, Object politica) throws Exception {
        Class<?> audioManager = Class.forName("android.media.AudioManager");
        String primo;
        try {
            int esito = (Integer) audioManager.getMethod("registerAudioPolicy", politicaC).invoke(gestore, politica);
            if (esito == 0) {
                return "istanza";
            }
            primo = "registerAudioPolicy: " + esito;
        } catch (Exception e) {
            primo = "registerAudioPolicy: " + descrivi(e);
        }
        Method statico = audioManager.getDeclaredMethod("registerAudioPolicyStatic", politicaC);
        statico.setAccessible(true);
        int esito = (Integer) statico.invoke(null, politica);
        if (esito != 0) {
            throw new IOException(primo + "; registerAudioPolicyStatic: " + esito);
        }
        return "statica";
    }

    private static void deregistra(Object gestore, Object politica) {
        Class<?> politicaC = politica.getClass();
        try {
            Class.forName("android.media.AudioManager").getMethod("unregisterAudioPolicy", politicaC).invoke(gestore, politica);
            return;
        } catch (Exception e) {
            // si prova il metodo statico
        }
        try {
            Method statico = Class.forName("android.media.AudioManager").getDeclaredMethod("unregisterAudioPolicyAsyncStatic", politicaC);
            statico.setAccessible(true);
            statico.invoke(null, politica);
        } catch (Exception e) {
            // Alla fine del processo il sistema la toglie comunque (il binder muore).
        }
    }

    static MediaCodec codificatore() throws IOException {
        MediaFormat f = MediaFormat.createAudioFormat("audio/mp4a-latm", FREQUENZA, CANALI);
        f.setInteger("bitrate", BIT_RATE);
        f.setInteger("aac-profile", 2); // AAC-LC
        f.setInteger("max-input-size", BLOCCO * BYTE_PER_CAMPIONE);
        MediaCodec c = MediaCodec.createEncoderByType("audio/mp4a-latm");
        c.configure(f, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
        return c;
    }

    /** Il thread di lettura: legge, misura e passa i blocchi alla spedizione o al codificatore. */
    static final class Lettura implements Runnable {
        private final AudioRecord registratore;
        private final Opzioni opzioni;
        private final BlockingQueue<byte[]> coda;
        private final BlockingQueue<Blocco> blocchi;
        private final AtomicLong persi;
        volatile boolean attiva = true;
        volatile Exception errore;

        private long campioni;
        private long letture;
        private long brevi;
        private long letturaMassima;
        /** Zeri esatti: sequenza in corso, sequenze contate, loro campioni, la più lunga. */
        private long zeriInCorso;
        private long zeri;
        private long campioniZero;
        private long zeroMassimo;
        /** Energia media recente (campione²) e quella all'inizio della sequenza di zeri. */
        private double energia;
        private double energiaPrima;
        /** Primo {@code AudioTimestamp} valido, riferimento della deriva. */
        private long posizioneBase = -1;
        private long nanoBase;

        Lettura(AudioRecord registratore, Opzioni opzioni, BlockingQueue<byte[]> coda,
                BlockingQueue<Blocco> blocchi, AtomicLong persi) {
            this.registratore = registratore;
            this.opzioni = opzioni;
            this.coda = coda;
            this.blocchi = blocchi;
            this.persi = persi;
        }

        @Override
        public void run() {
            try {
                if (opzioni.priorita) {
                    try {
                        Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO);
                    } catch (RuntimeException e) {
                        metti(coda, testo(0, "avviso priorità non concessa: " + e), persi);
                    }
                }
                int tid = Process.myTid();
                metti(coda, testo(0, "lettura tid=" + tid + " nice=" + nice(tid)), persi);
                leggi(tid);
            } catch (Exception e) {
                if (attiva) {
                    errore = e;
                }
            }
        }

        private void leggi(int tid) throws Exception {
            int lunghezza = BLOCCO * BYTE_PER_CAMPIONE;
            // In PCM i campioni si leggono direttamente dopo l'intestazione del pacchetto.
            int inizio = blocchi == null ? 12 : 0;
            long prossimaMisura = FREQUENZA;
            while (attiva) {
                byte[] dati = new byte[inizio + lunghezza];
                int letti = leggiTutto(dati, inizio, lunghezza);
                if (letti <= 0) {
                    if (!attiva) {
                        return;
                    }
                    throw new IOException("lettura dell'audio non riuscita: " + letti);
                }
                if (letti < lunghezza) {
                    dati = Arrays.copyOf(dati, inizio + letti);
                }
                analizza(dati, inizio, letti);
                long orario = campioni * 1_000_000L / FREQUENZA;
                if (blocchi == null) {
                    ByteBuffer.wrap(dati).putLong(orario).putInt(letti);
                    metti(coda, dati, persi);
                } else {
                    metti(blocchi, new Blocco(dati, orario), persi);
                }
                campioni += letti / BYTE_PER_CAMPIONE;
                if (campioni >= prossimaMisura) {
                    metti(coda, testo(orario, misura(tid)), persi);
                    prossimaMisura += FREQUENZA;
                }
            }
        }

        /** Legge un blocco intero (read può restituire meno del richiesto). */
        private int leggiTutto(byte[] dati, int inizio, int quanti) {
            int letti = 0;
            while (letti < quanti) {
                long prima = System.nanoTime();
                int n = registratore.read(dati, inizio + letti, quanti - letti);
                letturaMassima = Math.max(letturaMassima, System.nanoTime() - prima);
                letture++;
                if (n <= 0) {
                    return letti > 0 ? letti : n;
                }
                if (n < quanti - letti) {
                    brevi++;
                }
                letti += n;
            }
            return letti;
        }

        /**
         * Conta le sequenze di zeri esatti su entrambi i canali lunghe almeno
         * 1 ms, finite dal ritorno del suono e iniziate dopo un livello sopra
         * −40 dBFS (media mobile di ~10 ms): gli zeri che il remote submix mette
         * quando mancano dati, non i silenzi dei contenuti.
         */
        private void analizza(byte[] dati, int inizio, int quanti) {
            for (int i = inizio; i + 3 < inizio + quanti; i += BYTE_PER_CAMPIONE) {
                int sinistro = (short) ((dati[i] & 0xff) | (dati[i + 1] << 8));
                int destro = (short) ((dati[i + 2] & 0xff) | (dati[i + 3] << 8));
                if (sinistro == 0 && destro == 0) {
                    if (zeriInCorso == 0) {
                        energiaPrima = energia;
                    }
                    zeriInCorso++;
                    continue;
                }
                if (zeriInCorso >= ZERI_MINIMI && energiaPrima > SOGLIA_SUONO) {
                    zeri++;
                    campioniZero += zeriInCorso;
                    zeroMassimo = Math.max(zeroMassimo, zeriInCorso);
                }
                zeriInCorso = 0;
                double e = (sinistro * (double) sinistro + destro * (double) destro) / 2;
                energia += (e - energia) / MEDIA_LIVELLO;
            }
        }

        private String misura(int tid) {
            StringBuilder s = new StringBuilder("misura");
            s.append(" t=").append(campioni / FREQUENZA)
                    .append(" campioni=").append(campioni)
                    .append(" letture=").append(letture)
                    .append(" brevi=").append(brevi)
                    .append(" lettura_max_ms=").append(ms(letturaMassima / 1e6))
                    .append(" zeri=").append(zeri)
                    .append(" zeri_ms=").append(ms(campioniZero * 1000.0 / FREQUENZA))
                    .append(" zeri_max_ms=").append(ms(zeroMassimo * 1000.0 / FREQUENZA))
                    .append(" persi=").append(persi.get());
            letturaMassima = 0;
            AudioTimestamp ts = new AudioTimestamp();
            if (registratore.getTimestamp(ts, AudioTimestamp.TIMEBASE_MONOTONIC) == AudioRecord.SUCCESS) {
                if (posizioneBase < 0) {
                    posizioneBase = ts.framePosition;
                    nanoBase = ts.nanoTime;
                }
                // Deriva: tempo secondo i campioni catturati meno tempo dell'orologio
                // monotono, dal primo orario. Attesa: campioni catturati (stimati
                // adesso) ma non ancora letti da noi; se cresce senza fermarsi, o
                // supera il buffer, si perdono dati.
                double deriva = ((ts.framePosition - posizioneBase) / (double) FREQUENZA
                        - (ts.nanoTime - nanoBase) / 1e9) * 1000;
                double posizioneOra = ts.framePosition + (System.nanoTime() - ts.nanoTime) * (FREQUENZA / 1e9);
                s.append(" posizione=").append(ts.framePosition)
                        .append(" deriva_ms=").append(ms(deriva))
                        .append(" attesa_ms=").append(ms((posizioneOra - campioni) * 1000 / FREQUENZA));
            } else {
                s.append(" orario=assente");
            }
            s.append(" nice=").append(nice(tid));
            return s.toString();
        }
    }

    private static String ms(double valore) {
        // Locale.ROOT: sul telefono in italiano la virgola spezzerebbe i numeri.
        return String.format(Locale.ROOT, "%.2f", valore);
    }

    /**
     * Il «nice» effettivo del thread, dal campo 19 di
     * {@code /proc/self/task/<tid>/stat} (dopo il nome tra parentesi).
     */
    private static String nice(int tid) {
        try {
            String stat = new String(Files.readAllBytes(Paths.get("/proc/self/task/" + tid + "/stat")), StandardCharsets.UTF_8);
            String[] campi = stat.substring(stat.lastIndexOf(')') + 2).trim().split(" ");
            return campi[16];
        } catch (Exception e) {
            return "?";
        }
    }

    /** Pacchetto di testo (misura, avviso o errore) per il PC. */
    static byte[] testo(long orario, String riga) {
        byte[] t = riga.getBytes(StandardCharsets.UTF_8);
        byte[] pacchetto = new byte[12 + t.length];
        ByteBuffer.wrap(pacchetto).putLong(MISURA | orario).putInt(t.length).put(t);
        return pacchetto;
    }

    /** Mette in coda senza mai bloccare: se è piena si perde l'elemento più vecchio. */
    static <T> void metti(BlockingQueue<T> coda, T elemento, AtomicLong persi) {
        while (!coda.offer(elemento)) {
            if (coda.poll() != null) {
                persi.incrementAndGet();
            }
        }
    }

    static String descrivi(Throwable e) {
        while (e instanceof InvocationTargetException && e.getCause() != null) {
            e = e.getCause();
        }
        String s = e.toString();
        if (e.getCause() != null && e.getCause() != e) {
            s += " (causa: " + e.getCause() + ")";
        }
        return s.replace('\n', ' ');
    }

    /** Il codificatore AAC, sul thread principale: blocchi dalla lettura, pacchetti alla spedizione. */
    static void codifica(MediaCodec c, BlockingQueue<Blocco> blocchi, BlockingQueue<byte[]> coda,
            AtomicLong persi, Thread spedizione, Thread lettore) throws Exception {
        MediaCodec.BufferInfo info = new MediaCodec.BufferInfo();
        while (spedizione.isAlive() && lettore.isAlive()) {
            Blocco b = blocchi.poll(500, TimeUnit.MILLISECONDS);
            if (b == null) {
                continue;
            }
            int indice = c.dequeueInputBuffer(-1);
            ByteBuffer ingresso = c.getInputBuffer(indice);
            ingresso.clear();
            ingresso.put(b.dati);
            c.queueInputBuffer(indice, 0, b.dati.length, b.orario, 0);
            svuota(c, info, coda, persi);
        }
    }

    /** Mette in coda tutto quello che il codificatore ha pronto. */
    private static void svuota(MediaCodec c, MediaCodec.BufferInfo info, BlockingQueue<byte[]> coda, AtomicLong persi) {
        while (true) {
            int indice = c.dequeueOutputBuffer(info, 0);
            if (indice < 0) {
                // Nessun dato pronto, o formato cambiato: si riprova al prossimo blocco.
                if (indice == MediaCodec.INFO_TRY_AGAIN_LATER) {
                    return;
                }
                continue;
            }
            if (info.size > 0) {
                ByteBuffer uscita = c.getOutputBuffer(indice);
                byte[] pacchetto = new byte[12 + info.size];
                boolean config = (info.flags & MediaCodec.BUFFER_FLAG_CODEC_CONFIG) != 0;
                ByteBuffer.wrap(pacchetto)
                        .putLong(config ? CONFIGURAZIONE : info.presentationTimeUs)
                        .putInt(info.size);
                uscita.position(info.offset);
                uscita.get(pacchetto, 12, info.size);
                metti(coda, pacchetto, persi);
            }
            c.releaseOutputBuffer(indice, false);
        }
    }

    /** Scrive i pacchetti sull'uscita del processo; finisce quando il PC chiude. */
    private static void spedisci(BlockingQueue<byte[]> coda) {
        try (DataOutputStream uscita = new DataOutputStream(
                new BufferedOutputStream(new FileOutputStream(FileDescriptor.out), 1 << 16))) {
            while (true) {
                uscita.write(coda.take());
                if (coda.isEmpty()) {
                    uscita.flush();
                }
            }
        } catch (IOException | InterruptedException e) {
            // Il PC ha chiuso il canale: il thread principale se ne accorge e finisce.
        }
    }

    /** Dopo un errore: lascia al PC il tempo di ricevere l'ultima riga. */
    private static void attendiSpedizione(BlockingQueue<byte[]> coda) throws InterruptedException {
        for (int i = 0; i < 20 && !coda.isEmpty(); i++) {
            Thread.sleep(100);
        }
        Thread.sleep(200);
    }
}
