package phonestra;

import android.app.ActivityManager;
import android.app.TaskStackListener;
import android.content.ComponentName;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArraySet;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.ScheduledFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Eventi delle app sugli schermi di Phonestra, al posto dei {@code dumpsys} che
 * oggi fa il PC (memoria/componente.md, «Video»). Messaggi {@link Video#EVENTO}:
 * <ul>
 *   <li>{@code evento=orientamento id display verticale=0|1 valore=N}: l'app in
 *       vista accetta solo il verticale (oggi {@code COMANDO_ORIENTAMENTI});
 *   <li>{@code evento=protetta id display protetta=0|1}: schermata protetta
 *       (oggi {@code dumpsys window}, FLAG_SECURE);
 *   <li>{@code evento=spostata id task display}: un task dello schermo è
 *       passato su un altro schermo (l'app aperta anche sul telefono);
 *   <li>{@code evento=rimosso id task}: task chiuso.
 * </ul>
 * Orientamento e schermata protetta si mandano al primo controllo e poi solo
 * quando cambiano.
 *
 * <p>Come: un {@code TaskStackListener} (classe nascosta che implementa già
 * tutti i metodi di {@code ITaskStackListener}, anche quelli aggiunti dai
 * produttori; prove §43) registrato finché c'è almeno una sessione. Ogni
 * evento programma un controllo 150 ms dopo (gli eventi arrivano a gruppi);
 * in più un controllo ogni 3 s, come il giro del PC di oggi: una finestra
 * protetta può comparire senza eventi dei task (stessa attività).
 */
final class EventiApp extends TaskStackListener {
    private static final long RITARDO_MS = 150;
    private static final long GIRO_MS = 3000;

    private static final ScheduledExecutorService ORARIO = Executors.newSingleThreadScheduledExecutor(r -> {
        Thread t = new Thread(r, "eventi video");
        t.setDaemon(true);
        return t;
    });
    private static final Set<SessioneVideo> SESSIONI = new CopyOnWriteArraySet<>();
    /** Orientamento chiesto a runtime da un task, con l'attività in cima quando è arrivato. */
    private static final Map<Integer, Chiesto> CHIESTI = new ConcurrentHashMap<>();
    private static final AtomicBoolean PROGRAMMATO = new AtomicBoolean();
    private static EventiApp ascoltatore;
    private static Object atm;
    private static ScheduledFuture<?> giro;

    private static final class Chiesto {
        final int valore;
        ComponentName attivita;

        Chiesto(int valore) {
            this.valore = valore;
        }
    }

    private EventiApp() {
    }

    static synchronized void aggiungi(SessioneVideo s) {
        SESSIONI.add(s);
        if (ascoltatore == null) {
            try {
                atm = Nascoste.activityTaskManager();
                EventiApp a = new EventiApp();
                Nascoste.invoca(atm, "registerTaskStackListener", a);
                ascoltatore = a;
            } catch (Exception e) {
                // Senza eventi restano i controlli ogni 3 s.
                Video.log("eventi dei task non disponibili: " + Nascoste.causa(e));
            }
        }
        if (giro == null) {
            giro = ORARIO.scheduleWithFixedDelay(EventiApp::controlla, GIRO_MS, GIRO_MS, TimeUnit.MILLISECONDS);
        }
        programma();
    }

    static synchronized void togli(SessioneVideo s) {
        SESSIONI.remove(s);
        if (!SESSIONI.isEmpty()) {
            return;
        }
        if (giro != null) {
            giro.cancel(false);
            giro = null;
        }
        if (ascoltatore != null) {
            try {
                Nascoste.invoca(atm, "unregisterTaskStackListener", ascoltatore);
            } catch (Exception e) {
                Video.log("ascoltatore dei task non tolto: " + Nascoste.causa(e));
            }
            ascoltatore = null;
        }
        CHIESTI.clear();
    }

    /** Un controllo fra {@link #RITARDO_MS}, se non ce n'è già uno in attesa. */
    private static void programma() {
        if (PROGRAMMATO.compareAndSet(false, true)) {
            ORARIO.schedule(() -> {
                PROGRAMMATO.set(false);
                controlla();
            }, RITARDO_MS, TimeUnit.MILLISECONDS);
        }
    }

    // ------------------------------------------------------------------ eventi (thread del binder)

    @Override
    public void onTaskStackChanged() {
        programma();
    }

    @Override
    public void onTaskCreated(int task, ComponentName componente) {
        programma();
    }

    @Override
    public void onTaskMovedToFront(ActivityManager.RunningTaskInfo info) {
        programma();
    }

    @Override
    public void onTaskFocusChanged(int task, boolean fuoco) {
        programma();
    }

    @Override
    public void onTaskRemoved(int task) {
        CHIESTI.remove(task);
        for (SessioneVideo s : SESSIONI) {
            if (s.task.remove(task)) {
                Video.evento(s, "rimosso", "task=" + task);
            }
        }
        programma();
    }

    @Override
    public void onTaskDisplayChanged(int task, int display) {
        for (SessioneVideo s : SESSIONI) {
            if (!s.specchio && display != s.display && s.task.remove(task)) {
                Video.evento(s, "spostata", "task=" + task, "display=" + display);
            }
        }
        programma();
    }

    @Override
    public void onTaskRequestedOrientationChanged(int task, int orientamento) {
        CHIESTI.put(task, new Chiesto(orientamento));
        programma();
    }

    @Override
    public void onActivityRequestedOrientationChanged(int task, int orientamento) {
        CHIESTI.put(task, new Chiesto(orientamento));
        programma();
    }

    // ------------------------------------------------------------------ controllo

    /** Solo verticale: PORTRAIT, SENSOR_PORTRAIT, REVERSE_PORTRAIT, USER_PORTRAIT. */
    static boolean verticale(int orientamento) {
        return orientamento == 1 || orientamento == 7 || orientamento == 9 || orientamento == 12;
    }

    private static void controlla() {
        for (SessioneVideo s : SESSIONI) {
            if (s.chiusa()) {
                continue;
            }
            if (!s.specchio) {
                try {
                    orientamento(s);
                } catch (Exception e) {
                    Video.log("orientamento dello schermo " + s.display + " non letto: " + Nascoste.causa(e));
                }
            }
            protetta(s);
        }
    }

    /**
     * I task dello schermo (in cima per primi) e l'orientamento dell'app in
     * vista: quello chiesto a runtime se l'attività in cima è ancora la stessa,
     * altrimenti quello del manifest ({@code topActivityInfo.screenOrientation}).
     */
    private static void orientamento(SessioneVideo s) throws Exception {
        List<?> radici = (List<?>) Nascoste.invoca(atm != null ? atm : Nascoste.activityTaskManager(),
                "getAllRootTaskInfosOnDisplay", s.display);
        Integer valore = null;
        for (Object r : radici) {
            Object id = Nascoste.campo(r, "taskId");
            if (!(id instanceof Integer)) {
                continue;
            }
            s.task.add((Integer) id);
            if (valore != null || !visibile(r)) {
                continue;
            }
            Object cima = Nascoste.campo(r, "topActivity");
            Chiesto c = CHIESTI.get(id);
            if (c != null && c.attivita == null) {
                c.attivita = cima instanceof ComponentName ? (ComponentName) cima : null;
            }
            if (c != null && Objects.equals(c.attivita, cima)) {
                valore = c.valore;
            } else {
                CHIESTI.remove(id);
                Object info = Nascoste.campo(r, "topActivityInfo");
                Object o = info != null ? Nascoste.campo(info, "screenOrientation") : null;
                valore = o instanceof Integer ? (Integer) o : -1;
            }
        }
        if (valore == null) {
            return; // niente in vista (ancora): come oggi, nessuna risposta
        }
        Integer prima = s.orientamentoMandato;
        if (prima == null || verticale(prima) != verticale(valore)) {
            s.orientamentoMandato = valore;
            Video.evento(s, "orientamento", "display=" + s.display, "verticale=" + (verticale(valore) ? 1 : 0),
                    "valore=" + valore);
        } else {
            s.orientamentoMandato = valore;
        }
    }

    private static boolean visibile(Object info) {
        for (String campo : new String[] {"isVisible", "visible"}) {
            Object v = Nascoste.campo(info, campo);
            if (v instanceof Boolean) {
                return (Boolean) v;
            }
        }
        return true;
    }

    /** Controllo delle schermate protette acceso (spegnibile per le prove, §48). */
    static volatile boolean controllaProtetta = true;

    private static void protetta(SessioneVideo s) {
        if (!controllaProtetta) {
            return;
        }
        boolean p;
        try {
            p = Protetta.presente(s.display);
        } catch (Exception e) {
            if (!s.protettaInErrore) {
                s.protettaInErrore = true;
                Video.log("schermata protetta non controllabile sullo schermo " + s.display + ": " + Nascoste.causa(e));
            }
            return;
        }
        s.protettaInErrore = false;
        if (s.protettaMandata == null || s.protettaMandata != p) {
            s.protettaMandata = p;
            Video.evento(s, "protetta", "display=" + s.display, "protetta=" + (p ? 1 : 0));
        }
    }
}
