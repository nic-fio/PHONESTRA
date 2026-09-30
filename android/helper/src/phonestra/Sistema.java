package phonestra;

import android.app.ActivityOptions;
import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.graphics.Bitmap;
import android.hardware.display.DisplayManager;
import android.hardware.display.VirtualDisplay;
import android.media.Image;
import android.media.ImageReader;
import android.net.Uri;
import android.os.Bundle;
import android.view.Surface;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/**
 * Pezzi comuni alle prove del video ({@link VideoProva}): schermi virtuali,
 * avvio e chiusura delle app, immagini PNG. Il contesto della shell sta in
 * {@link Contesto}, gli adattatori delle API nascoste in {@link Nascoste}.
 *
 * <p>Le API usate sono descritte in notes/study/video.md.
 */
final class Sistema {
    static final String SHELL = Contesto.SHELL;

    // Flag di DisplayManager.VIRTUAL_DISPLAY_FLAG_* (valori uguali da Android 14
    // a 16; molti sono nascosti, per questo sono scritti qui).
    static final int PUBLIC = 1;
    static final int PRESENTATION = 1 << 1;
    static final int SECURE = 1 << 2;
    static final int OWN_CONTENT_ONLY = 1 << 3;
    static final int AUTO_MIRROR = 1 << 4;
    static final int SUPPORTS_TOUCH = 1 << 6;
    static final int ROTATES_WITH_CONTENT = 1 << 7;
    static final int DESTROY_CONTENT_ON_REMOVAL = 1 << 8;
    static final int SHOULD_SHOW_SYSTEM_DECORATIONS = 1 << 9;
    static final int TRUSTED = 1 << 10;
    static final int OWN_DISPLAY_GROUP = 1 << 11;
    static final int ALWAYS_UNLOCKED = 1 << 12;
    static final int TOUCH_FEEDBACK_DISABLED = 1 << 13;
    static final int OWN_FOCUS = 1 << 14;
    static final int DEVICE_DISPLAY_GROUP = 1 << 15;
    static final int STEAL_TOP_FOCUS_DISABLED = 1 << 16;

    private static final String[] NOMI_FLAG = {
        "PUBLIC", "PRESENTATION", "SECURE", "OWN_CONTENT_ONLY", "AUTO_MIRROR", "1<<5", "SUPPORTS_TOUCH",
        "ROTATES_WITH_CONTENT", "DESTROY_CONTENT_ON_REMOVAL", "SHOULD_SHOW_SYSTEM_DECORATIONS", "TRUSTED",
        "OWN_DISPLAY_GROUP", "ALWAYS_UNLOCKED", "TOUCH_FEEDBACK_DISABLED", "OWN_FOCUS", "DEVICE_DISPLAY_GROUP",
        "STEAL_TOP_FOCUS_DISABLED",
    };

    /** Flag proposti in video.md §1.1, senza ALWAYS_UNLOCKED (si aggiunge a parte). */
    static final int FLAG_PROPOSTI = PUBLIC | PRESENTATION | OWN_CONTENT_ONLY | SUPPORTS_TOUCH | ROTATES_WITH_CONTENT
            | DESTROY_CONTENT_ON_REMOVAL | TRUSTED | OWN_DISPLAY_GROUP | OWN_FOCUS | TOUCH_FEEDBACK_DISABLED;

    /** Orologio: Samsung, Google, AOSP (il primo presente). */
    private static final String[] OROLOGI = {
        "com.sec.android.app.clockpackage", "com.google.android.deskclock", "com.android.deskclock",
    };

    private Sistema() {
    }

    /** Stampa una riga subito (il PC la legge mentre la prova è in corso). */
    static synchronized void scrivi(String riga) {
        System.out.println(riga);
        System.out.flush();
    }

    static DisplayManager displayManager() throws Exception {
        Object dm = Contesto.shell().getSystemService(Context.DISPLAY_SERVICE);
        if (dm instanceof DisplayManager) {
            return (DisplayManager) dm;
        }
        // Ripiego: il costruttore nascosto DisplayManager(Context).
        java.lang.reflect.Constructor<DisplayManager> c = DisplayManager.class.getDeclaredConstructor(Context.class);
        c.setAccessible(true);
        return c.newInstance(Contesto.shell());
    }

    /** Esegue un comando di shell (siamo la shell) e ne restituisce l'uscita, errori compresi. */
    static String esegui(String comando) {
        try {
            Process p = new ProcessBuilder("sh", "-c", comando).redirectErrorStream(true).start();
            ByteArrayOutputStream o = new ByteArrayOutputStream();
            try (InputStream i = p.getInputStream()) {
                byte[] b = new byte[8192];
                int n;
                while ((n = i.read(b)) > 0) {
                    o.write(b, 0, n);
                }
            }
            p.waitFor();
            return new String(o.toByteArray(), StandardCharsets.UTF_8).trim();
        } catch (Exception e) {
            return "(comando non riuscito: " + Nascoste.causa(e) + ")";
        }
    }

    static void attendi(long millisecondi) {
        try {
            Thread.sleep(millisecondi);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }

    /** Nomi dei flag VIRTUAL_DISPLAY_FLAG_* chiesti. */
    static String nomiFlag(int flag) {
        StringBuilder s = new StringBuilder();
        for (int i = 0; i < 31; i++) {
            if ((flag & (1 << i)) != 0) {
                s.append(s.length() > 0 ? "|" : "").append(i < NOMI_FLAG.length ? NOMI_FLAG[i] : "1<<" + i);
            }
        }
        return String.format("0x%x %s", flag, s);
    }

    /**
     * Flag effettivi dello schermo ({@code Display.getFlags()}), coi nomi letti
     * dalle costanti {@code Display.FLAG_*} del telefono stesso.
     */
    static String nomiFlagDisplay(int flag) {
        StringBuilder s = new StringBuilder();
        int noti = 0;
        try {
            for (Field f : Class.forName("android.view.Display").getDeclaredFields()) {
                if (f.getType() == int.class && Modifier.isStatic(f.getModifiers()) && f.getName().startsWith("FLAG_")) {
                    f.setAccessible(true);
                    int v = f.getInt(null);
                    if (v != 0 && Integer.bitCount(v) == 1 && (flag & v) != 0) {
                        s.append(s.length() > 0 ? "|" : "").append(f.getName().substring(5));
                        noti |= v;
                    }
                }
            }
        } catch (Exception e) {
            s.append("(nomi non letti: ").append(Nascoste.causa(e)).append(')');
        }
        if ((flag & ~noti) != 0) {
            s.append(String.format(" + ignoti 0x%x", flag & ~noti));
        }
        return String.format("0x%x %s", flag, s);
    }

    /** Le righe di {@code dumpsys display} che parlano dello schermo con questo nome. */
    static void dumpsysDisplay(String nome) {
        int n = 0;
        for (String riga : esegui("dumpsys display").split("\n")) {
            if (riga.contains("\"" + nome + "\"") || riga.contains(nome + ",")) {
                String r = riga.trim();
                scrivi("dumpsys: " + (r.length() > 1200 ? r.substring(0, 1200) + "…" : r));
                n++;
            }
        }
        if (n == 0) {
            scrivi("dumpsys: nessuna riga con «" + nome + "»");
        }
    }

    /** Stato di SystemUI (per il difetto noto del gesto «indietro»). */
    static void sysUiState(String quando) {
        String uscita = esegui("dumpsys activity service com.android.systemui/.SystemUIService"
                + " | grep -A4 'SysUiState state:'");
        scrivi("SysUiState " + quando + ":");
        for (String riga : uscita.split("\n")) {
            scrivi("  " + riga.trim());
        }
    }

    /** Telefono bloccato? (KeyguardManager.isKeyguardLocked, «?» se non si sa). */
    static String bloccato() {
        try {
            Object km = Contesto.shell().getSystemService("keyguard");
            return String.valueOf(km.getClass().getMethod("isKeyguardLocked").invoke(km));
        } catch (Exception e) {
            return "? (" + Nascoste.causa(e) + ")";
        }
    }

    /** Pacchetto dell'orologio presente sul telefono. */
    static String orologio() throws Exception {
        for (String p : OROLOGI) {
            if (Contesto.shell().getPackageManager().getLaunchIntentForPackage(p) != null) {
                return p;
            }
        }
        return OROLOGI[0];
    }

    static boolean installata(String pacchetto) throws Exception {
        return Contesto.shell().getPackageManager().getLaunchIntentForPackage(pacchetto) != null;
    }

    /** Uno schermo virtuale creato da una prova, con le voci di pulizia registrate. */
    static final class Schermo {
        final VirtualDisplay display;
        final int id;
        final String nome;

        Schermo(VirtualDisplay display, String nome) {
            this.display = display;
            this.id = display.getDisplay().getDisplayId();
            this.nome = nome;
        }

        /** Chiude subito task e schermo (nell'ordine giusto). */
        void chiudi(Pulizia pulizia) {
            pulizia.chiudi("task dello schermo " + id);
            pulizia.chiudi("schermo virtuale " + id);
        }
    }

    /**
     * Crea uno schermo virtuale con l'API pubblica
     * {@code DisplayManager.createVirtualDisplay(nome, l, a, dpi, surface, flag)}
     * e registra la pulizia: prima i task che contiene, poi lo schermo.
     */
    static Schermo creaSchermo(Pulizia pulizia, String nome, int l, int a, int dpi, Surface superficie, int flag)
            throws Exception {
        VirtualDisplay vd = displayManager().createVirtualDisplay(nome, l, a, dpi, superficie, flag);
        if (vd == null) {
            throw new IllegalStateException("createVirtualDisplay ha restituito null");
        }
        Schermo s = new Schermo(vd, nome);
        pulizia.aggiungi("schermo virtuale " + s.id, vd::release);
        pulizia.aggiungi("task dello schermo " + s.id, () -> togliTask(s.id));
        return s;
    }

    /**
     * Avvia un'app sullo schermo {@code display}: {@code ActivityOptions.setLaunchDisplayId}
     * e {@code IActivityManager.startActivityAsUser} (nascosta); ripiego {@code am start --display}.
     * Con {@code url} apre l'indirizzo nell'app. Restituisce una riga che dice com'è andata.
     */
    static String avviaApp(int display, String pacchetto, String url) throws Exception {
        Intent intent;
        if (url != null) {
            intent = new Intent(Intent.ACTION_VIEW, Uri.parse(url));
            intent.setPackage(pacchetto);
        } else {
            intent = Contesto.shell().getPackageManager().getLaunchIntentForPackage(pacchetto);
            if (intent == null) {
                throw new IllegalArgumentException("app non trovata: " + pacchetto);
            }
        }
        String comando = "am start --display " + display;
        if (url != null) {
            comando += " -a android.intent.action.VIEW -d '" + url.replace("'", "") + "' -p " + pacchetto;
        } else {
            ComponentName c = intent.getComponent();
            comando += " -n " + c.flattenToShortString();
        }
        return avviaIntent(display, intent, pacchetto, comando);
    }

    /**
     * Avvia {@code intent} sullo schermo {@code display} (vedi {@link #avviaApp});
     * {@code ripiego} è il comando {@code am start} da usare se la via nascosta
     * non riesce. Restituisce una riga che dice com'è andata.
     */
    static String avviaIntent(int display, Intent intent, String pacchetto, String ripiego) {
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        Bundle opzioni = ActivityOptions.makeBasic().setLaunchDisplayId(display).toBundle();
        String errore;
        try {
            Object am = Nascoste.activityManager();
            Method avvia = null;
            for (Method m : Nascoste.metodi(am.getClass(), "startActivityAsUser")) {
                if (m.getParameterTypes().length == 11) {
                    avvia = m;
                }
            }
            if (avvia == null) {
                throw new NoSuchMethodException("IActivityManager.startActivityAsUser a 11 parametri");
            }
            avvia.setAccessible(true);
            Object esito = avvia.invoke(am, null, SHELL, intent, null, null, null, 0, 0, null, opzioni, -2);
            // 0 = avviata, 2 = portata davanti, 3 = consegnata a quella in cima; negativo = errore.
            return "avvio di " + pacchetto + " sullo schermo " + display + ": startActivityAsUser → " + esito;
        } catch (Exception e) {
            errore = Nascoste.causa(e);
        }
        return "avvio di " + pacchetto + " sullo schermo " + display + ": startActivityAsUser non riuscita (" + errore
                + "), ripiego «" + ripiego + "» → " + esegui(ripiego).replace('\n', ' ');
    }

    /**
     * Toglie (anche dalle recenti) i task sullo schermo: {@code IActivityTaskManager
     * .getAllRootTaskInfosOnDisplay} + {@code removeTask}; ripiego {@code am stack}.
     */
    static void togliTask(int display) throws Exception {
        List<Integer> task = new ArrayList<>();
        String via;
        try {
            Object atm = Nascoste.activityTaskManager();
            List<?> radici = (List<?>) Nascoste.invoca(atm, "getAllRootTaskInfosOnDisplay", display);
            for (Object r : radici) {
                Object id = Nascoste.campo(r, "taskId");
                if (id instanceof Integer) {
                    task.add((Integer) id);
                }
            }
            for (int id : task) {
                Nascoste.invoca(atm, "removeTask", id);
            }
            via = "removeTask";
        } catch (Exception e) {
            via = "am stack remove (removeTask non riuscita: " + Nascoste.causa(e) + ")";
            task.clear();
            for (String riga : esegui("am stack list").split("\n")) {
                riga = riga.trim();
                if (riga.startsWith("RootTask id=") && riga.contains("displayId=" + display + " ")
                        || riga.startsWith("RootTask id=") && riga.endsWith("displayId=" + display)) {
                    String id = riga.substring("RootTask id=".length()).split("\\D", 2)[0];
                    esegui("am stack remove " + id);
                    task.add(Integer.parseInt(id));
                }
            }
        }
        scrivi("# task tolti dallo schermo " + display + ": " + task + " con " + via);
    }

    /**
     * Legge le immagini di uno schermo virtuale ({@code ImageReader} RGBA). Tiene
     * l'ultima immagine (se lo schermo è fermo non ne arrivano di nuove) e
     * scarta le altre ogni 50 ms: con la coda piena chi disegna si fermerebbe.
     */
    static final class Cattura implements AutoCloseable {
        private final ImageReader lettore;
        private Image ultima;
        private volatile boolean chiusa;

        Cattura(int larghezza, int altezza) {
            // 1 = PixelFormat.RGBA_8888
            lettore = ImageReader.newInstance(larghezza, altezza, 1, 3);
            Thread svuota = new Thread(() -> {
                while (!chiusa) {
                    synchronized (this) {
                        if (!chiusa) {
                            aggiorna();
                        }
                    }
                    attendi(50);
                }
            }, "svuota immagini");
            svuota.setDaemon(true);
            svuota.start();
        }

        Surface superficie() {
            return lettore.getSurface();
        }

        private synchronized Image aggiorna() {
            Image nuova = lettore.acquireLatestImage();
            if (nuova != null) {
                if (ultima != null) {
                    ultima.close();
                }
                ultima = nuova;
            }
            return ultima;
        }

        /** Salva l'ultima immagine in /data/local/tmp/{nome}.png e ne stampa il percorso. */
        synchronized String png(String nome) throws Exception {
            Image img = null;
            for (int i = 0; i < 20 && img == null; i++) {
                img = aggiorna();
                if (img == null) {
                    attendi(100);
                }
            }
            if (img == null) {
                scrivi("nessuna immagine dallo schermo virtuale (niente disegnato in 2 s)");
                return null;
            }
            Image.Plane piano = img.getPlanes()[0];
            ByteBuffer b = piano.getBuffer();
            int passoPixel = piano.getPixelStride();
            int passoRiga = piano.getRowStride();
            int l = img.getWidth();
            int a = img.getHeight();
            int[] pixel = new int[l * a];
            for (int y = 0; y < a; y++) {
                int riga = y * passoRiga;
                for (int x = 0; x < l; x++) {
                    int i = riga + x * passoPixel;
                    pixel[y * l + x] = 0xff000000 | (b.get(i) & 0xff) << 16 | (b.get(i + 1) & 0xff) << 8 | b.get(i + 2) & 0xff;
                }
            }
            return salvaPng(Bitmap.createBitmap(pixel, l, a, Bitmap.Config.ARGB_8888), pixel, nome);
        }

        @Override
        public synchronized void close() {
            chiusa = true;
            if (ultima != null) {
                ultima.close();
                ultima = null;
            }
            lettore.close();
        }
    }

    /** Salva un PNG in /data/local/tmp e stampa «png: percorso» e la quota di nero. */
    static String salvaPng(Bitmap bitmap, int[] pixel, String nome) throws Exception {
        File file = new File("/data/local/tmp/" + nome + ".png");
        try (FileOutputStream o = new FileOutputStream(file)) {
            bitmap.compress(Bitmap.CompressFormat.PNG, 100, o);
        }
        file.setReadable(true, false);
        bitmap.recycle();
        int neri = 0;
        for (int p : pixel) {
            if ((p >> 16 & 0xff) < 12 && (p >> 8 & 0xff) < 12 && (p & 0xff) < 12) {
                neri++;
            }
        }
        scrivi(String.format("immagine: %.1f %% di pixel neri", 100.0 * neri / Math.max(1, pixel.length)));
        scrivi("png: " + file.getPath());
        return file.getPath();
    }
}
