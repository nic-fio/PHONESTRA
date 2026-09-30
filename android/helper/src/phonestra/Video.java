package phonestra;

import android.net.LocalSocket;

import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * Il pezzo «video» del servizio (notes/component.md, «Video»): schermi
 * virtuali per le app, specchio dello schermo principale, codifica, eventi
 * delle app, schermate protette, pannello: apertura degli schermi, avvio
 * delle app, ridimensionamento, fotogrammi chiave, pannello fisico, e gli
 * eventi di orientamento e schermate protette per il PC.
 *
 * <p>Messaggi del canale comandi (0x40–0x4f, contenuto: righe
 * {@code chiave=valore}); ogni domanda ha una risposta con lo stesso id
 * (o {@code ERRORE}). Le domande si eseguono in ordine su un thread
 * {@code video}, così il canale comandi non aspetta mai il video.
 *
 * <p>Canale {@code video:<id>}: aperto dal PC dopo APRI; il servizio ci scrive
 * i pacchetti ({@link SessioneVideo}). Se il PC lo chiude, la sessione si
 * chiude (senza togliere l'app dalle recenti, come oggi quando cade il
 * collegamento).
 */
final class Video {
    /** PC → servizio: nuova sessione. Risposta: {@code id display codec larghezza altezza}. */
    static final int APRI = 0x40;
    /** PC → servizio: {@code id [togli_task=1]}; chiude la sessione. */
    static final int CHIUDI = 0x41;
    /** PC → servizio: {@code id} e {@code app=<pacchetto>} o {@code informazioni=<pacchetto>}. */
    static final int AVVIA_APP = 0x42;
    /** PC → servizio: {@code id larghezza altezza}. */
    static final int RIDIMENSIONA = 0x43;
    /** PC → servizio: {@code id}; fotogramma chiave appena possibile. */
    static final int CHIAVE = 0x44;
    /** PC → servizio: {@code acceso=0|1}; pannello fisico del telefono. */
    static final int PANNELLO = 0x45;
    /** Servizio → PC, spontaneo: {@code evento=… id=…} (vedi {@link EventiApp}). */
    static final int EVENTO = 0x46;

    /** Senza canale video entro questo tempo, la sessione aperta si chiude. */
    static final long ATTESA_CANALE_MS = 10_000;

    private static final ExecutorService ESECUTORE = Executors.newSingleThreadExecutor(r -> {
        Thread t = new Thread(r, "video");
        t.setDaemon(true);
        return t;
    });
    private static final Map<Integer, SessioneVideo> SESSIONI = new ConcurrentHashMap<>();
    private static final AtomicInteger PROSSIMO = new AtomicInteger(1);

    private Video() {
    }

    /** Un messaggio video dal canale comandi: eseguito sul thread del video, poi la risposta. */
    static void comando(Protocollo.Messaggio m) {
        ESECUTORE.execute(() -> {
            try {
                String risposta = esegui(m.tipo, opzioni(m.testo()));
                Servizio.manda(m.tipo, Protocollo.RISPOSTA, m.id, risposta.getBytes(StandardCharsets.UTF_8));
            } catch (Throwable e) {
                log("messaggio 0x" + Integer.toHexString(m.tipo) + " non riuscito: " + Nascoste.causa(e));
                Servizio.manda(Protocollo.ERRORE, Protocollo.RISPOSTA, m.id,
                        Nascoste.causa(e).getBytes(StandardCharsets.UTF_8));
            }
        });
    }

    /** Righe (o parole) {@code chiave=valore}. */
    static Map<String, String> opzioni(String testo) {
        Map<String, String> o = new HashMap<>();
        for (String voce : testo.split("\\s+")) {
            int i = voce.indexOf('=');
            if (i > 0) {
                o.put(voce.substring(0, i), voce.substring(i + 1));
            }
        }
        return o;
    }

    private static SessioneVideo sessione(Map<String, String> o) {
        SessioneVideo s = SESSIONI.get(SessioneVideo.intero(o, "id", -1));
        if (s == null) {
            throw new IllegalArgumentException("sessione video sconosciuta: " + o.get("id"));
        }
        return s;
    }

    private static String esegui(int tipo, Map<String, String> o) throws Exception {
        switch (tipo) {
            case APRI:
                return apri(o);
            case CHIUDI:
                chiudi(SessioneVideo.intero(o, "id", -1), "1".equals(o.get("togli_task")), null);
                return "";
            case AVVIA_APP: {
                SessioneVideo s = sessione(o);
                if (o.containsKey("informazioni")) {
                    return s.avviaInformazioni(o.get("informazioni"));
                }
                return s.avviaApp(o.getOrDefault("app", ""));
            }
            case RIDIMENSIONA:
                return sessione(o).cambiaMisura(SessioneVideo.intero(o, "larghezza", 0),
                        SessioneVideo.intero(o, "altezza", 0));
            case CHIAVE:
                sessione(o).chiave();
                return "";
            case PANNELLO:
                return "schermi=" + Pannello.imposta(!"0".equals(o.get("acceso")));
            default:
                throw new IllegalArgumentException(String.format("tipo video sconosciuto 0x%02x", tipo));
        }
    }

    private static String apri(Map<String, String> o) throws Exception {
        // Interruttori di prova (prove §48): valgono per tutto il servizio.
        if (o.containsKey("max_fps")) {
            Codifica.fotogrammiMassimi = SessioneVideo.intero(o, "max_fps", 0);
        }
        if (o.containsKey("priorita")) {
            Codifica.priorita = SessioneVideo.intero(o, "priorita", 0);
        }
        if (o.containsKey("protetta")) {
            EventiApp.controllaProtetta = !"0".equals(o.get("protetta"));
        }
        int id = PROSSIMO.getAndIncrement();
        SessioneVideo s = SessioneVideo.apri(id, o);
        SESSIONI.put(id, s);
        String avvio = "";
        try {
            if (o.containsKey("app")) {
                avvio = s.avviaApp(o.get("app"));
            } else if (o.containsKey("informazioni")) {
                avvio = s.avviaInformazioni(o.get("informazioni"));
            }
        } catch (Exception e) {
            chiudi(id, false, null);
            throw e;
        }
        EventiApp.aggiungi(s);
        // Il PC deve aprire il canale subito dopo la risposta; altrimenti si chiude.
        Thread t = new Thread(() -> {
            Sistema.attendi(ATTESA_CANALE_MS);
            if (!s.collegata() && !s.chiusa()) {
                chiudiPiuTardi(id, "nessun canale video entro " + ATTESA_CANALE_MS / 1000 + " s");
            }
        }, "attesa canale " + id);
        t.setDaemon(true);
        t.start();
        return s.descrizione() + (avvio.isEmpty() ? "" : "avvio=" + avvio.replace('\n', ' ') + "\n");
    }

    /**
     * Chiude la sessione {@code id} (se c'è ancora); con {@code motivo} il PC
     * riceve l'evento «fine» (chiusura decisa dal telefono).
     */
    static void chiudi(int id, boolean togliTask, String motivo) {
        SessioneVideo s = SESSIONI.remove(id);
        if (s == null) {
            return;
        }
        EventiApp.togli(s);
        s.chiudi(togliTask);
        if (motivo != null) {
            log("sessione video " + id + " chiusa: " + motivo);
            evento(s, "fine", "motivo=" + motivo.replace('\n', ' '));
        }
    }

    /** Come {@link #chiudi}, sul thread del video (da un thread che non deve bloccarsi). */
    static void chiudiPiuTardi(int id, String motivo) {
        ESECUTORE.execute(() -> chiudi(id, false, motivo));
    }

    /**
     * Gestore del canale {@code video:<id>}: collega il canale alla sessione e
     * aspetta che il PC lo chiuda (il PC non ci scrive niente).
     */
    static void canale(LocalSocket socket, String tipo) throws Exception {
        int id;
        try {
            id = Integer.parseInt(tipo.substring(tipo.indexOf(':') + 1));
        } catch (NumberFormatException e) {
            socket.close();
            return;
        }
        SessioneVideo s = SESSIONI.get(id);
        if (s == null) {
            log("canale video per la sessione sconosciuta " + id);
            socket.close();
            return;
        }
        try {
            s.collega(socket);
        } catch (Exception e) {
            chiudi(id, false, "codifica non avviata: " + Nascoste.causa(e));
            return;
        }
        InputStream in = socket.getInputStream();
        byte[] b = new byte[64];
        try {
            while (in.read(b) >= 0) {
                // niente da leggere: si aspetta la chiusura
            }
        } catch (Exception e) {
            // canale interrotto: come chiuso
        }
        chiudi(id, false, null);
    }

    /** Evento spontaneo al PC: {@code evento=<nome> id=<id>} e le altre coppie, una per riga. */
    static void evento(SessioneVideo s, String nome, String... coppie) {
        StringBuilder t = new StringBuilder("evento=").append(nome).append("\nid=").append(s.id).append('\n');
        for (String c : coppie) {
            t.append(c).append('\n');
        }
        Servizio.manda(EVENTO, 0, 0, t.toString().getBytes(StandardCharsets.UTF_8));
    }

    static void log(String testo) {
        System.err.println("phonestra-servizio: video: " + testo);
        System.err.flush();
    }
}
