package phonestra;

import android.app.ActivityManager;
import android.app.TaskStackListener;
import android.content.ComponentName;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.media.MediaCodec;
import android.media.MediaCodecInfo;
import android.media.MediaCodecList;
import android.media.MediaFormat;
import android.os.Bundle;
import android.view.Surface;

import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.function.ObjIntConsumer;

/**
 * Strumento di misura del video (fase 0 del componente nostro,
 * memoria/studio/video.md, «Prove da fare sul telefono»). Solo Android 14+.
 *
 * <p>{@code video-prova <prova> [--opzione valore]…}; stampa risultati di
 * testo, riga per riga. Ogni prova chiude sempre quello che ha aperto (schermi
 * virtuali, task, codificatori), anche in caso di errore, e termina da sola:
 * un guardiano la chiude comunque dopo un tempo massimo.
 */
final class VideoProva {
    private static final String CHROME = "com.android.chrome";
    /** Pagina che si anima a ogni fotogramma (già usata nel banco di prova). */
    private static final String PAGINA_ANIMATA = "https://www.testufo.com/";
    private static final int COLOR_FORMAT_SURFACE = 0x7F000789;

    private VideoProva() {
    }

    static void main(String[] argomenti) {
        if (argomenti.length == 0) {
            uso();
            System.exit(2);
        }
        String prova = argomenti[0];
        Opzioni o;
        try {
            o = new Opzioni(Arrays.copyOfRange(argomenti, 1, argomenti.length));
        } catch (IllegalArgumentException e) {
            Sistema.scrivi("errore: " + e.getMessage());
            uso();
            System.exit(2);
            return;
        }
        Pulizia pulizia = new Pulizia();
        guardiano(pulizia, limite(prova, o));
        int codice = 0;
        Sistema.scrivi("# prova «" + prova + "», Android " + Sistema.esegui("getprop ro.build.version.release")
                + " (API " + Sistema.esegui("getprop ro.build.version.sdk") + ")");
        try {
            switch (prova) {
                case "schermo":
                    schermo(o, pulizia);
                    break;
                case "chiave":
                    chiave(o, pulizia);
                    break;
                case "istanze":
                    istanze(o, pulizia);
                    break;
                case "protetto":
                    protetto(o, pulizia);
                    break;
                case "task":
                    task(o, pulizia);
                    break;
                case "permessi":
                    permessi();
                    break;
                case "codificatori":
                    codificatori();
                    break;
                default:
                    uso();
                    codice = 2;
            }
        } catch (Throwable e) {
            codice = 1;
            Sistema.scrivi("errore: " + Nascoste.causa(e));
            Throwable t = e;
            while (t.getCause() != null) {
                t = t.getCause();
            }
            for (StackTraceElement r : t.getStackTrace()) {
                Sistema.scrivi("#   " + r);
            }
        } finally {
            pulizia.esegui();
        }
        Sistema.scrivi("fine della prova «" + prova + "»" + (codice == 0 ? "" : " (con errori)"));
        System.exit(codice);
    }

    private static void uso() {
        Sistema.scrivi("uso: video-prova <prova> [opzioni]\n"
                + "  schermo      [--app P] [--sempre-sbloccato] [--aperto S] [--misura LxA] [--dpi D]\n"
                + "  chiave       [--codec avc,hevc] [--secondi S] [--app P [--url U]] [--nome CODIFICATORE] [--misura LxA]\n"
                + "  istanze      [--codec hevc] [--misura 1120x1992] [--massimo N] [--app P1,P2…]\n"
                + "  protetto     [--app P] [--attesa S]\n"
                + "  task         [--secondi 30] [--app P1,P2…] [--senza-schermo]\n"
                + "  permessi\n"
                + "  codificatori");
    }

    /** Secondi dopo i quali il guardiano chiude la prova comunque. */
    private static int limite(String prova, Opzioni o) {
        switch (prova) {
            case "schermo":
                return 60 + o.intero("aperto", 0);
            case "chiave":
                return 90 + 2 * o.testo("codec", "avc,hevc").split(",").length * (o.intero("secondi", 20) + 10);
            case "istanze":
                return 60 + 15 * o.intero("massimo", 8);
            case "protetto":
                return 60 + o.intero("attesa", 3);
            case "task":
                return 60 + o.intero("secondi", 30);
            default:
                return 60;
        }
    }

    private static void guardiano(Pulizia pulizia, int secondi) {
        Thread t = new Thread(() -> {
            Sistema.attendi(secondi * 1000L);
            Sistema.scrivi("errore: tempo massimo di " + secondi + " s superato, chiudo la prova");
            pulizia.esegui();
            System.exit(3);
        }, "guardiano");
        t.setDaemon(true);
        t.start();
    }

    // ------------------------------------------------------------------ schermo

    /**
     * Prove 2 e 3 di video.md: schermo virtuale coi flag proposti (con o senza
     * ALWAYS_UNLOCKED) su un ImageReader, app avviata lì, PNG dopo 2 s, flag
     * effettivi; con {@code --aperto S} resta aperto S secondi (per bloccare e
     * sbloccare il telefono intanto) con un PNG ogni 10 s.
     */
    private static void schermo(Opzioni o, Pulizia pulizia) throws Exception {
        int[] misura = o.misura("misura", 1120, 1992);
        int dpi = o.intero("dpi", 448);
        int aperto = o.intero("aperto", 0);
        boolean sbloccato = o.vero("sempre-sbloccato");
        String app = o.testo("app", Sistema.orologio());
        int flag = Sistema.FLAG_PROPOSTI | (sbloccato ? Sistema.ALWAYS_UNLOCKED : 0);
        Sistema.scrivi("flag chiesti: " + Sistema.nomiFlag(flag));
        Sistema.sysUiState("prima");

        Sistema.Cattura cattura = new Sistema.Cattura(misura[0], misura[1]);
        pulizia.aggiungi("lettore di immagini", cattura::close);
        String nome = "phonestra-prova-" + (sbloccato ? "sbloccato" : "normale");
        Sistema.Schermo s = Sistema.creaSchermo(pulizia, nome, misura[0], misura[1], dpi, cattura.superficie(), flag);
        Sistema.scrivi("schermo virtuale " + s.id + " (" + misura[0] + "×" + misura[1] + ", " + dpi + " dpi)");
        Sistema.scrivi("flag effettivi (Display.getFlags): " + Sistema.nomiFlagDisplay(s.display.getDisplay().getFlags()));
        Sistema.dumpsysDisplay(nome);
        Sistema.scrivi(Sistema.avviaApp(s.id, app, null));
        Sistema.attendi(2000);
        cattura.png("phonestra-schermo-" + s.id);

        if (aperto > 0) {
            Sistema.scrivi("schermo aperto per " + aperto + " s: blocca e sblocca il telefono ora");
            long fine = System.currentTimeMillis() + aperto * 1000L;
            int n = 0;
            while (System.currentTimeMillis() < fine) {
                Sistema.attendi(Math.min(5000, Math.max(0, fine - System.currentTimeMillis())));
                long mancano = Math.max(0, (fine - System.currentTimeMillis()) / 1000);
                Sistema.scrivi("  telefono bloccato: " + Sistema.bloccato() + " (mancano " + mancano + " s)");
                if (++n % 2 == 0) {
                    cattura.png("phonestra-schermo-" + s.id + "-" + n * 5 + "s");
                }
            }
            Sistema.sysUiState("con lo schermo ancora aperto");
        }
        s.chiudi(pulizia);
        Sistema.attendi(500);
        Sistema.sysUiState("dopo la chiusura");
    }

    // ------------------------------------------------------------------ chiave

    /**
     * Prova 5 di video.md: codificatore hardware H.264/H.265 dalla Surface di
     * uno schermo virtuale con un'app che si muove; fotogramma chiave chiesto
     * ogni 2 s ({@code PARAMETER_KEY_REQUEST_SYNC_FRAME}), ritardo fino al primo
     * IDR; poi di nuovo con {@code KEY_PREPEND_HEADER_TO_SYNC_FRAMES=1}.
     */
    private static void chiave(Opzioni o, Pulizia pulizia) throws Exception {
        int[] misura = o.misura("misura", 1120, 1992);
        int dpi = o.intero("dpi", 448);
        int secondi = o.intero("secondi", 20);
        String nomeForzato = o.testo("nome", null);
        Sistema.Schermo s = Sistema.creaSchermo(pulizia, "phonestra-prova-chiave", misura[0], misura[1], dpi, null,
                Sistema.FLAG_PROPOSTI);
        Sistema.scrivi("schermo virtuale " + s.id + " (" + misura[0] + "×" + misura[1] + ")");
        avviaAnimata(o, s.id, 0);
        Sistema.attendi(4000);
        for (String codec : o.testo("codec", "avc,hevc").split(",")) {
            String mime = mime(codec);
            for (boolean anteponi : new boolean[] {false, true}) {
                misuraChiave(pulizia, s, mime, nomeForzato, misura, secondi, anteponi);
            }
        }
    }

    /** Avvia l'app indicata (o Chrome sulla pagina animata) sullo schermo. */
    private static void avviaAnimata(Opzioni o, int display, int indice) throws Exception {
        String[] app = o.testo("app", "").isEmpty() ? new String[0] : o.testo("app", "").split(",");
        if (app.length > 0) {
            if (indice < app.length) {
                String url = o.testo("url", null);
                Sistema.scrivi(Sistema.avviaApp(display, app[indice], indice == 0 ? url : null));
            }
            return;
        }
        if (indice > 0) {
            return;
        }
        if (Sistema.installata(CHROME)) {
            Sistema.scrivi(Sistema.avviaApp(display, CHROME, o.testo("url", PAGINA_ANIMATA)));
        } else {
            Sistema.scrivi("Chrome assente: avvio l'orologio (immagine quasi ferma)");
            Sistema.scrivi(Sistema.avviaApp(display, Sistema.orologio(), null));
        }
    }

    private static String mime(String codec) {
        switch (codec.trim().toLowerCase(Locale.ROOT)) {
            case "avc":
            case "h264":
                return "video/avc";
            case "hevc":
            case "h265":
                return "video/hevc";
            case "av1":
                return "video/av01";
            default:
                return codec.trim();
        }
    }

    /** Primo codificatore hardware (non alias) per il tipo, o quello col nome dato. */
    private static MediaCodecInfo codificatoreHardware(String mime, String nome) {
        for (MediaCodecInfo info : new MediaCodecList(MediaCodecList.REGULAR_CODECS).getCodecInfos()) {
            if (!info.isEncoder() || !supporta(info, mime)) {
                continue;
            }
            if (nome != null ? info.getName().equals(nome) : info.isHardwareAccelerated() && !info.isAlias()) {
                return info;
            }
        }
        return null;
    }

    private static boolean supporta(MediaCodecInfo info, String mime) {
        for (String t : info.getSupportedTypes()) {
            if (t.equalsIgnoreCase(mime)) {
                return true;
            }
        }
        return false;
    }

    /** Misura arrotondata per difetto all'allineamento chiesto dal codificatore. */
    private static int[] allinea(MediaCodecInfo info, String mime, int[] misura) {
        MediaCodecInfo.VideoCapabilities v = info.getCapabilitiesForType(mime).getVideoCapabilities();
        int al = Math.max(1, v.getWidthAlignment());
        int aa = Math.max(1, v.getHeightAlignment());
        return new int[] {misura[0] / al * al, misura[1] / aa * aa};
    }

    /** Formato del codificatore (video.md §3.1), senza le opzioni facoltative. */
    private static MediaFormat formato(String mime, int[] misura, int bitrate) {
        MediaFormat f = MediaFormat.createVideoFormat(mime, misura[0], misura[1]);
        f.setInteger("bitrate", bitrate);
        f.setInteger("frame-rate", 60);
        f.setInteger("color-format", COLOR_FORMAT_SURFACE);
        f.setInteger("i-frame-interval", 10);
        f.setLong("repeat-previous-frame-after", 100_000L);
        f.setInteger("priority", 0);
        f.setInteger("color-range", 2); // COLOR_RANGE_LIMITED
        return f;
    }

    /** Un'uscita del codificatore: quando è arrivata e com'era fatta. */
    private static final class Uscita {
        final long tempo;
        final boolean chiave;
        final int byte_;
        final boolean intestazione;

        Uscita(long tempo, boolean chiave, int byte_, boolean intestazione) {
            this.tempo = tempo;
            this.chiave = chiave;
            this.byte_ = byte_;
            this.intestazione = intestazione;
        }
    }

    /** Svuota l'uscita di un codificatore su un thread suo e annota ogni fotogramma. */
    private static final class Lettore extends Thread {
        private final MediaCodec codificatore;
        private final boolean hevc;
        private final boolean annota;
        final List<Uscita> uscite = new ArrayList<>();
        volatile int fotogrammi;
        volatile String errore;
        private volatile boolean fermo;

        Lettore(MediaCodec codificatore, String mime, boolean annota) {
            super("lettore " + codificatore.getName());
            this.codificatore = codificatore;
            this.hevc = mime.equals("video/hevc");
            this.annota = annota;
            setDaemon(true);
        }

        @Override
        public void run() {
            MediaCodec.BufferInfo info = new MediaCodec.BufferInfo();
            try {
                while (!fermo) {
                    int i = codificatore.dequeueOutputBuffer(info, 50_000);
                    if (i < 0) {
                        continue;
                    }
                    long adesso = System.nanoTime();
                    if ((info.flags & MediaCodec.BUFFER_FLAG_CODEC_CONFIG) == 0) {
                        fotogrammi++;
                        if (annota) {
                            boolean chiave = (info.flags & MediaCodec.BUFFER_FLAG_KEY_FRAME) != 0;
                            boolean intestazione = chiave
                                    && haIntestazione(codificatore.getOutputBuffer(i), info.offset, info.size, hevc);
                            synchronized (uscite) {
                                uscite.add(new Uscita(adesso, chiave, info.size, intestazione));
                            }
                        }
                    }
                    codificatore.releaseOutputBuffer(i, false);
                }
            } catch (Exception e) {
                if (!fermo) {
                    errore = Nascoste.causa(e);
                }
            }
        }

        void ferma() {
            fermo = true;
            try {
                join(2000);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
        }
    }

    /** Il fotogramma contiene SPS (H.264) o VPS/SPS (H.265) prima dei dati? */
    static boolean haIntestazione(ByteBuffer b, int inizio, int quanti, boolean hevc) {
        if (b == null) {
            return false;
        }
        int fine = inizio + Math.min(quanti, 1024);
        for (int i = inizio; i + 3 < fine; i++) {
            if (b.get(i) == 0 && b.get(i + 1) == 0 && b.get(i + 2) == 1) {
                int n = b.get(i + 3) & 0xff;
                int tipo = hevc ? n >> 1 & 0x3f : n & 0x1f;
                if (hevc ? tipo == 32 || tipo == 33 : tipo == 7) {
                    return true;
                }
            }
        }
        return false;
    }

    private static void misuraChiave(Pulizia pulizia, Sistema.Schermo s, String mime, String nomeForzato, int[] misura,
            int secondi, boolean anteponi) throws Exception {
        MediaCodecInfo info = codificatoreHardware(mime, nomeForzato);
        if (info == null) {
            Sistema.scrivi(mime + ": nessun codificatore hardware");
            return;
        }
        int[] m = allinea(info, mime, misura);
        String titolo = mime + " " + info.getName() + " " + m[0] + "×" + m[1]
                + (anteponi ? " con prepend-sps-pps-to-idr-frames=1" : "");
        Sistema.scrivi("== " + titolo);
        MediaCodec c = MediaCodec.createByCodecName(info.getName());
        String voce = "codificatore " + info.getName();
        pulizia.aggiungi(voce, c::release);
        MediaFormat f = formato(mime, m, 8_000_000);
        if (anteponi) {
            f.setInteger("prepend-sps-pps-to-idr-frames", 1);
        }
        try {
            c.configure(f, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
        } catch (Exception e) {
            Sistema.scrivi("configure rifiutato: " + Nascoste.causa(e));
            pulizia.chiudi(voce);
            return;
        }
        Surface superficie = c.createInputSurface();
        c.start();
        Lettore lettore = new Lettore(c, mime, true);
        lettore.start();
        s.display.setSurface(superficie);
        List<Long> richieste = new ArrayList<>();
        long inizio = System.nanoTime();
        try {
            Sistema.attendi(2000);
            for (int t = 0; t < secondi; t += 2) {
                Bundle b = new Bundle();
                b.putInt(MediaCodec.PARAMETER_KEY_REQUEST_SYNC_FRAME, 0);
                richieste.add(System.nanoTime());
                c.setParameters(b);
                Sistema.attendi(2000);
                if (lettore.errore != null) {
                    break;
                }
            }
        } finally {
            s.display.setSurface(null);
            lettore.ferma();
            try {
                c.stop();
            } catch (Exception e) {
                Sistema.scrivi("stop: " + Nascoste.causa(e));
            }
            pulizia.chiudi(voce);
            superficie.release();
        }
        if (lettore.errore != null) {
            Sistema.scrivi("errore del codificatore: " + lettore.errore);
        }
        analizzaChiave(richieste, lettore.uscite, (System.nanoTime() - inizio) / 1e9);
    }

    private static void analizzaChiave(List<Long> richieste, List<Uscita> uscite, double durata) {
        List<Double> ritardi = new ArrayList<>();
        Set<Uscita> abbinate = new HashSet<>();
        int mancate = 0;
        long totaleByte = 0;
        int chiavi = 0;
        int conIntestazione = 0;
        long byteChiavi = 0;
        for (Uscita u : uscite) {
            totaleByte += u.byte_;
            if (u.chiave) {
                chiavi++;
                byteChiavi += u.byte_;
                if (u.intestazione) {
                    conIntestazione++;
                }
            }
        }
        for (int r = 0; r < richieste.size(); r++) {
            long t = richieste.get(r);
            long limite = r + 1 < richieste.size() ? richieste.get(r + 1) : t + 2_000_000_000L;
            int prima = 0;
            Uscita trovata = null;
            for (Uscita u : uscite) {
                if (u.tempo < t || u.tempo >= limite) {
                    continue;
                }
                if (u.chiave) {
                    trovata = u;
                    break;
                }
                prima++;
            }
            if (trovata == null) {
                mancate++;
                Sistema.scrivi(String.format("  richiesta %2d: nessun IDR entro %d ms", r + 1, (limite - t) / 1_000_000));
            } else {
                double ms = (trovata.tempo - t) / 1e6;
                ritardi.add(ms);
                abbinate.add(trovata);
                Sistema.scrivi(String.format("  richiesta %2d: IDR dopo %6.1f ms (%d fotogrammi prima), %d KB%s", r + 1, ms,
                        prima, trovata.byte_ / 1024, trovata.intestazione ? ", con intestazione" : ""));
            }
        }
        if (!ritardi.isEmpty()) {
            double min = Double.MAX_VALUE;
            double max = 0;
            double somma = 0;
            for (double d : ritardi) {
                min = Math.min(min, d);
                max = Math.max(max, d);
                somma += d;
            }
            Sistema.scrivi(String.format("ritardo richiesta→IDR: minimo %.1f ms, medio %.1f ms, massimo %.1f ms", min,
                    somma / ritardi.size(), max));
        }
        int fotogrammi = uscite.size();
        Sistema.scrivi(String.format("%d richieste, %d senza IDR; %d IDR in tutto (%d non chiesti), %d con SPS/PPS davanti",
                richieste.size(), mancate, chiavi, chiavi - abbinate.size(), conIntestazione));
        Sistema.scrivi(String.format("%.1f fotogrammi/s, %.0f KB/s, IDR medio %d KB, fotogramma medio %d KB",
                fotogrammi / durata, totaleByte / 1024.0 / durata, chiavi > 0 ? byteChiavi / chiavi / 1024 : 0,
                fotogrammi > 0 ? totaleByte / fotogrammi / 1024 : 0));
    }

    // ------------------------------------------------------------------ istanze

    /** Uno schermo virtuale con il suo codificatore. */
    private static final class Istanza {
        final Lettore lettore;
        final Sistema.Schermo schermo;

        Istanza(Lettore lettore, Sistema.Schermo schermo) {
            this.lettore = lettore;
            this.schermo = schermo;
        }
    }

    /**
     * Prova 8 di video.md: apre 1, 2, 3… schermi virtuali con codificatore
     * finché uno fallisce; confronta con quanto dichiara il codificatore.
     */
    private static void istanze(Opzioni o, Pulizia pulizia) throws Exception {
        String mime = mime(o.testo("codec", "hevc"));
        int massimo = o.intero("massimo", 8);
        int dpi = o.intero("dpi", 448);
        MediaCodecInfo info = codificatoreHardware(mime, o.testo("nome", null));
        if (info == null) {
            Sistema.scrivi(mime + ": nessun codificatore hardware");
            return;
        }
        int[] m = allinea(info, mime, o.misura("misura", 1120, 1992));
        Sistema.scrivi("codificatore " + info.getName() + ", " + m[0] + "×" + m[1]);
        dichiarati(info, mime, m);
        List<Istanza> aperte = new ArrayList<>();
        int reggono = 0;
        for (int n = 1; n <= massimo; n++) {
            Istanza nuova;
            try {
                nuova = apriIstanza(pulizia, info, mime, m, dpi, n);
            } catch (Exception e) {
                Sistema.scrivi("istanza " + n + ": fallita all'apertura: " + Nascoste.causa(e));
                break;
            }
            aperte.add(nuova);
            avviaAnimata(o, nuova.schermo.id, n - 1);
            for (Istanza i : aperte) {
                i.lettore.fotogrammi = 0;
            }
            Sistema.attendi(3000);
            StringBuilder riga = new StringBuilder("con " + n + " aperte, fotogrammi/s:");
            boolean errore = false;
            for (Istanza i : aperte) {
                riga.append(String.format(" %.1f", i.lettore.fotogrammi / 3.0));
                if (i.lettore.errore != null) {
                    riga.append(" (errore: ").append(i.lettore.errore).append(')');
                    errore = true;
                }
            }
            Sistema.scrivi(riga.toString());
            if (errore) {
                break;
            }
            reggono = n;
        }
        Sistema.scrivi("reggono insieme: " + reggono + (reggono == massimo ? " (limite della prova raggiunto)" : "")
                + "; dichiarate: " + info.getCapabilitiesForType(mime).getMaxSupportedInstances());
    }

    private static Istanza apriIstanza(Pulizia pulizia, MediaCodecInfo info, String mime, int[] m, int dpi, int n)
            throws Exception {
        MediaCodec c = MediaCodec.createByCodecName(info.getName());
        Lettore[] lettore = new Lettore[1];
        Surface[] superficie = new Surface[1];
        pulizia.aggiungi("codificatore " + n, () -> {
            if (lettore[0] != null) {
                lettore[0].ferma();
                c.stop();
            }
            c.release();
            if (superficie[0] != null) {
                superficie[0].release();
            }
        });
        c.configure(formato(mime, m, 8_000_000), null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
        superficie[0] = c.createInputSurface();
        c.start();
        lettore[0] = new Lettore(c, mime, false);
        lettore[0].start();
        Sistema.Schermo s = Sistema.creaSchermo(pulizia, "phonestra-prova-istanza-" + n, m[0], m[1], dpi, superficie[0],
                Sistema.FLAG_PROPOSTI);
        Sistema.scrivi("istanza " + n + ": schermo " + s.id + " e codificatore aperti");
        return new Istanza(lettore[0], s);
    }

    /** Quello che il codificatore dichiara: istanze, PerformancePoint, misura a 60 fps. */
    private static void dichiarati(MediaCodecInfo info, String mime, int[] m) {
        MediaCodecInfo.CodecCapabilities cc = info.getCapabilitiesForType(mime);
        MediaCodecInfo.VideoCapabilities v = cc.getVideoCapabilities();
        Sistema.scrivi("  getMaxSupportedInstances: " + cc.getMaxSupportedInstances());
        Sistema.scrivi("  areSizeAndRateSupported(" + m[0] + "×" + m[1] + ", 60): " + v.areSizeAndRateSupported(m[0], m[1], 60));
        List<MediaCodecInfo.VideoCapabilities.PerformancePoint> punti = v.getSupportedPerformancePoints();
        if (punti == null || punti.isEmpty()) {
            Sistema.scrivi("  PerformancePoint: nessuno dichiarato");
            return;
        }
        MediaCodecInfo.VideoCapabilities.PerformancePoint nostro =
                new MediaCodecInfo.VideoCapabilities.PerformancePoint(m[0], m[1], 60);
        for (MediaCodecInfo.VideoCapabilities.PerformancePoint p : punti) {
            Sistema.scrivi("  PerformancePoint " + p + (p.covers(nostro) ? " (copre " + m[0] + "×" + m[1] + "@60)" : ""));
        }
    }

    // ------------------------------------------------------------------ protetto

    /**
     * Prova 10 di video.md: app sullo schermo virtuale, poi
     * {@code IWindowManager.captureDisplay} e {@code containsSecureLayers()};
     * per confronto il PNG dell'ImageReader e le finestre SECURE di dumpsys.
     */
    private static void protetto(Opzioni o, Pulizia pulizia) throws Exception {
        int[] misura = o.misura("misura", 1120, 1992);
        int dpi = o.intero("dpi", 448);
        String app = o.testo("app", Sistema.orologio());
        Sistema.Cattura cattura = new Sistema.Cattura(misura[0], misura[1]);
        pulizia.aggiungi("lettore di immagini", cattura::close);
        Sistema.Schermo s = Sistema.creaSchermo(pulizia, "phonestra-prova-protetto", misura[0], misura[1], dpi,
                cattura.superficie(), Sistema.FLAG_PROPOSTI);
        Sistema.scrivi("schermo virtuale " + s.id);
        Sistema.scrivi(Sistema.avviaApp(s.id, app, null));
        Sistema.attendi(o.intero("attesa", 3) * 1000L);
        cattura.png("phonestra-protetto-" + s.id + "-lettore");
        Sistema.scrivi("dumpsys window: finestre con SECURE sullo schermo " + s.id + ": " + finestreProtette(s.id));
        long inizio = System.nanoTime();
        Object buffer = catturaDisplay(s.id);
        Sistema.scrivi(String.format("captureDisplay in %.0f ms", (System.nanoTime() - inizio) / 1e6));
        if (buffer == null) {
            Sistema.scrivi("captureDisplay: nessuna immagine");
            return;
        }
        try {
            Sistema.scrivi("containsSecureLayers: " + Nascoste.invoca(buffer, "containsSecureLayers"));
        } catch (Exception e) {
            Sistema.scrivi("containsSecureLayers non disponibile: " + Nascoste.causa(e));
        }
        try {
            Bitmap hw = (Bitmap) Nascoste.invoca(buffer, "asBitmap");
            Bitmap b = hw.copy(Bitmap.Config.ARGB_8888, false);
            int[] pixel = new int[b.getWidth() * b.getHeight()];
            b.getPixels(pixel, 0, b.getWidth(), 0, 0, b.getWidth(), b.getHeight());
            Sistema.salvaPng(b, pixel, "phonestra-protetto-" + s.id + "-cattura");
            hw.recycle();
        } catch (Exception e) {
            Sistema.scrivi("immagine della cattura non salvata: " + Nascoste.causa(e));
        }
        try {
            Object hb = Nascoste.invoca(buffer, "getHardwareBuffer");
            Nascoste.invoca(hb, "close");
        } catch (Exception e) {
            // già chiuso o metodo assente: niente da liberare
        }
    }

    /** Conta i blocchi «Window #» di dumpsys window sullo schermo con SECURE nei flag. */
    private static int finestreProtette(int display) {
        int n = 0;
        for (String blocco : Sistema.esegui("dumpsys window windows").split("Window #")) {
            if (blocco.contains("mDisplayId=" + display + " ") && blocco.matches("(?s).*fl=[^\\n]*\\bSECURE\\b.*")) {
                n++;
            }
        }
        return n;
    }

    /**
     * {@code IWindowManager.captureDisplay(displayId, CaptureArgs, ScreenCaptureListener)}:
     * la classe degli argomenti cambia tra Android 14–15 ({@code ScreenCapture}) e 16
     * ({@code ScreenCaptureInternal}), quindi si leggono i tipi dalla firma trovata.
     * Ascoltatore sincrono ({@code createSyncCaptureListener}) o, in ripiego, costruito
     * con un {@code ObjIntConsumer}. Attende al massimo 5 s.
     */
    private static Object catturaDisplay(int display) throws Exception {
        Object wm = Nascoste.windowManager();
        Method metodo = null;
        for (Method m : Nascoste.metodi(wm.getClass(), "captureDisplay")) {
            if (m.getParameterTypes().length == 3 && m.getParameterTypes()[0] == int.class) {
                metodo = m;
            }
        }
        if (metodo == null) {
            throw new NoSuchMethodException("IWindowManager.captureDisplay(int, …, …)");
        }
        Class<?>[] tipi = metodo.getParameterTypes();
        metodo.setAccessible(true);
        Sistema.scrivi("firma: captureDisplay(int, " + tipi[1].getName() + ", " + tipi[2].getName() + ")");
        Object argomenti = argomentiCattura(tipi[1]);
        Object[] risultato = new Object[1];
        CountDownLatch arrivato = new CountDownLatch(1);
        Object ascoltatore = null;
        Method sincrono = null;
        for (Class<?> c : new Class<?>[] {tipi[2].getDeclaringClass(), tipi[2]}) {
            if (c == null) {
                continue;
            }
            try {
                sincrono = c.getMethod("createSyncCaptureListener");
                break;
            } catch (NoSuchMethodException e) {
                // si prova la classe successiva
            }
        }
        if (sincrono != null) {
            ascoltatore = sincrono.invoke(null);
            Sistema.scrivi("ascoltatore: " + sincrono.getDeclaringClass().getSimpleName() + ".createSyncCaptureListener");
        } else {
            for (Constructor<?> c : tipi[2].getConstructors()) {
                if (c.getParameterTypes().length == 1 && c.getParameterTypes()[0] == ObjIntConsumer.class) {
                    ObjIntConsumer<Object> consumatore = (buffer, stato) -> {
                        risultato[0] = buffer;
                        Sistema.scrivi("stato della cattura: " + stato);
                        arrivato.countDown();
                    };
                    ascoltatore = c.newInstance(consumatore);
                    Sistema.scrivi("ascoltatore: costruttore con ObjIntConsumer");
                }
            }
        }
        if (ascoltatore == null) {
            throw new NoSuchMethodException("nessun modo di creare " + tipi[2].getName());
        }
        metodo.invoke(wm, display, argomenti, ascoltatore);
        if (sincrono != null) {
            Object a = ascoltatore;
            Thread t = new Thread(() -> {
                try {
                    risultato[0] = Nascoste.invoca(a, "getBuffer");
                } catch (Exception e) {
                    Sistema.scrivi("getBuffer: " + Nascoste.causa(e));
                }
                arrivato.countDown();
            }, "cattura");
            t.setDaemon(true);
            t.start();
        }
        if (!arrivato.await(5, TimeUnit.SECONDS)) {
            Sistema.scrivi("captureDisplay: nessuna risposta in 5 s");
        }
        return risultato[0];
    }

    /** CaptureArgs coi valori predefiniti (dal suo Builder); {@code null} se non si riesce. */
    private static Object argomentiCattura(Class<?> tipo) {
        try {
            Constructor<?> c = Class.forName(tipo.getName() + "$Builder").getDeclaredConstructor();
            c.setAccessible(true);
            Object costruttore = c.newInstance();
            Method build = costruttore.getClass().getMethod("build");
            build.setAccessible(true);
            return build.invoke(costruttore);
        } catch (Exception e) {
            Sistema.scrivi("CaptureArgs predefiniti non creati (" + Nascoste.causa(e) + "): passo null");
            return null;
        }
    }

    // ------------------------------------------------------------------ task

    private static long inizioTask;

    /**
     * Ascolta gli eventi dei task ({@code ITaskStackListener}) attraverso la
     * classe nascosta {@code TaskStackListener}, che implementa già tutti i
     * metodi (anche quelli aggiunti da Samsung) con corpi vuoti. Firme lette da
     * framework.jar di API 34 e 37 (immagini AOSP dell'SDK); in 37
     * {@code onTaskRequestedOrientationChanged} non c'è più.
     */
    private static final class Ascoltatore extends TaskStackListener {
        private static void evento(String testo) {
            Sistema.scrivi(String.format("[%6.1f s] %s", (System.nanoTime() - inizioTask) / 1e9, testo));
        }

        @Override
        public void onTaskCreated(int task, ComponentName componente) {
            evento("onTaskCreated task " + task + " " + (componente != null ? componente.flattenToShortString() : ""));
        }

        @Override
        public void onTaskRemoved(int task) {
            evento("onTaskRemoved task " + task);
        }

        @Override
        public void onTaskMovedToFront(ActivityManager.RunningTaskInfo info) {
            evento("onTaskMovedToFront " + descriviTask(info));
        }

        @Override
        public void onTaskMovedToBack(ActivityManager.RunningTaskInfo info) {
            evento("onTaskMovedToBack " + descriviTask(info));
        }

        @Override
        public void onTaskRemovalStarted(ActivityManager.RunningTaskInfo info) {
            evento("onTaskRemovalStarted " + descriviTask(info));
        }

        @Override
        public void onTaskDisplayChanged(int task, int display) {
            evento("onTaskDisplayChanged task " + task + " → schermo " + display);
        }

        @Override
        public void onTaskFocusChanged(int task, boolean fuoco) {
            evento("onTaskFocusChanged task " + task + " fuoco " + fuoco);
        }

        @Override
        public void onTaskRequestedOrientationChanged(int task, int orientamento) {
            evento("onTaskRequestedOrientationChanged task " + task + " " + orientamento(orientamento));
        }

        @Override
        public void onActivityRequestedOrientationChanged(int task, int orientamento) {
            evento("onActivityRequestedOrientationChanged task " + task + " " + orientamento(orientamento));
        }

        @Override
        public void onActivityRotation(int display) {
            evento("onActivityRotation schermo " + display);
        }

        @Override
        public void onActivityLaunchOnSecondaryDisplayFailed(ActivityManager.RunningTaskInfo info, int display) {
            evento("onActivityLaunchOnSecondaryDisplayFailed schermo " + display + " " + descriviTask(info));
        }

        @Override
        public void onActivityLaunchOnSecondaryDisplayRerouted(ActivityManager.RunningTaskInfo info, int display) {
            evento("onActivityLaunchOnSecondaryDisplayRerouted schermo " + display + " " + descriviTask(info));
        }

        @Override
        public void onBackPressedOnTaskRoot(ActivityManager.RunningTaskInfo info) {
            evento("onBackPressedOnTaskRoot " + descriviTask(info));
        }
    }

    private static String descriviTask(Object info) {
        if (info == null) {
            return "(null)";
        }
        Object attivita = Nascoste.campo(info, "topActivity");
        return "task " + Nascoste.campo(info, "taskId") + " schermo " + Nascoste.campo(info, "displayId") + " "
                + (attivita instanceof ComponentName ? ((ComponentName) attivita).flattenToShortString() : "");
    }

    private static final Map<Integer, String> ORIENTAMENTI = new HashMap<>();

    static {
        String[] nomi = {"LANDSCAPE", "PORTRAIT", "USER", "BEHIND", "SENSOR", "NOSENSOR", "SENSOR_LANDSCAPE",
            "SENSOR_PORTRAIT", "REVERSE_LANDSCAPE", "REVERSE_PORTRAIT", "FULL_SENSOR", "USER_LANDSCAPE",
            "USER_PORTRAIT", "FULL_USER", "LOCKED"};
        for (int i = 0; i < nomi.length; i++) {
            ORIENTAMENTI.put(i, nomi[i]);
        }
        ORIENTAMENTI.put(-1, "UNSPECIFIED");
        ORIENTAMENTI.put(-2, "UNSET");
    }

    private static String orientamento(int valore) {
        String nome = ORIENTAMENTI.get(valore);
        return valore + (nome != null ? " " + nome : "");
    }

    /**
     * Prova 11 di video.md: eventi dei task per {@code --secondi} secondi. Salvo
     * {@code --senza-schermo}, apre uno schermo virtuale e ci avvia le app di
     * {@code --app} (una ogni 5 s); intanto si può aprire la stessa app sul telefono.
     */
    private static void task(Opzioni o, Pulizia pulizia) throws Exception {
        int secondi = o.intero("secondi", 30);
        controllaFirme();
        Object atm = Nascoste.activityTaskManager();
        Ascoltatore ascoltatore = new Ascoltatore();
        inizioTask = System.nanoTime();
        Nascoste.invoca(atm, "registerTaskStackListener", ascoltatore);
        pulizia.aggiungi("ascoltatore dei task", () -> Nascoste.invoca(atm, "unregisterTaskStackListener", ascoltatore));
        Sistema.scrivi("ascolto gli eventi dei task per " + secondi + " s");
        long fine = System.nanoTime() + secondi * 1_000_000_000L;
        if (!o.vero("senza-schermo")) {
            Sistema.Cattura cattura = new Sistema.Cattura(1120, 1992);
            pulizia.aggiungi("lettore di immagini", cattura::close);
            Sistema.Schermo s = Sistema.creaSchermo(pulizia, "phonestra-prova-task", 1120, 1992, 448, cattura.superficie(),
                    Sistema.FLAG_PROPOSTI);
            Sistema.scrivi("schermo virtuale " + s.id);
            String elenco = o.testo("app", Sistema.orologio());
            for (String app : elenco.split(",")) {
                if (System.nanoTime() > fine) {
                    break;
                }
                Sistema.scrivi(Sistema.avviaApp(s.id, app.trim(), null));
                Sistema.attendi(5000);
            }
        }
        long resto = (fine - System.nanoTime()) / 1_000_000;
        if (resto > 0) {
            Sistema.attendi(resto);
        }
    }

    /** Segnala i nostri metodi che la classe del telefono non ha (firme diverse: eventi persi). */
    private static void controllaFirme() {
        try {
            Class<?> reale = Class.forName("android.app.TaskStackListener");
            Set<String> nostri = new HashSet<>();
            for (Method m : Ascoltatore.class.getDeclaredMethods()) {
                if (!m.getName().startsWith("on")) {
                    continue;
                }
                nostri.add(m.getName());
                try {
                    reale.getMethod(m.getName(), m.getParameterTypes());
                } catch (NoSuchMethodException e) {
                    Sistema.scrivi("attenzione: il telefono non ha " + m.getName() + Arrays.toString(m.getParameterTypes())
                            + ": quell'evento non arriverà");
                }
            }
            List<String> altri = new ArrayList<>();
            for (Method m : reale.getDeclaredMethods()) {
                if (m.getName().startsWith("on") && !nostri.contains(m.getName())) {
                    altri.add(m.getName());
                }
            }
            Sistema.scrivi("# eventi del telefono non ascoltati: " + altri);
        } catch (Exception e) {
            Sistema.scrivi("controllo delle firme non riuscito: " + Nascoste.causa(e));
        }
    }

    // ------------------------------------------------------------------ permessi e codificatori

    /** Prova 1 di video.md: i permessi della shell che servono al video. */
    private static void permessi() throws Exception {
        String[] permessi = {"ADD_TRUSTED_DISPLAY", "ADD_ALWAYS_UNLOCKED_DISPLAY", "CAPTURE_VIDEO_OUTPUT",
            "CAPTURE_SECURE_VIDEO_OUTPUT", "READ_FRAME_BUFFER", "MANAGE_ACTIVITY_TASKS", "REMOVE_TASKS",
            "INTERNAL_SYSTEM_WINDOW", "START_ACTIVITIES_FROM_BACKGROUND", "MANAGE_DISPLAYS", "DEVICE_POWER"};
        PackageManager pm = Contesto.shell().getPackageManager();
        for (String p : permessi) {
            boolean si = pm.checkPermission("android.permission." + p, Sistema.SHELL) == PackageManager.PERMISSION_GRANTED;
            Sistema.scrivi(String.format("  %-34s %s", p, si ? "sì" : "no"));
        }
        Sistema.scrivi("modello " + Sistema.esegui("getprop ro.product.model") + ", SoC "
                + Sistema.esegui("getprop ro.soc.model"));
    }

    /** Prova 14 di video.md: i codificatori video, con allineamenti, profili e limiti. */
    private static void codificatori() {
        for (MediaCodecInfo info : new MediaCodecList(MediaCodecList.REGULAR_CODECS).getCodecInfos()) {
            if (!info.isEncoder()) {
                continue;
            }
            for (String tipo : info.getSupportedTypes()) {
                if (!tipo.startsWith("video/")) {
                    continue;
                }
                MediaCodecInfo.CodecCapabilities cc = info.getCapabilitiesForType(tipo);
                MediaCodecInfo.VideoCapabilities v = cc.getVideoCapabilities();
                Sistema.scrivi(info.getName() + " " + tipo + (info.isHardwareAccelerated() ? " hardware" : "")
                        + (info.isSoftwareOnly() ? " software" : "") + (info.isVendor() ? " fornitore" : "")
                        + (info.isAlias() ? " alias" : ""));
                Sistema.scrivi("  larghezze " + v.getSupportedWidths() + " altezze " + v.getSupportedHeights()
                        + " allineamento " + v.getWidthAlignment() + "×" + v.getHeightAlignment() + ", bitrate "
                        + v.getBitrateRange() + ", istanze " + cc.getMaxSupportedInstances());
                MediaCodecInfo.EncoderCapabilities e = cc.getEncoderCapabilities();
                Sistema.scrivi("  CQ " + e.isBitrateModeSupported(0) + ", VBR " + e.isBitrateModeSupported(1) + ", CBR "
                        + e.isBitrateModeSupported(2) + ", intra-refresh " + cc.isFeatureSupported("intra-refresh")
                        + ", 1120×1992@60 " + v.areSizeAndRateSupported(1120, 1992, 60));
                StringBuilder profili = new StringBuilder("  profili/livelli:");
                if (cc.profileLevels != null) {
                    for (MediaCodecInfo.CodecProfileLevel pl : cc.profileLevels) {
                        profili.append(String.format(" %x/%x", pl.profile, pl.level));
                    }
                }
                Sistema.scrivi(profili.toString());
                List<MediaCodecInfo.VideoCapabilities.PerformancePoint> punti = v.getSupportedPerformancePoints();
                Sistema.scrivi("  PerformancePoint: " + (punti == null ? "nessuno" : punti.toString()));
            }
        }
    }

    // ------------------------------------------------------------------ opzioni

    /** {@code --nome valore} o {@code --nome} da solo (vero). */
    private static final class Opzioni {
        private final Map<String, String> valori = new HashMap<>();

        Opzioni(String[] argomenti) {
            for (int i = 0; i < argomenti.length; i++) {
                if (!argomenti[i].startsWith("--")) {
                    throw new IllegalArgumentException("opzione non valida: " + argomenti[i]);
                }
                String nome = argomenti[i].substring(2);
                if (i + 1 < argomenti.length && !argomenti[i + 1].startsWith("--")) {
                    valori.put(nome, argomenti[++i]);
                } else {
                    valori.put(nome, "sì");
                }
            }
        }

        String testo(String nome, String predefinito) {
            return valori.getOrDefault(nome, predefinito);
        }

        int intero(String nome, int predefinito) {
            String v = valori.get(nome);
            return v == null ? predefinito : Integer.parseInt(v);
        }

        boolean vero(String nome) {
            return valori.containsKey(nome);
        }

        /** {@code LxA}, per esempio 1120x1992. */
        int[] misura(String nome, int l, int a) {
            String v = valori.get(nome);
            if (v == null) {
                return new int[] {l, a};
            }
            String[] parti = v.toLowerCase(Locale.ROOT).split("[x×]");
            return new int[] {Integer.parseInt(parti[0]), Integer.parseInt(parti[1])};
        }
    }
}
