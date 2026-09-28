package phonestra;

import android.content.IOnPrimaryClipChangedListener;
import android.os.SystemClock;
import android.view.InputEvent;
import android.view.KeyCharacterMap;
import android.view.KeyEvent;
import android.view.MotionEvent;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Il modulo input del servizio (memoria/componente.md, «Input»): tocchi,
 * rotellina, tasti, testo, «indietro» e appunti, mandati dal PC sul canale
 * comandi e iniettati nello schermo indicato (principale o virtuale).
 *
 * <p>Fa quello che Phonestra fa oggi con scrcpy, nello stesso modo
 * (studio/input.md §2–§4, §6); codice nostro, scrcpy solo come documentazione:
 * <ul>
 *   <li>ogni evento con {@code InputEvent.setDisplayId} e
 *       {@code injectInputEvent} in modo asincrono: il PC non aspetta niente;
 *   <li>clic e dita come <b>dita</b> ({@code SOURCE_TOUCHSCREEN}): trascinare
 *       scorre, la pressione lunga apre i menu. Con più dita, un evento per
 *       messaggio con tutte le dita appoggiate, {@code POINTER_DOWN/UP} per
 *       quelle dopo la prima; al massimo 10 dita;
 *   <li>coordinate date rispetto alla misura dell'immagine che vede il PC,
 *       scalate su quella dello schermo; un evento calcolato su una misura
 *       vecchia (durante un ridimensionamento) si scarta;
 *   <li>rotellina come mouse ({@code ACTION_SCROLL}), valori frazionari;
 *   <li>testo con la mappa dei tasti virtuale (in pratica ASCII: il resto il
 *       PC lo manda con «incolla»); tasti con codice e modificatori;
 *   <li>incolla = appunti + tasto {@code PASTE}; le copie fatte sul telefono
 *       vanno al PC, quelle segnate come sensibili senza testo.
 * </ul>
 *
 * <p>Il thread che legge i comandi non inietta niente: mette i messaggi in
 * coda al thread «input», che li esegue in ordine.
 */
final class Input {
    // Tipi dei messaggi del canale comandi, fascia 0x50–0x5f (Protocollo.java).
    /** PC → servizio: tocchi di una o più dita. */
    static final int TOCCHI = 0x50;
    /** PC → servizio: rotellina. */
    static final int ROTELLINA = 0x51;
    /** PC → servizio: un tasto Android. */
    static final int TASTO = 0x52;
    /** PC → servizio: testo da scrivere coi tasti virtuali. */
    static final int TESTO = 0x53;
    /** PC → servizio: «indietro» (giù o su). */
    static final int INDIETRO = 0x54;
    /** PC → servizio: testo negli appunti, e se chiesto incollato; risposta se l'id non è 0. */
    static final int APPUNTI_SCRIVI = 0x55;
    /** PC → servizio, domanda: gli appunti attuali (stato e testo). */
    static final int APPUNTI_LEGGI = 0x56;
    /** PC → servizio: accende o spegne l'avviso delle copie; risposta se l'id non è 0. */
    static final int APPUNTI_ASCOLTA = 0x57;
    /** Servizio → PC, spontaneo: gli appunti sono cambiati sul telefono (stato e testo). */
    static final int APPUNTI_CAMBIATI = 0x58;
    /** PC → servizio, domanda di diagnosi: eventi iniettati, falliti, scartati. */
    static final int CONTEGGI = 0x5c;
    /** PC → servizio, domanda: comandi delle prove ({@link InputProva}). */
    static final int PROVA = 0x5d;

    // Stato degli appunti nelle risposte e negli avvisi.
    static final int APPUNTI_VUOTI = 0;
    static final int APPUNTI_TESTO = 1;
    static final int APPUNTI_SENSIBILI = 2;
    static final int APPUNTI_SCONOSCIUTI = 3;

    // Valori delle costanti Android (uguali da 14 a 16).
    static final int GIU = 0;
    static final int SU = 1;
    static final int MOVIMENTO = 2;
    private static final int POINTER_DOWN = 5;
    private static final int POINTER_UP = 6;
    private static final int SCROLL = 8;
    private static final int INDICE_SHIFT = 8;
    private static final int DITO = 1;
    private static final int MOUSE = 3;
    private static final int ASSE_VERTICALE = 9;
    private static final int ASSE_ORIZZONTALE = 10;
    private static final int SORGENTE_TOUCH = 0x1002;
    private static final int SORGENTE_MOUSE = 0x2002;
    private static final int SORGENTE_TASTIERA = 0x101;
    private static final int TASTIERA_VIRTUALE = -1;
    private static final int KEYCODE_BACK = 4;
    private static final int KEYCODE_POWER = 26;
    private static final int KEYCODE_PASTE = 279;
    /** {@code InputManager.INJECT_INPUT_EVENT_MODE_ASYNC}. */
    private static final int ASINCRONO = 0;

    /** Differenza di proporzioni oltre la quale la misura del PC è considerata vecchia. */
    static final double TOLLERANZA = 0.02;
    /** Una copia uguale al testo appena messo da noi entro questo tempo è nostra, non si rimanda. */
    private static final long ECO_MS = 3000;

    private static final ExecutorService CODA = Executors.newSingleThreadExecutor(r -> {
        Thread t = new Thread(r, "input");
        t.setDaemon(true);
        return t;
    });

    // Usati solo dal thread «input».
    private static final Map<Integer, Dita> DITA = new HashMap<>();
    private static Object gestore;
    private static Method inietta;
    private static Method impostaDisplay;
    private static Method infoDisplay;
    private static Object displayGlobale;
    private static KeyCharacterMap mappa;
    private static Ascoltatore ascoltatore;
    private static String ultimoMesso;
    private static long ultimoMessoQuando;

    /** Misura dell'immagine video di ogni schermo, se il modulo video la dichiara. */
    private static final Map<Integer, int[]> VIDEO = new HashMap<>();
    /** Mentre il servizio stesso cambia gli appunti: la copia non va rimandata al PC. */
    private static volatile boolean impostando;

    private static long iniettati;
    private static long falliti;
    private static long scartati;
    private static long cambiatiMandati;
    private static String ultimoErrore = "";
    private static long annotati;

    private Input() {
    }

    /** Se il tipo appartiene al modulo input (per il canale comandi). */
    static boolean nostro(int tipo) {
        return tipo >= 0x50 && tipo <= 0x5f;
    }

    /**
     * Dal thread che legge il canale comandi: il messaggio va in coda al thread
     * «input». I comandi delle prove (lenti: aprono schermi e app) hanno un
     * thread loro, così non fermano gli eventi.
     */
    static void ricevi(Protocollo.Messaggio m) {
        if (m.tipo == PROVA) {
            Thread t = new Thread(() -> InputProva.esegui(m), "input-prova");
            t.setDaemon(true);
            t.start();
            return;
        }
        CODA.execute(() -> {
            try {
                gestisci(m);
            } catch (Throwable e) {
                errore("messaggio 0x" + Integer.toHexString(m.tipo), e);
                if (m.id != 0) {
                    Servizio.manda(Protocollo.ERRORE, Protocollo.RISPOSTA, m.id,
                            Nascoste.causa(e).getBytes(StandardCharsets.UTF_8));
                }
            }
        });
    }

    /**
     * Per il modulo video: la misura dell'immagine che il PC vede per lo
     * schermo. Da quel momento gli eventi valgono solo se calcolati su questa
     * misura esatta (come scrcpy); senza, vale il controllo delle proporzioni.
     */
    static void dimensioneVideo(int display, int larghezza, int altezza) {
        CODA.execute(() -> VIDEO.put(display, new int[] {larghezza, altezza}));
    }

    /** Per il modulo video: lo schermo è stato chiuso, si dimenticano dita e misura. */
    static void dimentica(int display) {
        CODA.execute(() -> {
            VIDEO.remove(display);
            DITA.remove(display);
        });
    }

    private static void gestisci(Protocollo.Messaggio m) throws Exception {
        DataInputStream in = new DataInputStream(new ByteArrayInputStream(m.dati));
        switch (m.tipo) {
            case TOCCHI: {
                int display = in.readInt();
                int l = in.readUnsignedShort();
                int a = in.readUnsignedShort();
                int n = in.readUnsignedByte();
                for (int i = 0; i < n; i++) {
                    long dito = in.readLong();
                    int azione = in.readUnsignedByte();
                    int x = in.readInt();
                    int y = in.readInt();
                    float pressione = in.readFloat();
                    tocco(display, dito, azione, x, y, l, a, pressione);
                }
                break;
            }
            case ROTELLINA: {
                int display = in.readInt();
                int x = in.readInt();
                int y = in.readInt();
                int l = in.readUnsignedShort();
                int a = in.readUnsignedShort();
                float orizzontale = in.readFloat();
                float verticale = in.readFloat();
                rotellina(display, x, y, l, a, orizzontale, verticale);
                break;
            }
            case TASTO: {
                int display = in.readInt();
                int azione = in.readUnsignedByte();
                int codice = in.readInt();
                int ripetizione = in.readInt();
                int meta = in.readInt();
                tasto(display, azione, codice, ripetizione, meta);
                break;
            }
            case TESTO: {
                int display = in.readInt();
                testo(display, resto(in));
                break;
            }
            case INDIETRO: {
                int display = in.readInt();
                indietro(display, in.readUnsignedByte());
                break;
            }
            case APPUNTI_SCRIVI: {
                int display = in.readInt();
                boolean incolla = in.readUnsignedByte() != 0;
                String testo = resto(in);
                scriviAppunti(testo);
                if (incolla) {
                    tasto(display, GIU, KEYCODE_PASTE, 0, 0);
                    tasto(display, SU, KEYCODE_PASTE, 0, 0);
                }
                rispondi(m, new byte[0]);
                break;
            }
            case APPUNTI_LEGGI:
                rispondi(m, statoAppunti(null));
                break;
            case APPUNTI_ASCOLTA:
                ascolta(in.readUnsignedByte() != 0);
                rispondi(m, new byte[0]);
                break;
            case CONTEGGI:
                rispondi(m, conteggi().getBytes(StandardCharsets.UTF_8));
                break;
            default:
                throw new IllegalArgumentException(String.format("tipo sconosciuto 0x%02x", m.tipo));
        }
    }

    private static String resto(DataInputStream in) throws IOException {
        ByteArrayOutputStream o = new ByteArrayOutputStream();
        byte[] b = new byte[4096];
        int n;
        while ((n = in.read(b)) > 0) {
            o.write(b, 0, n);
        }
        return new String(o.toByteArray(), StandardCharsets.UTF_8);
    }

    private static void rispondi(Protocollo.Messaggio m, byte[] dati) {
        if (m.id != 0) {
            Servizio.manda(m.tipo, Protocollo.RISPOSTA, m.id, dati);
        }
    }

    // ------------------------------------------------------------------ iniezione

    private static void prepara() throws Exception {
        if (inietta != null) {
            return;
        }
        gestore = Nascoste.inputManager();
        Method m = Nascoste.metodo(gestore.getClass(), "injectInputEvent", 2, 3);
        impostaDisplay = Nascoste.metodo(InputEvent.class, "setDisplayId", 1);
        inietta = m;
    }

    /**
     * Inietta l'evento nello schermo {@code display}, senza aspettare l'app
     * (modo asincrono). Un errore si conta e si annota, non ferma gli altri.
     */
    private static void inietta(InputEvent evento, int display) {
        try {
            prepara();
            impostaDisplay.invoke(evento, display);
            Object esito = inietta.getParameterTypes().length == 2
                    ? inietta.invoke(gestore, evento, ASINCRONO)
                    // Terzo parametro: uid di destinazione, -1 = qualsiasi (Process.INVALID_UID).
                    : inietta.invoke(gestore, evento, ASINCRONO, -1);
            if (Boolean.FALSE.equals(esito)) {
                falliti++;
                annota("iniezione rifiutata sullo schermo " + display);
            } else {
                iniettati++;
            }
        } catch (Exception e) {
            falliti++;
            annota("iniezione sullo schermo " + display + ": " + Nascoste.causa(e));
        }
    }

    /** Misura logica dello schermo (larghezza, altezza); {@code null} se lo schermo non c'è. */
    private static int[] misuraSchermo(int display) throws Exception {
        if (infoDisplay == null) {
            Class<?> classe = Class.forName("android.hardware.display.DisplayManagerGlobal");
            displayGlobale = classe.getMethod("getInstance").invoke(null);
            infoDisplay = classe.getMethod("getDisplayInfo", int.class);
        }
        Object info = infoDisplay.invoke(displayGlobale, display);
        if (info == null) {
            return null;
        }
        Object l = Nascoste.campo(info, "logicalWidth");
        Object a = Nascoste.campo(info, "logicalHeight");
        if (!(l instanceof Integer) || !(a instanceof Integer)) {
            throw new IllegalStateException("DisplayInfo senza logicalWidth/logicalHeight");
        }
        return new int[] {(Integer) l, (Integer) a};
    }

    /**
     * Coordinate del PC ({@code x}, {@code y} su un'immagine di
     * {@code lRif}×{@code aRif}) in pixel dello schermo di misura {@code schermo}.
     * {@code null} se l'evento è calcolato su una misura vecchia: diversa da
     * quella dichiarata dal video ({@code video}, se c'è), oppure con
     * proporzioni diverse da quelle dello schermo oltre {@link #TOLLERANZA}
     * (la differenza piccola viene dagli arrotondamenti della misura del video).
     */
    static float[] scala(int x, int y, int lRif, int aRif, int[] schermo, int[] video) {
        if (lRif <= 0 || aRif <= 0 || schermo == null || schermo[0] <= 0 || schermo[1] <= 0) {
            return null;
        }
        if (video != null && (video[0] != lRif || video[1] != aRif)) {
            return null;
        }
        int l = schermo[0];
        int a = schermo[1];
        if (l == lRif && a == aRif) {
            return new float[] {x, y};
        }
        double rif = (double) lRif / aRif;
        double vero = (double) l / a;
        if (Math.abs(rif - vero) > TOLLERANZA * vero) {
            return null;
        }
        return new float[] {(float) ((double) x * l / lRif), (float) ((double) y * a / aRif)};
    }

    private static float[] punto(int display, int x, int y, int l, int a) throws Exception {
        float[] p = scala(x, y, l, a, misuraSchermo(display), VIDEO.get(display));
        if (p == null) {
            scartati++;
        }
        return p;
    }

    private static Dita dita(int display) {
        Dita d = DITA.get(display);
        if (d == null) {
            d = new Dita();
            DITA.put(display, d);
        }
        return d;
    }

    /**
     * Un dito ({@code dito} è l'identificativo del PC: −1 il mouse, −2 il dito
     * generico, altri per i gesti a più dita): giù, su o movimento.
     */
    static void tocco(int display, long dito, int azione, int x, int y, int l, int a, float pressione) throws Exception {
        if (azione != GIU && azione != SU && azione != MOVIMENTO) {
            scartati++;
            return;
        }
        float[] p = punto(display, x, y, l, a);
        if (p == null) {
            return;
        }
        Dita d = dita(display);
        long adesso = SystemClock.uptimeMillis();
        Dita.Evento e = d.tocca(dito, azione, p[0], p[1], pressione, adesso);
        if (e == null) {
            scartati++;
            return;
        }
        int n = e.x.length;
        MotionEvent.PointerProperties[] proprieta = new MotionEvent.PointerProperties[n];
        MotionEvent.PointerCoords[] coordinate = new MotionEvent.PointerCoords[n];
        for (int i = 0; i < n; i++) {
            proprieta[i] = new MotionEvent.PointerProperties();
            proprieta[i].id = e.id[i];
            proprieta[i].toolType = DITO;
            coordinate[i] = new MotionEvent.PointerCoords();
            coordinate[i].x = e.x[i];
            coordinate[i].y = e.y[i];
            coordinate[i].pressure = e.pressione[i];
            coordinate[i].size = 0;
        }
        // Pulsanti a zero: «Buttons must not be set for touch events».
        MotionEvent evento = MotionEvent.obtain(e.giu, adesso, e.azione, n, proprieta, coordinate, 0, 0, 1f, 1f, 0, 0,
                SORGENTE_TOUCH, 0);
        inietta(evento, display);
        evento.recycle();
    }

    /** Rotellina nel punto: scatti orizzontali (positivo = destra) e verticali (positivo = su). */
    static void rotellina(int display, int x, int y, int l, int a, float orizzontale, float verticale) throws Exception {
        float[] p = punto(display, x, y, l, a);
        if (p == null) {
            return;
        }
        MotionEvent.PointerProperties[] proprieta = {new MotionEvent.PointerProperties()};
        proprieta[0].id = 0;
        proprieta[0].toolType = MOUSE;
        MotionEvent.PointerCoords[] coordinate = {new MotionEvent.PointerCoords()};
        coordinate[0].x = p[0];
        coordinate[0].y = p[1];
        coordinate[0].setAxisValue(ASSE_ORIZZONTALE, orizzontale);
        coordinate[0].setAxisValue(ASSE_VERTICALE, verticale);
        long adesso = SystemClock.uptimeMillis();
        MotionEvent evento = MotionEvent.obtain(dita(display).ultimoGiu, adesso, SCROLL, 1, proprieta, coordinate, 0, 0,
                1f, 1f, 0, 0, SORGENTE_MOUSE, 0);
        inietta(evento, display);
        evento.recycle();
    }

    /** Un tasto: {@code azione} 0 giù, 1 su; {@code meta} i modificatori ({@code KeyEvent.META_*}). */
    static void tasto(int display, int azione, int codice, int ripetizione, int meta) {
        if (azione != GIU && azione != SU) {
            scartati++;
            return;
        }
        long adesso = SystemClock.uptimeMillis();
        inietta(new KeyEvent(adesso, adesso, azione, codice, ripetizione, meta, TASTIERA_VIRTUALE, 0, 0, SORGENTE_TASTIERA),
                display);
    }

    /**
     * Testo coi tasti della mappa virtuale, un carattere alla volta: un
     * carattere che la mappa non ha si salta (e si conta tra gli scartati).
     */
    static void testo(int display, String testo) {
        if (mappa == null) {
            mappa = KeyCharacterMap.load(TASTIERA_VIRTUALE);
        }
        for (int i = 0; i < testo.length(); i++) {
            KeyEvent[] eventi = mappa.getEvents(new char[] {testo.charAt(i)});
            if (eventi == null) {
                scartati++;
                annota(String.format("carattere U+%04X senza tasto nella mappa virtuale", (int) testo.charAt(i)));
                continue;
            }
            for (KeyEvent e : eventi) {
                inietta(e, display);
            }
        }
    }

    /**
     * «Indietro»: il tasto BACK. Solo per lo schermo principale spento (non
     * interattivo) il tasto al rilascio è POWER, per accenderlo, come fa oggi
     * scrcpy; gli schermi virtuali sono sempre accesi.
     */
    static void indietro(int display, int azione) {
        if (display != 0 || interattivo()) {
            tasto(display, azione, KEYCODE_BACK, 0, 0);
        } else if (azione == SU) {
            tasto(display, GIU, KEYCODE_POWER, 0, 0);
            tasto(display, SU, KEYCODE_POWER, 0, 0);
        }
    }

    private static boolean interattivo() {
        try {
            Object pm = Contesto.shell().getSystemService("power");
            return Boolean.TRUE.equals(pm.getClass().getMethod("isInteractive").invoke(pm));
        } catch (Exception e) {
            annota("stato dello schermo principale sconosciuto: " + Nascoste.causa(e));
            return true;
        }
    }

    // ------------------------------------------------------------------ appunti

    /**
     * Mette il testo negli appunti. Se ci sono già, messi da noi (etichetta
     * {@link Appunti#ETICHETTA}), non si riscrivono: niente doppioni nella
     * cronologia degli appunti. Un clip di un'altra app non si legge (niente
     * avviso «… ha incollato dagli appunti»): si sostituisce e basta.
     */
    static void scriviAppunti(String testo) throws Exception {
        try {
            Object d = Appunti.descrizione();
            if (d != null && Appunti.ETICHETTA.equals(Appunti.etichetta(d)) && testo.equals(Appunti.testo(Appunti.clip()))) {
                return;
            }
        } catch (Exception e) {
            annota("appunti attuali non letti: " + Nascoste.causa(e));
        }
        impostaNostro(Appunti.nuovo(Appunti.ETICHETTA, testo, false), testo);
    }

    /** Imposta un clip senza rimandarlo al PC come copia fatta sul telefono. */
    static void impostaNostro(Object clip, String testo) throws Exception {
        impostando = true;
        try {
            ultimoMesso = testo;
            ultimoMessoQuando = SystemClock.uptimeMillis();
            Appunti.imposta(clip);
        } finally {
            impostando = false;
        }
    }

    /**
     * {@code stato u8 · testo}: vuoti (o non di testo), testo, sensibili (senza
     * testo: una password non esce dal telefono), sconosciuti (senza testo: nel
     * dubbio non passa, come oggi). Prima la descrizione, che non fa comparire
     * l'avviso di accesso agli appunti, poi il testo.
     */
    private static byte[] statoAppunti(String[] testoLetto) {
        int stato;
        String testo = null;
        try {
            Object d = Appunti.descrizione();
            if (d == null) {
                stato = APPUNTI_VUOTI;
            } else if (Appunti.sensibile(d)) {
                stato = APPUNTI_SENSIBILI;
            } else {
                testo = Appunti.testo(Appunti.clip());
                stato = testo == null ? APPUNTI_VUOTI : APPUNTI_TESTO;
            }
        } catch (Exception e) {
            annota("appunti: " + Nascoste.causa(e));
            stato = APPUNTI_SCONOSCIUTI;
            testo = null;
        }
        if (testoLetto != null) {
            testoLetto[0] = testo;
        }
        byte[] t = testo == null ? new byte[0] : testo.getBytes(StandardCharsets.UTF_8);
        byte[] r = new byte[1 + t.length];
        r[0] = (byte) stato;
        System.arraycopy(t, 0, r, 1, t.length);
        return r;
    }

    private static void ascolta(boolean attivo) throws Exception {
        if (attivo && ascoltatore == null) {
            Ascoltatore a = new Ascoltatore();
            Appunti.ascolta(a);
            ascoltatore = a;
        } else if (!attivo && ascoltatore != null) {
            Appunti.smetti(ascoltatore);
            ascoltatore = null;
        }
    }

    /** Dal thread «input»: gli appunti sono cambiati; se non è opera nostra, si avvisa il PC. */
    private static void appuntiCambiati(boolean nostro) {
        if (nostro || ascoltatore == null) {
            return;
        }
        String[] testo = new String[1];
        byte[] stato = statoAppunti(testo);
        if (stato[0] == APPUNTI_VUOTI) {
            return;
        }
        // La notifica della nostra scrittura può arrivare dopo la fine di setPrimaryClip.
        if (testo[0] != null && testo[0].equals(ultimoMesso) && SystemClock.uptimeMillis() - ultimoMessoQuando < ECO_MS) {
            return;
        }
        // Samsung notifica ogni copia due volte (visto sul S23+, 28 set): lo
        // stesso avviso entro mezzo secondo è un doppione e non si rimanda.
        long adesso = SystemClock.uptimeMillis();
        if (java.util.Arrays.equals(stato, ultimoAvviso) && adesso - ultimoAvvisoQuando < DOPPIONE_MS) {
            return;
        }
        ultimoAvviso = stato;
        ultimoAvvisoQuando = adesso;
        cambiatiMandati++;
        Servizio.manda(APPUNTI_CAMBIATI, 0, 0, stato);
    }

    private static final long DOPPIONE_MS = 500;
    private static byte[] ultimoAvviso;
    private static long ultimoAvvisoQuando;

    /** L'ascoltatore degli appunti: chiamato su un thread del Binder, passa la palla al thread «input». */
    private static final class Ascoltatore extends IOnPrimaryClipChangedListener.Stub {
        @Override
        public void dispatchPrimaryClipChanged() {
            boolean nostro = impostando;
            CODA.execute(() -> appuntiCambiati(nostro));
        }
    }

    // ------------------------------------------------------------------ diagnosi

    private static String conteggi() {
        return "iniettati=" + iniettati + "\nfalliti=" + falliti + "\nscartati=" + scartati + "\navvisi_appunti="
                + cambiatiMandati + "\nascolto_appunti=" + (ascoltatore != null) + "\nultimo_errore="
                + ultimoErrore.replace('\n', ' ') + "\n";
    }

    /** Annota un errore: il primo e poi uno ogni 100 vanno nel log, l'ultimo resta nei conteggi. */
    private static void annota(String testo) {
        ultimoErrore = testo;
        if (annotati++ % 100 == 0) {
            System.err.println("phonestra-servizio: input: " + testo);
            System.err.flush();
        }
    }

    private static void errore(String dove, Throwable e) {
        falliti++;
        annota(dove + ": " + Nascoste.causa(e));
    }

    /**
     * Le dita appoggiate su uno schermo (al massimo {@link #MASSIMO}): ogni dito
     * del PC ha un numero locale 0–9, il più piccolo libero, che è l'id del
     * puntatore nell'evento Android. Ogni evento porta tutte le dita appoggiate;
     * quelle alzate si tolgono dopo averlo composto.
     */
    static final class Dita {
        static final int MASSIMO = 10;

        private static final class Dito {
            final long id;
            final int locale;
            float x;
            float y;
            float pressione;
            boolean su;

            Dito(long id, int locale) {
                this.id = id;
                this.locale = locale;
            }
        }

        /** Un evento da iniettare: azione Android, ora del primo «giù» del gesto, dita. */
        static final class Evento {
            int azione;
            long giu;
            int[] id;
            float[] x;
            float[] y;
            float[] pressione;
        }

        private final List<Dito> elenco = new ArrayList<>();
        long ultimoGiu;

        /**
         * Aggiorna il dito {@code id} e compone l'evento; {@code null} se ci sono
         * già 10 dita appoggiate e questo è nuovo.
         */
        Evento tocca(long id, int azione, float x, float y, float pressione, long adesso) {
            int indice = -1;
            for (int i = 0; i < elenco.size(); i++) {
                if (elenco.get(i).id == id) {
                    indice = i;
                }
            }
            if (indice < 0) {
                if (elenco.size() >= MASSIMO) {
                    return null;
                }
                elenco.add(new Dito(id, liberoPiuPiccolo()));
                indice = elenco.size() - 1;
            }
            Dito d = elenco.get(indice);
            d.x = x;
            d.y = y;
            d.pressione = pressione;
            d.su = azione == SU;
            int n = elenco.size();
            Evento e = new Evento();
            e.id = new int[n];
            e.x = new float[n];
            e.y = new float[n];
            e.pressione = new float[n];
            for (int i = 0; i < n; i++) {
                Dito p = elenco.get(i);
                e.id[i] = p.locale;
                e.x[i] = p.x;
                e.y[i] = p.y;
                e.pressione[i] = p.pressione;
            }
            if (n == 1) {
                if (azione == GIU) {
                    ultimoGiu = adesso;
                }
                e.azione = azione;
            } else if (azione == SU) {
                e.azione = POINTER_UP | indice << INDICE_SHIFT;
            } else if (azione == GIU) {
                e.azione = POINTER_DOWN | indice << INDICE_SHIFT;
            } else {
                e.azione = azione;
            }
            e.giu = ultimoGiu;
            elenco.removeIf(p -> p.su);
            return e;
        }

        int quante() {
            return elenco.size();
        }

        private int liberoPiuPiccolo() {
            for (int n = 0; ; n++) {
                boolean usato = false;
                for (Dito d : elenco) {
                    usato |= d.locale == n;
                }
                if (!usato) {
                    return n;
                }
            }
        }
    }
}
