package phonestra;

import android.content.Context;
import android.content.pm.PackageManager;

import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Autotest all'avvio del servizio (notes/component.md §6): per ogni API
 * nascosta di cui i pezzi futuri avranno bisogno, controlla che il telefono
 * l'abbia e con quale firma, <b>senza usarla</b> (nessuno schermo creato,
 * nessun evento iniettato, nessuna politica audio registrata). L'esito va al
 * PC nel {@code CIAO}: il PC spegne le funzioni che mancano o usa il ripiego.
 *
 * <p>Ogni voce vale {@code ok <dettagli>} oppure {@code manca <causa>}.
 */
final class Autotest {
    /** Una prova: restituisce i dettagli se la funzione c'è, lancia se manca. */
    private interface Prova {
        String esegui() throws Exception;
    }

    /** Permessi della shell che servono ai pezzi del componente (api-android.md). */
    private static final String[] PERMESSI = {
        "android.permission.CAPTURE_AUDIO_OUTPUT",
        "android.permission.MODIFY_AUDIO_ROUTING",
        "android.permission.CAPTURE_VIDEO_OUTPUT",
        "android.permission.ADD_TRUSTED_DISPLAY",
        "android.permission.ADD_ALWAYS_UNLOCKED_DISPLAY",
        "android.permission.INJECT_EVENTS",
        "android.permission.MANAGE_ACTIVITY_TASKS",
        "android.permission.READ_CLIPBOARD_IN_BACKGROUND",
        "android.permission.WRITE_SETTINGS",
    };

    private Autotest() {
    }

    /** Tutte le prove, nell'ordine; chiavi senza spazi (vanno nel CIAO come {@code autotest.<chiave>}). */
    static Map<String, String> esegui() {
        Map<String, String> esiti = new LinkedHashMap<>();
        prova(esiti, "contesto", Autotest::contesto);
        prova(esiti, "permessi", Autotest::permessi);
        prova(esiti, "display_manager", Autotest::displayManager);
        prova(esiti, "capture_display", Autotest::captureDisplay);
        prova(esiti, "task_stack_listener", Autotest::taskStackListener);
        prova(esiti, "inject_input_event", Autotest::injectInputEvent);
        prova(esiti, "audio_policy", Autotest::audioPolicy);
        prova(esiti, "appunti", Autotest::appunti);
        return esiti;
    }

    private static void prova(Map<String, String> esiti, String nome, Prova p) {
        String esito;
        try {
            String dettagli = p.esegui();
            esito = "ok" + (dettagli.isEmpty() ? "" : " " + dettagli);
        } catch (Throwable e) {
            esito = "manca " + Nascoste.causa(e);
        }
        // Un valore per riga nel CIAO.
        esiti.put(nome, esito.replace('\n', ' ').replace('\r', ' '));
    }

    /** Contesto della shell preparato, col pacchetto giusto. */
    private static String contesto() throws Exception {
        Context c = Contesto.shell();
        String dettagli = "pacchetto=" + c.getPackageName();
        List<String> note = Contesto.note();
        if (!note.isEmpty()) {
            dettagli += " note=" + String.join("; ", note).replace(' ', '_');
        }
        if (!Contesto.SHELL.equals(c.getPackageName())) {
            throw new IllegalStateException("pacchetto del contesto: " + c.getPackageName());
        }
        return dettagli;
    }

    /** Permessi che la shell ha davvero su questo telefono (Samsung può toglierne). */
    private static String permessi() throws Exception {
        PackageManager pm = Contesto.shell().getPackageManager();
        List<String> mancano = new ArrayList<>();
        for (String p : PERMESSI) {
            if (pm.checkPermission(p, Contesto.SHELL) != PackageManager.PERMISSION_GRANTED) {
                mancano.add(p.substring(p.lastIndexOf('.') + 1));
            }
        }
        if (!mancano.isEmpty()) {
            throw new IllegalStateException("senza " + String.join(",", mancano));
        }
        return PERMESSI.length + "/" + PERMESSI.length;
    }

    /** IDisplayManager.createVirtualDisplay (schermi virtuali col nostro contesto). */
    private static String displayManager() throws Exception {
        Object dm = Nascoste.displayManager();
        Nascoste.metodo(dm.getClass(), "createVirtualDisplay", 4, 5);
        Class.forName("android.hardware.display.DisplayManagerGlobal").getMethod("getInstance");
        return "createVirtualDisplay=" + Nascoste.firme(dm.getClass(), "createVirtualDisplay");
    }

    /** IWindowManager.captureDisplay(int, argomenti, ascoltatore): schermate protette senza dumpsys. */
    private static String captureDisplay() throws Exception {
        Object wm = Nascoste.windowManager();
        for (Method m : Nascoste.metodi(wm.getClass(), "captureDisplay")) {
            Class<?>[] t = m.getParameterTypes();
            if (t.length == 3 && t[0] == int.class) {
                return "argomenti=" + t[1].getName().substring(t[1].getName().lastIndexOf('.') + 1);
            }
        }
        throw new NoSuchMethodException("captureDisplay(int, …, …) (varianti: " + Nascoste.firme(wm.getClass(), "captureDisplay") + ")");
    }

    /** Eventi dei task: classe TaskStackListener e registerTaskStackListener. */
    private static String taskStackListener() throws Exception {
        Class.forName("android.app.TaskStackListener");
        Object atm = Nascoste.activityTaskManager();
        Nascoste.metodo(atm.getClass(), "registerTaskStackListener", 1);
        Nascoste.metodo(atm.getClass(), "unregisterTaskStackListener", 1);
        return "";
    }

    /** Iniezione di tocchi e tasti: injectInputEvent(evento, modo[, uid]). */
    private static String injectInputEvent() throws Exception {
        Object im = Nascoste.inputManager();
        Method m = Nascoste.metodo(im.getClass(), "injectInputEvent", 2, 3);
        return im.getClass().getSimpleName() + "/" + m.getParameterTypes().length
                + " setDisplayId=" + (Nascoste.metodi(Class.forName("android.view.InputEvent"), "setDisplayId").isEmpty() ? "no" : "si");
    }

    /** Cattura dell'audio in loopback: le classi di AudioPolicy e la registrazione. */
    private static String audioPolicy() throws Exception {
        String base = "android.media.audiopolicy.";
        Class<?> regola = Class.forName(base + "AudioMixingRule$Builder");
        Class.forName(base + "AudioMix$Builder");
        Class<?> politica = Class.forName(base + "AudioPolicy");
        Class.forName(base + "AudioPolicy$Builder");
        Nascoste.metodo(politica, "createAudioRecordSink", 1);
        Nascoste.metodo(regola, "setTargetMixRole", 1);
        Class<?> am = Class.forName("android.media.AudioManager");
        String registrazione;
        if (!Nascoste.metodi(am, "registerAudioPolicy").isEmpty()) {
            registrazione = "istanza";
        } else {
            Method statica = null;
            for (Method m : am.getDeclaredMethods()) {
                if (m.getName().equals("registerAudioPolicyStatic")) {
                    statica = m;
                }
            }
            if (statica == null) {
                throw new NoSuchMethodException("AudioManager.registerAudioPolicy");
            }
            registrazione = "statica";
        }
        return "registrazione=" + registrazione;
    }

    /** IClipboard: lettura, descrizione (segno «sensibile») e ascoltatore. */
    private static String appunti() throws Exception {
        Object c = Nascoste.appunti();
        Class<?> k = c.getClass();
        for (String nome : new String[] {"getPrimaryClip", "getPrimaryClipDescription", "addPrimaryClipChangedListener"}) {
            if (Nascoste.metodi(k, nome).isEmpty()) {
                throw new NoSuchMethodException("IClipboard." + nome);
            }
        }
        return "getPrimaryClip=" + Nascoste.firme(k, "getPrimaryClip")
                + " addPrimaryClipChangedListener=" + Nascoste.firme(k, "addPrimaryClipChangedListener")
                + " semclipboard=" + (Nascoste.esiste("semclipboard") ? "si" : "no");
    }
}
