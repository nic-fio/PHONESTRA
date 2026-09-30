package phonestra;

import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;

/**
 * Adattatori per le API nascoste di Android (notes/study/system.md §4).
 *
 * <p>Regola: si guarda cosa c'è sul telefono, non la versione. Ogni servizio si
 * prende con {@code ServiceManager.getService} + {@code Stub.asInterface} (o col
 * metodo statico pubblico, dove c'è), i metodi si cercano per nome e numero di
 * parametri tra le varianti note, e un errore spegne solo la funzione che ne
 * dipende ({@link Autotest}), mai il servizio intero.
 *
 * <p>Nelle app queste chiamate sarebbero bloccate; in {@code app_process} la
 * politica delle API nascoste è spenta (sistema.md §4.1).
 */
final class Nascoste {
    private Nascoste() {
    }

    /** Interfaccia AIDL di un servizio di sistema: ServiceManager + Stub.asInterface. */
    static Object servizio(String nome, String interfaccia) throws Exception {
        Object binder = Class.forName("android.os.ServiceManager").getMethod("getService", String.class).invoke(null, nome);
        if (binder == null) {
            throw new IllegalStateException("servizio «" + nome + "» assente");
        }
        return Class.forName(interfaccia + "$Stub").getMethod("asInterface", Class.forName("android.os.IBinder"))
                .invoke(null, binder);
    }

    /** Se il servizio di sistema con questo nome esiste (senza usarlo). */
    static boolean esiste(String nome) {
        try {
            return Class.forName("android.os.ServiceManager").getMethod("checkService", String.class).invoke(null, nome) != null;
        } catch (Exception e) {
            return false;
        }
    }

    static Object activityTaskManager() throws Exception {
        try {
            return Class.forName("android.app.ActivityTaskManager").getMethod("getService").invoke(null);
        } catch (Exception e) {
            return servizio("activity_task", "android.app.IActivityTaskManager");
        }
    }

    static Object activityManager() throws Exception {
        try {
            return Class.forName("android.app.ActivityManager").getMethod("getService").invoke(null);
        } catch (Exception e) {
            return servizio("activity", "android.app.IActivityManager");
        }
    }

    static Object windowManager() throws Exception {
        return servizio("window", "android.view.IWindowManager");
    }

    static Object displayManager() throws Exception {
        return servizio("display", "android.hardware.display.IDisplayManager");
    }

    static Object appunti() throws Exception {
        return servizio("clipboard", "android.content.IClipboard");
    }

    /**
     * Chi inietta tocchi e tasti: {@code InputManagerGlobal.getInstance()} (Android
     * 14+), in ripiego il vecchio {@code InputManager.getInstance()}.
     */
    static Object inputManager() throws Exception {
        Exception primo;
        try {
            return Class.forName("android.hardware.input.InputManagerGlobal").getMethod("getInstance").invoke(null);
        } catch (Exception e) {
            primo = e;
        }
        try {
            Method m = Class.forName("android.hardware.input.InputManager").getDeclaredMethod("getInstance");
            m.setAccessible(true);
            return m.invoke(null);
        } catch (Exception e) {
            throw new IllegalStateException("né InputManagerGlobal (" + causa(primo) + ") né InputManager (" + causa(e) + ")");
        }
    }

    /** La prima classe che esiste tra quelle indicate. */
    static Class<?> classe(String... nomi) throws ClassNotFoundException {
        for (String nome : nomi) {
            try {
                return Class.forName(nome);
            } catch (ClassNotFoundException e) {
                // si prova la successiva
            }
        }
        throw new ClassNotFoundException(String.join(" | ", nomi));
    }

    /** Metodi pubblici con questo nome (le firme cambiano tra le versioni). */
    static List<Method> metodi(Class<?> classe, String nome) {
        List<Method> trovati = new ArrayList<>();
        for (Method m : classe.getMethods()) {
            if (m.getName().equals(nome)) {
                trovati.add(m);
            }
        }
        return trovati;
    }

    /**
     * Il metodo con questo nome e uno dei numeri di parametri indicati, nell'ordine
     * di preferenza (la variante più recente per prima).
     */
    static Method metodo(Class<?> classe, String nome, int... parametri) throws NoSuchMethodException {
        List<Method> candidati = metodi(classe, nome);
        for (int n : parametri) {
            for (Method m : candidati) {
                if (m.getParameterTypes().length == n) {
                    m.setAccessible(true);
                    return m;
                }
            }
        }
        throw new NoSuchMethodException(classe.getName() + "." + nome + " (varianti: " + firme(classe, nome) + ")");
    }

    /** Numeri di parametri delle varianti di un metodo, per esempio «2,3» (o «nessuna»). */
    static String firme(Class<?> classe, String nome) {
        Set<Integer> n = new TreeSet<>();
        for (Method m : metodi(classe, nome)) {
            n.add(m.getParameterTypes().length);
        }
        if (n.isEmpty()) {
            return "nessuna";
        }
        StringBuilder s = new StringBuilder();
        for (int i : n) {
            s.append(s.length() > 0 ? "," : "").append(i);
        }
        return s.toString();
    }

    /** Chiama il metodo con questo nome i cui parametri accettano gli argomenti dati. */
    static Object invoca(Object oggetto, String nome, Object... argomenti) throws Exception {
        for (Method m : metodi(oggetto.getClass(), nome)) {
            if (compatibili(m.getParameterTypes(), argomenti)) {
                try {
                    // I proxy AIDL sono classi private: senza questo l'invoke può essere negato.
                    m.setAccessible(true);
                    return m.invoke(oggetto, argomenti);
                } catch (InvocationTargetException e) {
                    Throwable t = e.getCause();
                    throw t instanceof Exception ? (Exception) t : new RuntimeException(t);
                }
            }
        }
        throw new NoSuchMethodException(oggetto.getClass().getName() + "." + nome + " con " + argomenti.length + " argomenti");
    }

    private static boolean compatibili(Class<?>[] tipi, Object[] argomenti) {
        if (tipi.length != argomenti.length) {
            return false;
        }
        for (int i = 0; i < tipi.length; i++) {
            Object a = argomenti[i];
            Class<?> t = tipi[i];
            if (a == null) {
                if (t.isPrimitive()) {
                    return false;
                }
            } else if (t == int.class) {
                if (!(a instanceof Integer)) {
                    return false;
                }
            } else if (t == boolean.class) {
                if (!(a instanceof Boolean)) {
                    return false;
                }
            } else if (t == long.class) {
                if (!(a instanceof Long)) {
                    return false;
                }
            } else if (!t.isInstance(a)) {
                return false;
            }
        }
        return true;
    }

    /** Campo (anche nascosto o ereditato) di un oggetto; {@code null} se non c'è. */
    static Object campo(Object oggetto, String nome) {
        for (Class<?> c = oggetto.getClass(); c != null; c = c.getSuperclass()) {
            try {
                Field f = c.getDeclaredField(nome);
                f.setAccessible(true);
                return f.get(oggetto);
            } catch (NoSuchFieldException e) {
                // si cerca nella classe madre
            } catch (Exception e) {
                return null;
            }
        }
        return null;
    }

    /** Causa leggibile di un errore (senza gli involucri della riflessione). */
    static String causa(Throwable e) {
        while (e.getCause() != null && (e instanceof InvocationTargetException || e.getClass() == RuntimeException.class)) {
            e = e.getCause();
        }
        return e.getClass().getSimpleName() + (e.getMessage() != null ? ": " + e.getMessage() : "");
    }
}
