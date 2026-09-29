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
 * {@code android_servers}.
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
     * Ripristino del custode: riaccende il pannello in Java con la copia del jar
     * lasciata dal servizio, senza toccare il blocco del telefono. Il vecchio ripiego (addormentare e
     * risvegliare) bloccava il telefono e, sui Samsung, faceva cadere il
     * collegamento quando il servizio si riavviava (28 set): resta solo se la
     * copia manca.
     */
    static String ripristino() {
        String jar = Servizio.jarCustode;
        if (jar != null) {
            return "CLASSPATH=" + jar + " app_process / phonestra.Aiuto pannello 1";
        }
        return "dumpsys power | grep -q 'mWakefulness=Awake'"
                + " && { input keyevent KEYCODE_SLEEP; sleep 1; input keyevent KEYCODE_WAKEUP; }";
    }

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
            c.imposta(AZIONE, ORDINE, ripristino());
        }
        // A pannello acceso: il display va a 60 Hz prima dello spegnimento (prove §59).
        String minimaOriginale = !acceso && !spentoDaNoi ? frequenzaA60(c) : null;
        int n = 0;
        try {
            for (Object token : token()) {
                modo.invoke(null, token, acceso ? ACCESO : SPENTO);
                n++;
            }
        } finally {
            if (minimaOriginale != null) {
                rimettiMinima(minimaOriginale, c);
            }
        }
        spentoDaNoi = !acceso;
        if (acceso && c != null) {
            c.togli(AZIONE);
        }
        return n;
    }

    /** Impostazione di Android della frequenza minima del display, in Hz. */
    private static final String MINIMA = "min_refresh_rate";
    private static final String AZIONE_MINIMA = "frequenza minima";
    private static final int ORDINE_MINIMA = 410;
    /** Attesa massima della conferma dei 60 Hz prima di spegnere. */
    private static final long ATTESA_60_MS = 1000;

    /**
     * Porta il display a 60 Hz e aspetta che SurfaceFlinger lo confermi.
     * Coi Samsung la frequenza cambia da sola (10–120 Hz): dopo un attimo di
     * calma è a 24 Hz. Allo spegnimento SurfaceFlinger passa a 60 Hz, ma la
     * conferma del cambio richiede i vsync del pannello, che non arrivano più:
     * il suo modello resta a 24 Hz, prende i fotogrammi delle app a quel ritmo e
     * le app che disegnano più in fretta restano ferme ad aspettarlo. Facebook,
     * coi reel AV1 decodificati in software, intanto lascia a secco l'audio
     * (micro-interruzioni, prove §59). Se il modello è già a 60 Hz lo
     * spegnimento non cambia nulla.
     *
     * <p>Restituisce il valore originale di {@code min_refresh_rate} da
     * rimettere dopo lo spegnimento ({@code "null"} se non c'era), oppure
     * {@code null} se non è stato toccato.
     */
    private static String frequenzaA60(Custode c) {
        if (a60()) {
            return null;
        }
        String originale = Sistema.esegui("settings get system " + MINIMA);
        if (!originale.equals("null") && !originale.matches("[0-9.]+")) {
            Video.log("frequenza minima illeggibile: " + originale);
            return null;
        }
        try {
            if (c != null) {
                c.imposta(AZIONE_MINIMA, ORDINE_MINIMA, comandoMinima(originale));
            }
            Sistema.esegui("settings put system " + MINIMA + " 60");
            long inizio = System.currentTimeMillis();
            long fine = inizio + ATTESA_60_MS;
            boolean confermata;
            while (!(confermata = a60()) && System.currentTimeMillis() < fine) {
                Sistema.attendi(50);
            }
            Video.log("frequenza del display a 60 Hz prima dello spegnimento: "
                    + (confermata ? "confermata in " + (System.currentTimeMillis() - inizio) + " ms" : "non confermata"));
        } catch (Exception e) {
            Video.log("frequenza a 60 Hz non riuscita: " + Nascoste.causa(e));
        }
        return originale;
    }

    private static void rimettiMinima(String originale, Custode c) {
        Sistema.esegui(comandoMinima(originale));
        try {
            if (c != null) {
                c.togli(AZIONE_MINIMA);
            }
        } catch (Exception e) {
            Video.log("custode: " + Nascoste.causa(e));
        }
    }

    private static String comandoMinima(String originale) {
        return originale.equals("null") ? "settings delete system " + MINIMA
                : "settings put system " + MINIMA + " " + originale;
    }

    /**
     * Vero se il modello dei vsync di SurfaceFlinger è confermato ad almeno
     * 60 Hz (righe «mDisplayModePtr=…vsyncRate=» e
     * «mPeriodConfirmationInProgress=» di {@code dumpsys SurfaceFlinger}). Se
     * le righe mancano (altre versioni di Android) vale vero: niente da fare.
     */
    private static boolean a60() {
        String uscita = Sistema.esegui("dumpsys SurfaceFlinger | grep -m2 -e mPeriodConfirmationInProgress= -e mDisplayModePtr=");
        return confermatoA60(uscita);
    }

    static boolean confermatoA60(String uscita) {
        if (uscita.contains("mPeriodConfirmationInProgress=1")) {
            return false;
        }
        int i = uscita.indexOf("mDisplayModePtr=");
        int v = i < 0 ? -1 : uscita.indexOf("vsyncRate=", i);
        if (v < 0) {
            return true;
        }
        int fine = v + "vsyncRate=".length();
        while (fine < uscita.length() && (Character.isDigit(uscita.charAt(fine)) || uscita.charAt(fine) == '.')) {
            fine++;
        }
        try {
            return Double.parseDouble(uscita.substring(v + "vsyncRate=".length(), fine)) >= 59.5;
        } catch (NumberFormatException e) {
            return true;
        }
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
