package phonestra;

import android.content.Intent;
import android.hardware.display.DisplayManager;
import android.hardware.display.VirtualDisplay;
import android.net.LocalSocket;
import android.net.Uri;
import android.view.Surface;

import java.io.IOException;
import java.io.OutputStream;
import java.lang.reflect.Method;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

/**
 * Una finestra di Phonestra sul telefono: uno schermo virtuale (con l'app) o lo
 * specchio dello schermo principale (per il drawer), il suo codificatore e il
 * suo canale {@code video:<id>}. notes/component.md, «Video».
 *
 * <p>Pacchetti sul canale nel formato che il PC legge già
 * ({@code sessione::leggi_pacchetto}): intestazione di 12 byte big-endian,
 * <ul>
 *   <li>nuova misura: {@code 0x80000000 · larghezza u32 · altezza u32};
 *   <li>dati: {@code pts u64} (µs dell'orologio monotono del telefono, lo
 *       stesso dell'audio, bit 62 = parametri del
 *       codec, bit 61 = fotogramma chiave) {@code · lunghezza u32}, poi i dati
 *       (Annex B, come escono da MediaCodec).
 * </ul>
 *
 * <p>Il codificatore parte quando il PC apre il canale: così il primo pacchetto
 * è la misura, poi i parametri e il primo fotogramma chiave, e non si perde niente.
 */
final class SessioneVideo implements Codifica.Uscita {
    static final long FLAG_SESSIONE = 1L << 63;
    static final long FLAG_CONFIG = 1L << 62;
    static final long FLAG_CHIAVE = 1L << 61;

    /** Lato massimo dello specchio dello schermo principale (oggi {@code max_size=1920}). */
    static final int LATO_SPECCHIO = 1920;

    /**
     * Dopo una richiesta di fotogramma chiave, se in questo tempo non ne è uscito
     * nessuno si fa ridisegnare lo schermo ({@link #ridisegna}); poi si riprova
     * una volta sola.
     */
    static final long ATTESA_CHIAVE_MS = 80;
    private static final ScheduledExecutorService ORARIO = Executors.newSingleThreadScheduledExecutor(r -> {
        Thread t = new Thread(r, "fotogramma chiave");
        t.setDaemon(true);
        return t;
    });

    final int id;
    final boolean specchio;
    final String mime;
    final int dpi;
    /** Lo schermo di cui si parla negli eventi: quello virtuale, o 0 per lo specchio. */
    final int display;
    /** Task visti sullo schermo (per «spostata» e «rimosso»), aggiornati da {@link EventiApp}. */
    final Set<Integer> task = ConcurrentHashMap.newKeySet();
    /** Ultimo stato mandato al PC ({@code null} = non ancora mandato), usato da {@link EventiApp}. */
    Integer orientamentoMandato;
    Boolean protettaMandata;
    boolean protettaInErrore;

    private VirtualDisplay schermo;
    private int larghezza;
    private int altezza;
    private int lato;
    private final Object scrittura = new Object();
    private Codifica codifica;
    private volatile OutputStream uscita;
    private volatile LocalSocket socket;
    private boolean orologioVisto;
    private byte[] ultimaConfig;
    private boolean rimandaConfig;
    /** Quando è uscito l'ultimo fotogramma chiave ({@code System.nanoTime}). */
    private volatile long ultimaChiave;
    private volatile boolean chiusa;

    private SessioneVideo(int id, boolean specchio, String mime, int dpi, VirtualDisplay schermo, int larghezza,
            int altezza) {
        this.id = id;
        this.specchio = specchio;
        this.mime = mime;
        this.dpi = dpi;
        this.schermo = schermo;
        this.display = specchio ? 0 : schermo.getDisplay().getDisplayId();
        this.larghezza = larghezza;
        this.altezza = altezza;
    }

    /**
     * Apre la sessione con le opzioni del PC ({@code chiave=valore}):
     * {@code codec}, e {@code larghezza altezza dpi} per lo schermo di un'app,
     * oppure {@code specchio=1} (e {@code lato_massimo}) per lo schermo principale.
     */
    static SessioneVideo apri(int id, Map<String, String> o) throws Exception {
        String mime = Codifica.mime(o.getOrDefault("codec", "h264"));
        if ("1".equals(o.get("specchio"))) {
            int lato = intero(o, "lato_massimo", LATO_SPECCHIO);
            int[] m = misuraSpecchio(mime, lato);
            SessioneVideo s = new SessioneVideo(id, true, mime, 1, creaSpecchio(id, m, null), m[0], m[1]);
            s.lato = lato;
            s.seguiRotazione();
            return s;
        }
        int[] m = Codifica.allinea(mime, intero(o, "larghezza", 720), intero(o, "altezza", 1280));
        int dpi = intero(o, "dpi", 320);
        VirtualDisplay vd = Sistema.displayManager().createVirtualDisplay("phonestra-" + id, m[0], m[1], dpi, null,
                Sistema.FLAG_PROPOSTI);
        if (vd == null) {
            throw new IllegalStateException("createVirtualDisplay ha restituito null");
        }
        SessioneVideo s = new SessioneVideo(id, false, mime, dpi, vd, m[0], m[1]);
        // La forma la decide la finestra: le app che chiedono un orientamento non
        // ruotano lo schermo (altrimenti larghezza e altezza si scambierebbero e
        // il video diventerebbe una striscia). Stessi comandi che oggi manda il PC.
        int d = s.display;
        Thread t = new Thread(() -> Sistema.esegui("cmd window set-ignore-orientation-request -d " + d
                + " true; cmd window user-rotation -d " + d + " lock 0"), "orientamento " + d);
        t.setDaemon(true);
        t.start();
        return s;
    }

    static int intero(Map<String, String> o, String chiave, int predefinito) {
        String v = o.get(chiave);
        return v == null ? predefinito : Integer.parseInt(v.trim());
    }

    /** Risposta ad APRI: righe {@code chiave=valore}. */
    synchronized String descrizione() {
        return "id=" + id + "\ndisplay=" + display + "\ncodec=" + Codifica.breve(mime) + "\nlarghezza=" + larghezza
                + "\naltezza=" + altezza + "\n";
    }

    boolean collegata() {
        return uscita != null;
    }

    boolean chiusa() {
        return chiusa;
    }

    // ------------------------------------------------------------------ app

    /** Avvia l'app sullo schermo (come START_APP di oggi). */
    String avviaApp(String pacchetto) throws Exception {
        controllaApp();
        return Sistema.avviaApp(display, pacchetto, null);
    }

    /** Apre sullo schermo la pagina «Informazioni app» delle impostazioni. */
    String avviaInformazioni(String pacchetto) throws Exception {
        controllaApp();
        if (!pacchetto.matches("[A-Za-z0-9_.]+")) {
            throw new IllegalArgumentException("nome di pacchetto non valido");
        }
        Intent intent = new Intent("android.settings.APPLICATION_DETAILS_SETTINGS", Uri.parse("package:" + pacchetto));
        String ripiego = "am start --display " + display + " -a android.settings.APPLICATION_DETAILS_SETTINGS -d package:"
                + pacchetto;
        return Sistema.avviaIntent(display, intent, pacchetto, ripiego);
    }

    private void controllaApp() {
        if (specchio) {
            throw new IllegalStateException("sullo schermo principale non si avviano app");
        }
    }

    // ------------------------------------------------------------------ canale e codifica

    /** Il PC ha aperto il canale {@code video:<id>}: si comincia a codificare. */
    void collega(LocalSocket s) throws Exception {
        synchronized (scrittura) {
            if (uscita != null || chiusa) {
                throw new IllegalStateException("canale video già aperto o sessione chiusa");
            }
            socket = s;
            uscita = s.getOutputStream();
        }
        synchronized (this) {
            Codifica c = Codifica.crea(mime, larghezza, altezza);
            sostituisci(c);
            c.avvia(this);
            schermo.setSurface(c.superficie);
        }
    }

    /** La nuova codifica diventa quella attuale; prima dei suoi pacchetti va la misura. */
    private Codifica sostituisci(Codifica nuova) throws IOException {
        synchronized (scrittura) {
            Codifica vecchia = codifica;
            codifica = nuova;
            ultimaConfig = null;
            if (uscita != null) {
                scrivi(misura(nuova.larghezza, nuova.altezza));
                // L'input accetta da ora solo coordinate calcolate su questa misura.
                Input.dimensioneVideo(display, nuova.larghezza, nuova.altezza);
            }
            return vecchia;
        }
    }

    /**
     * Nuova misura dello schermo (RESIZE_DISPLAY di oggi): la densità resta
     * quella iniziale. Codificatore e Surface si ricreano solo se la misura,
     * allineata, cambia davvero; il nuovo si prepara prima di chiudere il vecchio.
     */
    synchronized String cambiaMisura(int l, int a) throws Exception {
        if (specchio) {
            return "specchio: misura dello schermo del telefono";
        }
        int[] m = Codifica.allinea(mime, l, a);
        if (m[0] == larghezza && m[1] == altezza) {
            return "misura invariata " + larghezza + "x" + altezza;
        }
        if (!collegata()) {
            Nascoste.invoca(schermo, "resize", m[0], m[1], dpi);
            larghezza = m[0];
            altezza = m[1];
            return "misura " + larghezza + "x" + altezza;
        }
        Codifica nuova = Codifica.crea(mime, m[0], m[1]);
        Codifica vecchia;
        try {
            vecchia = sostituisci(nuova);
        } catch (IOException e) {
            nuova.chiudi();
            throw e;
        }
        nuova.avvia(this);
        Nascoste.invoca(schermo, "resize", m[0], m[1], dpi);
        schermo.setSurface(nuova.superficie);
        larghezza = m[0];
        altezza = m[1];
        if (vecchia != null) {
            vecchia.chiudi();
        }
        return "misura " + larghezza + "x" + altezza;
    }

    /** Fotogramma chiave appena possibile, coi parametri davanti. */
    void chiave() {
        Codifica c;
        synchronized (scrittura) {
            rimandaConfig = true;
            c = codifica;
        }
        if (c == null) {
            throw new IllegalStateException("codifica non ancora avviata (canale video non aperto)");
        }
        long chiesta = System.nanoTime();
        c.chiave();
        controllaChiave(c, chiesta, 1);
    }

    /**
     * Il codificatore Qualcomm ignora {@code repeat-previous-frame-after}
     * (prove sul telefono: a schermo fermo nessun fotogramma), quindi
     * {@code REQUEST_SYNC_FRAME} aspetterebbe il prossimo cambiamento dello
     * schermo e la finestra resterebbe senza immagine. Se il fotogramma chiave
     * non esce entro {@link #ATTESA_CHIAVE_MS}, si fa ridisegnare lo schermo.
     */
    private void controllaChiave(Codifica c, long chiesta, int tentativo) {
        ORARIO.schedule(() -> {
            if (chiusa || ultimaChiave - chiesta > 0) {
                return;
            }
            synchronized (scrittura) {
                if (c != codifica) {
                    return; // codificatore cambiato: il nuovo comincia comunque con un fotogramma chiave
                }
            }
            ridisegna(c);
            if (tentativo < 2) {
                controllaChiave(c, chiesta, tentativo + 1);
            }
        }, ATTESA_CHIAVE_MS * tentativo, TimeUnit.MILLISECONDS);
    }

    /**
     * Ridisegno forzato: si stacca la Surface dallo schermo e la si riattacca.
     * Il compositore rifà l'uscita dello schermo virtuale e compone subito un
     * fotogramma nella Surface, che il codificatore codifica come fotogramma
     * chiave (la richiesta è già in sospeso). Vale anche per lo specchio.
     */
    private synchronized void ridisegna(Codifica c) {
        if (chiusa) {
            return;
        }
        try {
            schermo.setSurface(null);
            schermo.setSurface(c.superficie);
        } catch (Exception e) {
            Video.log("ridisegno dello schermo " + display + " non riuscito: " + Nascoste.causa(e));
        }
    }

    @Override
    public void pacchetto(Codifica da, boolean config, boolean chiave, long ptsUs, byte[] dati) {
        try {
            synchronized (scrittura) {
                if (da != codifica || uscita == null || chiusa) {
                    return; // codificatore già sostituito o sessione chiusa
                }
                if (config) {
                    ultimaConfig = dati;
                    scrivi(dati(FLAG_CONFIG, dati));
                    return;
                }
                if (chiave && rimandaConfig) {
                    rimandaConfig = false;
                    // Con «prepend-sps-pps» i parametri sono già nel fotogramma.
                    if (!da.anteponi && ultimaConfig != null) {
                        scrivi(dati(FLAG_CONFIG, ultimaConfig));
                    }
                }
                if (!orologioVisto) {
                    orologioVisto = true;
                    Video.log("orologio dei fotogrammi: pts_us=" + ptsUs + " monotono_us=" + System.nanoTime() / 1000);
                }
                long pts = Math.max(0, ptsUs) & ~(FLAG_SESSIONE | FLAG_CONFIG | FLAG_CHIAVE);
                scrivi(dati(pts | (chiave ? FLAG_CHIAVE : 0), dati));
                if (chiave) {
                    ultimaChiave = System.nanoTime();
                }
            }
        } catch (IOException e) {
            Video.chiudiPiuTardi(id, "canale video chiuso (" + Nascoste.causa(e) + ")");
        }
    }

    @Override
    public void errore(Codifica da, String causa) {
        synchronized (scrittura) {
            if (da != codifica) {
                return;
            }
        }
        Video.chiudiPiuTardi(id, "codificatore fermo: " + causa);
    }

    private void scrivi(byte[] b) throws IOException {
        uscita.write(b);
        uscita.flush();
    }

    /** Pacchetto «nuova misura». */
    static byte[] misura(int l, int a) {
        byte[] b = new byte[12];
        scriviU32(b, 0, (int) (FLAG_SESSIONE >>> 32));
        scriviU32(b, 4, l);
        scriviU32(b, 8, a);
        return b;
    }

    /** Pacchetto di dati: intestazione e contenuto in un blocco (una sola scrittura). */
    static byte[] dati(long testa, byte[] dati) {
        byte[] b = new byte[12 + dati.length];
        scriviU32(b, 0, (int) (testa >>> 32));
        scriviU32(b, 4, (int) testa);
        scriviU32(b, 8, dati.length);
        System.arraycopy(dati, 0, b, 12, dati.length);
        return b;
    }

    private static void scriviU32(byte[] b, int da, int v) {
        b[da] = (byte) (v >>> 24);
        b[da + 1] = (byte) (v >>> 16);
        b[da + 2] = (byte) (v >>> 8);
        b[da + 3] = (byte) v;
    }

    // ------------------------------------------------------------------ specchio

    /** Misura dello schermo principale adesso, ridotta a {@code lato} e allineata. */
    static int[] misuraSpecchio(String mime, int lato) throws Exception {
        Object dmg = Class.forName("android.hardware.display.DisplayManagerGlobal").getMethod("getInstance").invoke(null);
        Object info = Nascoste.invoca(dmg, "getDisplayInfo", 0);
        int l = (Integer) Nascoste.campo(info, "logicalWidth");
        int a = (Integer) Nascoste.campo(info, "logicalHeight");
        return riduci(mime, l, a, lato);
    }

    /** {@code l}×{@code a} ridotte perché il lato lungo non superi {@code lato}, poi allineate. */
    static int[] riduci(String mime, int l, int a, int lato) {
        int lungo = Math.max(l, a);
        if (lungo > lato) {
            l = (int) ((long) l * lato / lungo);
            a = (int) ((long) a * lato / lungo);
        }
        return Codifica.allinea(mime, l, a);
    }

    /**
     * Specchio dello schermo principale: {@code DisplayManager.createVirtualDisplay(nome,
     * l, a, schermoDaSpecchiare, surface)}, statica e nascosta ({@code CAPTURE_VIDEO_OUTPUT},
     * che la shell ha; study/video.md §1.2).
     */
    private static VirtualDisplay creaSpecchio(int id, int[] m, Surface superficie) throws Exception {
        Method crea = DisplayManager.class.getMethod("createVirtualDisplay", String.class, int.class, int.class,
                int.class, Surface.class);
        crea.setAccessible(true);
        VirtualDisplay vd = (VirtualDisplay) crea.invoke(null, "phonestra-specchio-" + id, m[0], m[1], 0, superficie);
        if (vd == null) {
            throw new IllegalStateException("specchio dello schermo principale non creato");
        }
        return vd;
    }

    /**
     * Il telefono ruota: lo specchio si rifà con i lati scambiati (nuovo
     * codificatore, nuovo specchio, poi si chiudono i vecchi), e il PC riceve
     * la nuova misura. Si controlla ogni mezzo secondo (una chiamata leggera).
     */
    private void seguiRotazione() {
        Thread t = new Thread(() -> {
            while (!chiusa) {
                Sistema.attendi(500);
                try {
                    int[] m = misuraSpecchio(mime, lato);
                    synchronized (this) {
                        if (!chiusa && collegata() && (m[0] != larghezza || m[1] != altezza)) {
                            rifaiSpecchio(m);
                        }
                    }
                } catch (Exception e) {
                    Video.chiudiPiuTardi(id, "specchio non rifatto: " + Nascoste.causa(e));
                    return;
                }
            }
        }, "rotazione " + id);
        t.setDaemon(true);
        t.start();
    }

    private void rifaiSpecchio(int[] m) throws Exception {
        Codifica nuova = Codifica.crea(mime, m[0], m[1]);
        VirtualDisplay nuovo;
        try {
            nuovo = creaSpecchio(id, m, nuova.superficie);
        } catch (Exception e) {
            nuova.chiudi();
            throw e;
        }
        Codifica vecchia = sostituisci(nuova);
        nuova.avvia(this);
        VirtualDisplay vecchio = schermo;
        schermo = nuovo;
        larghezza = m[0];
        altezza = m[1];
        vecchio.release();
        if (vecchia != null) {
            vecchia.chiudi();
        }
    }

    // ------------------------------------------------------------------ chiusura

    /**
     * Chiude il canale in modo che il PC se ne accorga. Il solo {@code close()}
     * non basta: il thread di {@link Video#canale} è fermo in {@code read} sullo
     * stesso socket, e in Linux {@code close} di un descrittore con una lettura
     * in corso in un altro thread non chiude davvero il socket (niente fine del
     * flusso verso adbd, e la lettura resta ferma). {@code shutdown} invece
     * manda la fine del flusso e sblocca la lettura; poi {@code close}.
     */
    static void chiudiCanale(LocalSocket s) {
        for (String metodo : new String[] {"shutdownOutput", "shutdownInput"}) {
            try {
                Nascoste.invoca(s, metodo);
            } catch (Exception e) {
                // già chiuso da questa parte o dal PC
            }
        }
        try {
            s.close();
        } catch (IOException e) {
            // già chiuso
        }
    }

    /**
     * Chiude tutto, una volta sola: codificatore, task (solo con {@code togliTask}:
     * la finestra chiusa dall'utente toglie l'app dalle recenti, come oggi),
     * schermo, canale.
     */
    void chiudi(boolean togliTask) {
        synchronized (this) {
            if (chiusa) {
                return;
            }
            chiusa = true;
        }
        // Prima il canale: una scrittura bloccata (rete ferma) finisce subito con
        // un errore e libera il lucchetto della scrittura.
        LocalSocket s = socket;
        if (s != null) {
            chiudiCanale(s);
        }
        Codifica c;
        synchronized (scrittura) {
            c = codifica;
            codifica = null;
            uscita = null;
        }
        if (c != null) {
            c.chiudi();
        }
        if (togliTask && !specchio) {
            try {
                Sistema.togliTask(display);
            } catch (Exception e) {
                Video.log("task dello schermo " + display + " non tolti: " + Nascoste.causa(e));
            }
        }
        synchronized (this) {
            try {
                schermo.release();
            } catch (Exception e) {
                Video.log("schermo " + display + " non chiuso: " + Nascoste.causa(e));
            }
        }
        Input.dimentica(display);
    }
}
