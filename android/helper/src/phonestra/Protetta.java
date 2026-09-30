package phonestra;

import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;

/**
 * Schermata protetta (FLAG_SECURE) su uno schermo, senza {@code dumpsys}:
 * {@code IWindowManager.captureDisplay(display, CaptureArgs, ascoltatore)} e poi
 * {@code ScreenshotHardwareBuffer.containsSecureLayers()} (prove §43: vero con
 * Bitwarden, falso con l'Orologio).
 *
 * <p>La cattura si chiede rimpicciolita ({@code setFrameScale}, se il
 * {@code Builder} ce l'ha): serve solo la risposta sì/no, non l'immagine. I tipi
 * del 2° e 3° parametro si leggono dalla firma trovata (Android 14–15
 * {@code ScreenCapture$…}, 16 {@code ScreenCaptureInternal$…}); la firma si
 * cerca una volta sola.
 */
final class Protetta {
    /** Lato dell'immagine chiesta: 5% dell'originale (basta per la risposta). */
    private static final float SCALA = 0.05f;

    private static Method cattura;
    private static Class<?> tipoArgomenti;
    private static Method creaAscoltatore;
    private static final ExecutorService ATTESA = Executors.newCachedThreadPool(r -> {
        Thread t = new Thread(r, "cattura protetta");
        t.setDaemon(true);
        return t;
    });

    private Protetta() {
    }

    private static synchronized void prepara(Object wm) throws Exception {
        if (cattura != null) {
            return;
        }
        for (Method m : Nascoste.metodi(wm.getClass(), "captureDisplay")) {
            if (m.getParameterTypes().length == 3 && m.getParameterTypes()[0] == int.class) {
                m.setAccessible(true);
                cattura = m;
            }
        }
        if (cattura == null) {
            throw new NoSuchMethodException("IWindowManager.captureDisplay(int, …, …)");
        }
        Class<?>[] tipi = cattura.getParameterTypes();
        tipoArgomenti = tipi[1];
        for (Class<?> c : new Class<?>[] {tipi[2].getDeclaringClass(), tipi[2]}) {
            if (c == null) {
                continue;
            }
            try {
                creaAscoltatore = c.getMethod("createSyncCaptureListener");
                creaAscoltatore.setAccessible(true);
                break;
            } catch (NoSuchMethodException e) {
                // si prova la classe successiva
            }
        }
        if (creaAscoltatore == null) {
            throw new NoSuchMethodException("createSyncCaptureListener per " + tipi[2].getName());
        }
    }

    /** CaptureArgs rimpiccioliti; coi valori predefiniti o {@code null} se il Builder manca. */
    private static Object argomenti() {
        try {
            Constructor<?> c = Class.forName(tipoArgomenti.getName() + "$Builder").getDeclaredConstructor();
            c.setAccessible(true);
            Object b = c.newInstance();
            try {
                Method scala = b.getClass().getMethod("setFrameScale", float.class);
                scala.setAccessible(true);
                scala.invoke(b, SCALA);
            } catch (NoSuchMethodException e) {
                // immagine intera: costa di più, la risposta è la stessa
            }
            Method build = b.getClass().getMethod("build");
            build.setAccessible(true);
            return build.invoke(b);
        } catch (Exception e) {
            return null;
        }
    }

    /**
     * Se sullo schermo c'è una finestra protetta. Lancia se la cattura non si
     * può fare o non risponde entro 2 s.
     */
    static boolean presente(int display) throws Exception {
        Object wm = Nascoste.windowManager();
        prepara(wm);
        Object ascoltatore = creaAscoltatore.invoke(null);
        cattura.invoke(wm, display, argomenti(), ascoltatore);
        Future<Object> futuro = ATTESA.submit(() -> Nascoste.invoca(ascoltatore, "getBuffer"));
        Object buffer;
        try {
            buffer = futuro.get(2, TimeUnit.SECONDS);
        } catch (java.util.concurrent.TimeoutException e) {
            futuro.cancel(true);
            throw new IllegalStateException("captureDisplay senza risposta in 2 s");
        }
        if (buffer == null) {
            throw new IllegalStateException("captureDisplay senza immagine");
        }
        try {
            return Boolean.TRUE.equals(Nascoste.invoca(buffer, "containsSecureLayers"));
        } finally {
            try {
                Nascoste.invoca(Nascoste.invoca(buffer, "getHardwareBuffer"), "close");
            } catch (Exception e) {
                // già chiuso o metodo assente
            }
        }
    }
}
