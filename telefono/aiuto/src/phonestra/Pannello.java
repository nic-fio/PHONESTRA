package phonestra;

import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;

/**
 * Il pannello fisico del telefono acceso o spento, lasciando il telefono
 * sveglio (SPECIFICHE §5.9, studio/video.md §4.5): per ogni schermo fisico
 * {@code SurfaceControl.setDisplayPowerMode(token, 0 spento / 2 acceso)}.
 * Agisce solo sul compositore: Android crede lo schermo acceso (il touch del
 * telefono resta attivo) e qualunque cambio di stato (tasto di accensione)
 * riaccende il pannello.
 *
 * <p>I token degli schermi fisici: {@code SurfaceControl.getPhysicalDisplayIds/
 * getPhysicalDisplayToken} dove ci sono ancora; da Android 14 in poi stanno in
 * {@code com.android.server.display.DisplayControl}, dentro {@code services.jar}
 * (non nel classpath di {@code app_process}): la si carica con un class loader
 * sul {@code SYSTEMSERVERCLASSPATH} e con la sua libreria nativa
 * {@code android_servers}. Codice nostro, scrcpy solo come documentazione.
 *
 * <p>Pannello spento = azione registrata presso il {@link Custode}: se il
 * servizio muore col pannello spento, il custode lo riaccende.
 */
final class Pannello {
    static final int SPENTO = 0;
    static final int ACCESO = 2;
    private static final String AZIONE = "pannello";
    private static final int ORDINE = 400;

    /**
     * Ripristino del custode (niente API Java nella shell): se il telefono è
     * sveglio lo si addormenta e risveglia, e il pannello torna acceso. Costa
     * il blocco del telefono, ma lascia il pannello come l'utente se lo aspetta
     * invece che nero col touch attivo. Se il telefono dorme già non si tocca.
     */
    static final String RIPRISTINO = "dumpsys power | grep -q 'mWakefulness=Awake'"
            + " && { input keyevent KEYCODE_SLEEP; sleep 1; input keyevent KEYCODE_WAKEUP; }";

    private static Class<?> displayControl;
    private static boolean spentoDaNoi;

    private Pannello() {
    }

    /** Accende o spegne il pannello; restituisce quanti schermi fisici ha toccato. */
    static synchronized int imposta(boolean acceso) throws Exception {
        Class<?> sc = Class.forName("android.view.SurfaceControl");
        Method modo = Nascoste.metodo(sc, "setDisplayPowerMode", 2);
        Custode c = Servizio.custode();
        if (!acceso && c != null) {
            // Prima si registra il ripristino, poi si spegne: nessun istante scoperto.
            c.imposta(AZIONE, ORDINE, RIPRISTINO);
        }
        int n = 0;
        for (Object token : token()) {
            modo.invoke(null, token, acceso ? ACCESO : SPENTO);
            n++;
        }
        spentoDaNoi = !acceso;
        if (acceso && c != null) {
            c.togli(AZIONE);
        }
        return n;
    }

    static synchronized boolean spento() {
        return spentoDaNoi;
    }

    /** I token degli schermi fisici. */
    private static List<Object> token() throws Exception {
        List<Object> token = new ArrayList<>();
        Class<?> sc = Class.forName("android.view.SurfaceControl");
        Class<?> origine;
        try {
            sc.getMethod("getPhysicalDisplayIds");
            origine = sc;
        } catch (NoSuchMethodException e) {
            origine = displayControl();
        }
        Method ids = origine.getMethod("getPhysicalDisplayIds");
        Method prendi = origine.getMethod("getPhysicalDisplayToken", long.class);
        ids.setAccessible(true);
        prendi.setAccessible(true);
        for (long id : (long[]) ids.invoke(null)) {
            Object t = prendi.invoke(null, id);
            if (t != null) {
                token.add(t);
            }
        }
        if (token.isEmpty()) {
            throw new IllegalStateException("nessuno schermo fisico trovato");
        }
        return token;
    }

    /** {@code DisplayControl} da services.jar, con la sua libreria nativa (una volta sola). */
    private static Class<?> displayControl() throws Exception {
        if (displayControl != null) {
            return displayControl;
        }
        String percorso = System.getenv("SYSTEMSERVERCLASSPATH");
        if (percorso == null) {
            throw new IllegalStateException("SYSTEMSERVERCLASSPATH assente");
        }
        Class<?> fabbrica = Class.forName("com.android.internal.os.ClassLoaderFactory");
        Method crea = null;
        for (Method m : fabbrica.getDeclaredMethods()) {
            // createClassLoader(classPath, librarySearchPath, libraryPermittedPath,
            // parent, targetSdkVersion, isNamespaceShared, classLoaderName)
            if (m.getName().equals("createClassLoader") && m.getParameterTypes().length == 7) {
                crea = m;
            }
        }
        if (crea == null) {
            throw new NoSuchMethodException("ClassLoaderFactory.createClassLoader a 7 parametri");
        }
        crea.setAccessible(true);
        ClassLoader cl = (ClassLoader) crea.invoke(null, percorso, null, null, ClassLoader.getSystemClassLoader(), 0,
                true, null);
        Class<?> classe = cl.loadClass("com.android.server.display.DisplayControl");
        Method carica = Runtime.class.getDeclaredMethod("loadLibrary0", Class.class, String.class);
        carica.setAccessible(true);
        carica.invoke(Runtime.getRuntime(), classe, "android_servers");
        displayControl = classe;
        return classe;
    }
}
