// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

package phonestra;

import android.media.Image;
import android.media.ImageReader;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Comandi delle prove del modulo input ({@code phonestra-prova input-componente},
 * messaggio {@link Input#PROVA}): uno schermo virtuale con un'app nota su cui
 * iniettare, la sua «firma» (immagine rimpicciolita, per vedere se un evento ha
 * cambiato qualcosa), il salvataggio degli appunti dell'utente e una copia
 * «esterna» per provare l'avviso delle copie. Lo schermo di prova si crea come
 * nelle prove del video ({@link Sistema#creaSchermo}); il modulo video non
 * c'entra.
 *
 * <p>Il contenuto è una riga di testo, comando e argomenti separati da spazi:
 * <ul>
 *   <li>{@code apri <larghezza> <altezza> <dpi>} → {@code display=<id>};
 *   <li>{@code avvia <display> <pacchetto>}: l'app principale del pacchetto;
 *   <li>{@code azione <display> <azione> [pacchetto]}: {@code am start --display … -a … [-p …]};
 *   <li>{@code firma <display>} → {@code colonne u8 · righe u8 · luminosità u8…};
 *   <li>{@code chiudi <display>}: task tolti, schermo chiuso;
 *   <li>{@code salva-appunti} / {@code ripristina-appunti}: il clip dell'utente
 *       resta nella memoria del servizio, non va al PC;
 *   <li>{@code esterno <0|1> <testo>}: un clip messo «da un'altra app» (1 =
 *       segnato come sensibile), per l'ascoltatore.
 * </ul>
 * Nessun comando generico: pacchetti e azioni solo con lettere, cifre, punti e
 * trattini bassi. Se il servizio muore, il custode toglie i task delle app
 * avviate (lo schermo sparisce da solo con il processo).
 */
final class InputProva {
    private static final int COLONNE = 32;
    private static final int RIGHE = 56;

    private static final class Schermo {
        final Pulizia pulizia = new Pulizia();
        Sistema.Schermo schermo;
        Fotogrammi fotogrammi;
    }

    private static final Map<Integer, Schermo> SCHERMI = new HashMap<>();
    private static Object appuntiSalvati;
    private static boolean salvati;

    private InputProva() {
    }

    static void esegui(Protocollo.Messaggio m) {
        try {
            byte[] r = comando(m.testo().trim());
            Servizio.manda(Input.PROVA, Protocollo.RISPOSTA, m.id, r);
        } catch (Throwable e) {
            Servizio.manda(Protocollo.ERRORE, Protocollo.RISPOSTA, m.id, Nascoste.causa(e).getBytes(StandardCharsets.UTF_8));
        }
    }

    private static byte[] comando(String riga) throws Exception {
        String[] p = riga.split(" +");
        switch (p[0]) {
            case "apri":
                return testo(apri(intero(p, 1), intero(p, 2), intero(p, 3)));
            case "avvia":
                return testo(avvia(intero(p, 1), nome(p, 2), false));
            case "azione":
                return testo(avvia(intero(p, 1), nome(p, 2) + (p.length > 3 ? " -p " + nome(p, 3) : ""), true));
            case "firma":
                return firma(schermo(intero(p, 1)));
            case "chiudi":
                return testo(chiudi(intero(p, 1)));
            case "salva-appunti":
                return testo(salvaAppunti());
            case "ripristina-appunti":
                return testo(ripristinaAppunti());
            case "esterno": {
                String[] q = riga.split(" ", 3);
                String t = q.length > 2 ? q[2] : "";
                Appunti.imposta(Appunti.nuovo("prova esterna", t, q.length > 1 && "1".equals(q[1])));
                return testo("messo");
            }
            default:
                throw new IllegalArgumentException("comando di prova sconosciuto: " + p[0]);
        }
    }

    private static byte[] testo(String s) {
        return s.getBytes(StandardCharsets.UTF_8);
    }

    private static int intero(String[] p, int i) {
        if (p.length <= i) {
            throw new IllegalArgumentException("argomento mancante");
        }
        return Integer.parseInt(p[i]);
    }

    private static String nome(String[] p, int i) {
        String n = p.length > i ? p[i] : "";
        if (!n.matches("[A-Za-z0-9_.]+")) {
            throw new IllegalArgumentException("nome non valido: «" + n + "»");
        }
        return n;
    }

    private static synchronized Schermo schermo(int id) {
        Schermo s = SCHERMI.get(id);
        if (s == null) {
            throw new IllegalArgumentException("nessuno schermo di prova " + id);
        }
        return s;
    }

    private static String apri(int l, int a, int dpi) throws Exception {
        if (l < 64 || a < 64 || l > 4096 || a > 4096 || dpi < 72 || dpi > 800) {
            throw new IllegalArgumentException("misura non valida");
        }
        Schermo s = new Schermo();
        try {
            s.fotogrammi = new Fotogrammi(l, a);
            s.pulizia.aggiungi("lettore di immagini", s.fotogrammi::close);
            s.schermo = Sistema.creaSchermo(s.pulizia, "phonestra-prova-input", l, a, dpi, s.fotogrammi.superficie(),
                    Sistema.FLAG_PROPOSTI);
        } catch (Exception e) {
            s.pulizia.esegui();
            throw e;
        }
        synchronized (InputProva.class) {
            SCHERMI.put(s.schermo.id, s);
        }
        return "display=" + s.schermo.id;
    }

    /** L'app principale di un pacchetto, o con {@code azione} l'argomento di {@code am start -a} già controllato. */
    private static String avvia(int id, String nome, boolean azione) throws Exception {
        Schermo s = schermo(id);
        String esito = azione
                ? "am start --display " + id + " -a " + nome + " → " + Sistema.esegui("am start --display " + id + " -a " + nome).replace('\n', ' ')
                : Sistema.avviaApp(id, nome, null);
        // I task nati sullo schermo: se il servizio muore, li toglie il custode
        // (su Samsung altrimenti finirebbero sullo schermo del telefono).
        Sistema.attendi(1500);
        List<Integer> task = taskSulloSchermo(id);
        if (!task.isEmpty()) {
            StringBuilder c = new StringBuilder();
            for (int t : task) {
                c.append(c.length() > 0 ? "; " : "").append("am stack remove ").append(t);
            }
            Servizio.custode().imposta("input-prova-task-" + id, 500, c.toString());
        }
        s.pulizia.aggiungi("custode dei task dello schermo " + id, () -> Servizio.custode().togli("input-prova-task-" + id));
        return esito + " (task " + task + ")";
    }

    private static List<Integer> taskSulloSchermo(int id) {
        List<Integer> task = new ArrayList<>();
        try {
            Object atm = Nascoste.activityTaskManager();
            for (Object r : (List<?>) Nascoste.invoca(atm, "getAllRootTaskInfosOnDisplay", id)) {
                Object t = Nascoste.campo(r, "taskId");
                if (t instanceof Integer) {
                    task.add((Integer) t);
                }
            }
        } catch (Exception e) {
            System.err.println("phonestra-servizio: input: task dello schermo " + id + " non letti: " + Nascoste.causa(e));
        }
        return task;
    }

    private static String chiudi(int id) {
        Schermo s;
        synchronized (InputProva.class) {
            s = SCHERMI.remove(id);
        }
        if (s == null) {
            throw new IllegalArgumentException("nessuno schermo di prova " + id);
        }
        // Prima i task (e la voce del custode, registrata per ultima), poi lo schermo.
        s.pulizia.esegui();
        Input.dimentica(id);
        return "chiuso";
    }

    private static String salvaAppunti() throws Exception {
        appuntiSalvati = Appunti.clip();
        salvati = true;
        return appuntiSalvati == null ? "vuoti" : "salvati";
    }

    private static String ripristinaAppunti() throws Exception {
        if (!salvati) {
            return "niente da ripristinare";
        }
        if (appuntiSalvati == null) {
            Appunti.svuota();
        } else {
            Input.impostaNostro(appuntiSalvati, null);
        }
        salvati = false;
        appuntiSalvati = null;
        return "ripristinati";
    }

    /** Luminosità media di {@link #COLONNE}×{@link #RIGHE} riquadri dell'ultima immagine. */
    private static byte[] firma(Schermo s) throws Exception {
        int[] lum = s.fotogrammi.firma(COLONNE, RIGHE);
        byte[] r = new byte[2 + lum.length];
        r[0] = (byte) COLONNE;
        r[1] = (byte) RIGHE;
        for (int i = 0; i < lum.length; i++) {
            r[2 + i] = (byte) lum[i];
        }
        return r;
    }

    /** Le immagini dello schermo di prova: tiene l'ultima e scarta le altre (la coda piena fermerebbe chi disegna). */
    private static final class Fotogrammi implements AutoCloseable {
        private final ImageReader lettore;
        private Image ultima;
        private volatile boolean chiuso;

        Fotogrammi(int l, int a) {
            // 1 = PixelFormat.RGBA_8888
            lettore = ImageReader.newInstance(l, a, 1, 3);
            Thread svuota = new Thread(() -> {
                while (!chiuso) {
                    synchronized (this) {
                        if (!chiuso) {
                            aggiorna();
                        }
                    }
                    Sistema.attendi(50);
                }
            }, "input-prova-immagini");
            svuota.setDaemon(true);
            svuota.start();
        }

        android.view.Surface superficie() {
            return lettore.getSurface();
        }

        private Image aggiorna() {
            Image nuova = lettore.acquireLatestImage();
            if (nuova != null) {
                if (ultima != null) {
                    ultima.close();
                }
                ultima = nuova;
            }
            return ultima;
        }

        synchronized int[] firma(int colonne, int righe) {
            Image img = aggiorna();
            if (img == null) {
                throw new IllegalStateException("nessuna immagine dallo schermo di prova");
            }
            Image.Plane piano = img.getPlanes()[0];
            ByteBuffer b = piano.getBuffer();
            int passoPixel = piano.getPixelStride();
            int passoRiga = piano.getRowStride();
            int l = img.getWidth();
            int a = img.getHeight();
            long[] somma = new long[colonne * righe];
            int[] quanti = new int[colonne * righe];
            for (int y = 0; y < a; y += 2) {
                int riga = y * righe / a;
                for (int x = 0; x < l; x += 2) {
                    int i = y * passoRiga + x * passoPixel;
                    int lum = (299 * (b.get(i) & 0xff) + 587 * (b.get(i + 1) & 0xff) + 114 * (b.get(i + 2) & 0xff)) / 1000;
                    int cella = riga * colonne + x * colonne / l;
                    somma[cella] += lum;
                    quanti[cella]++;
                }
            }
            int[] r = new int[colonne * righe];
            for (int i = 0; i < r.length; i++) {
                r[i] = quanti[i] == 0 ? 0 : (int) (somma[i] / quanti[i]);
            }
            return r;
        }

        @Override
        public synchronized void close() {
            chiuso = true;
            if (ultima != null) {
                ultima.close();
                ultima = null;
            }
            lettore.close();
        }
    }
}
