package phonestra;

import android.content.Context;
import android.os.Looper;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.List;

/**
 * Il contesto Android del processo. {@code app_process} non è un'app: nessuno
 * ha creato l'{@code ActivityThread} né il contesto, e molti servizi di sistema
 * (DisplayManager, AudioManager, PackageManager…) li vogliono.
 *
 * <p>Si prepara una volta sola (le chiamate successive restituiscono lo stesso
 * oggetto), un passo alla volta: un passo che non riesce viene annotato in
 * {@link #note()} e, se non è indispensabile, non ferma gli altri.
 *
 * <ol>
 *   <li>Looper principale (alcuni gestori lo cercano anche se non gira);
 *   <li>un {@code ActivityThread} di sistema, impostato come quello corrente;
 *   <li>il suo {@code ConfigurationController}: senza, su Samsung
 *       {@code DisplayManagerGlobal} va in NullPointerException (problema noto,
 *       scrcpy #4467);
 *   <li>il contesto di sistema e, da questo, quello del pacchetto
 *       {@code com.android.shell}: Android 16 controlla che il pacchetto
 *       dichiarato sia quello dell'uid (2000), e i permessi della shell
 *       (cattura dell'audio, schermi fidati…) valgono solo così.
 * </ol>
 *
 * Codice nostro; scrcpy è servito solo come documentazione dei problemi noti.
 */
final class Contesto {
    static final String SHELL = "com.android.shell";

    private static Context sistema;
    private static Context shell;
    private static final List<String> NOTE = new ArrayList<>();

    private Contesto() {
    }

    /** Contesto di sistema (preparato la prima volta). */
    static synchronized Context sistema() throws Exception {
        if (sistema != null) {
            return sistema;
        }
        if (Looper.getMainLooper() == null) {
            Looper.prepareMainLooper();
        }
        Class<?> classe = Class.forName("android.app.ActivityThread");
        Constructor<?> costruttore = classe.getDeclaredConstructor();
        costruttore.setAccessible(true);
        Object thread = costruttore.newInstance();
        imposta(classe, null, "sCurrentActivityThread", thread);
        Field sistemaF = classe.getDeclaredField("mSystemThread");
        sistemaF.setAccessible(true);
        sistemaF.setBoolean(thread, true);
        try {
            Class<?> controllore = Class.forName("android.app.ConfigurationController");
            Constructor<?> c = controllore.getDeclaredConstructor(Class.forName("android.app.ActivityThreadInternal"));
            c.setAccessible(true);
            imposta(classe, thread, "mConfigurationController", c.newInstance(thread));
        } catch (Exception e) {
            NOTE.add("ConfigurationController non impostato: " + Nascoste.causa(e));
        }
        sistema = (Context) classe.getDeclaredMethod("getSystemContext").invoke(thread);
        return sistema;
    }

    /** Contesto del pacchetto della shell, quello da usare per i servizi. */
    static synchronized Context shell() throws Exception {
        if (shell == null) {
            Context s = sistema();
            shell = (Context) s.getClass().getMethod("createPackageContext", String.class, int.class).invoke(s, SHELL, 0);
        }
        return shell;
    }

    /** Passi non riusciti durante la preparazione (vuoto se è andato tutto bene). */
    static synchronized List<String> note() {
        return new ArrayList<>(NOTE);
    }

    private static void imposta(Class<?> classe, Object oggetto, String nome, Object valore) throws Exception {
        Field f = classe.getDeclaredField(nome);
        f.setAccessible(true);
        f.set(oggetto, valore);
    }
}
