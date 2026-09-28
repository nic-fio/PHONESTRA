package phonestra;

import android.content.Context;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.content.pm.PackageManager;
import android.content.pm.ResolveInfo;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.drawable.Drawable;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.PrintStream;
import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Base64;
import java.util.List;

/**
 * Aiutante di Phonestra sul telefono: avviato con {@code app_process} come la
 * shell, stampa le app del launcher e termina. Non è un'app installata.
 *
 * <p>{@code app <lato>}: una riga per attività del launcher,
 * {@code pacchetto \t attività \t nome \t icona PNG in base64}, con l'icona
 * disegnata {@code lato}×{@code lato} (le icone adattive con la forma del telefono).
 *
 * <p>Altri comandi: {@code sfondo <larghezza>}, {@code appunti-sensibili},
 * {@code audio [sorgente=…] [formato=…] [priorita=…] [voce=…]} (vedi
 * {@link Audio}), {@code codificatori} (vedi {@link Codificatori}).
 * <p>{@code video-prova <prova> [opzioni]}: misure del video per il componente
 * nostro ({@link VideoProva}).
 * <p>{@code servizio}: il componente di lunga durata, un processo per
 * collegamento ({@link Servizio}, memoria/componente.md).
 */
public final class Aiuto {
    private Aiuto() {
    }

    public static void main(String[] args) throws Exception {
        String comando = args.length > 0 ? args[0] : "app";
        if (comando.equals("servizio")) {
            Servizio.main(Arrays.copyOfRange(args, 1, args.length));
            return;
        }
        if (comando.equals("sfondo")) {
            int larghezza = args.length > 1 ? Integer.parseInt(args[1]) : 540;
            System.out.println(sfondo(larghezza));
            return;
        }
        if (comando.equals("audio")) {
            Audio.cattura(Contesto.shell(), args, 1);
            return;
        }
        if (comando.equals("codificatori")) {
            Codificatori.stampa();
            return;
        }
        if (comando.equals("video-prova")) {
            VideoProva.main(Arrays.copyOfRange(args, 1, args.length));
            return;
        }
        if (comando.equals("appunti-sensibili")) {
            System.out.println(appuntiSensibili());
            return;
        }
        if (!comando.equals("app")) {
            System.err.println("comando sconosciuto: " + comando);
            System.exit(2);
        }
        caratterePredefinito();
        int lato = args.length > 1 ? Integer.parseInt(args[1]) : 96;
        PackageManager pm = Contesto.sistema().getPackageManager();
        stampaApp(pm, lato);
    }

    private static void stampaApp(PackageManager pm, int lato) throws Exception {
        Intent launcher = new Intent(Intent.ACTION_MAIN);
        launcher.addCategory(Intent.CATEGORY_LAUNCHER);
        List<ResolveInfo> attivita = pm.queryIntentActivities(launcher, 0);
        PrintStream uscita = new PrintStream(System.out, false, "UTF-8");
        for (ResolveInfo r : attivita) {
            ActivityInfo a = r.activityInfo;
            String nome;
            String icona;
            System.err.println("# " + a.packageName);
            try {
                nome = pulisci(String.valueOf(r.loadLabel(pm)));
                icona = png(r.loadIcon(pm), lato);
            } catch (Exception e) {
                System.err.println(a.packageName + ": " + e);
                continue;
            }
            uscita.print(a.packageName + "\t" + a.name + "\t" + nome + "\t" + icona + "\n");
            // Riga per riga: se il processo si interrompe, quelle già lette restano.
            uscita.flush();
        }
        uscita.flush();
    }

    /**
     * I caratteri di sistema li carica lo zygote, non app_process: senza un
     * carattere predefinito la prima icona con del testo (il Calendario
     * Samsung con la data) fa abortire il processo («gDefaultTypeface ==
     * nullptr»). {@code Typeface.loadPreinstalledSystemFontMap()} sul Samsung
     * fallisce; si imposta direttamente Roboto.
     */
    private static void caratterePredefinito() {
        String[] candidati = {"/system/fonts/Roboto-Regular.ttf", "/system/fonts/RobotoStatic-Regular.ttf"};
        for (String percorso : candidati) {
            File file = new File(percorso);
            if (!file.exists()) {
                continue;
            }
            try {
                // Font → FontFamily → Typeface nativo, senza il ripiego sul
                // predefinito che Typeface.Builder richiederebbe.
                Class<?> typeface = Class.forName("android.graphics.Typeface");
                Class<?> fontB = Class.forName("android.graphics.fonts.Font$Builder");
                Object font = fontB.getMethod("build").invoke(fontB.getConstructor(File.class).newInstance(file));
                Class<?> famiglia = Class.forName("android.graphics.fonts.FontFamily");
                Class<?> famigliaB = Class.forName("android.graphics.fonts.FontFamily$Builder");
                Object f = famigliaB.getMethod("build").invoke(
                        famigliaB.getConstructor(Class.forName("android.graphics.fonts.Font")).newInstance(font));
                long puntatore = (Long) famiglia.getMethod("getNativePtr").invoke(f);
                Method crea = typeface.getDeclaredMethod("nativeCreateFromArray", long[].class, long.class, int.class, int.class);
                crea.setAccessible(true);
                long nativo = (Long) crea.invoke(null, new long[] {puntatore}, 0L, -1, -1);
                Constructor<?> nuovo = typeface.getDeclaredConstructor(long.class);
                nuovo.setAccessible(true);
                Object carattere = nuovo.newInstance(nativo);
                Method imposta = typeface.getDeclaredMethod("setDefault", typeface);
                imposta.setAccessible(true);
                imposta.invoke(null, carattere);
                return;
            } catch (Exception e) {
                Throwable t = e.getCause() != null ? e.getCause() : e;
                System.err.println("carattere " + percorso + ": " + t);
                for (StackTraceElement r : t.getStackTrace()) {
                    System.err.println("  " + r);
                }
            }
        }
    }

    /**
     * «sensibile», «normale» o «vuoto»: se gli appunti attuali sono segnati
     * come sensibili (password, Android 13+: extra
     * {@code android.content.extra.IS_SENSITIVE} della ClipDescription).
     * Chiede al servizio degli appunti come pacchetto {@code com.android.shell};
     * la firma di {@code getPrimaryClipDescription} cambia tra le versioni.
     */
    private static String appuntiSensibili() throws Exception {
        Class<?> gestore = Class.forName("android.os.ServiceManager");
        Object binder = gestore.getMethod("getService", String.class).invoke(null, "clipboard");
        Class<?> stub = Class.forName("android.content.IClipboard$Stub");
        Object appunti = stub.getMethod("asInterface", Class.forName("android.os.IBinder")).invoke(null, binder);
        Object descrizione = null;
        for (Method m : appunti.getClass().getMethods()) {
            if (!m.getName().equals("getPrimaryClipDescription")) {
                continue;
            }
            Class<?>[] p = m.getParameterTypes();
            Object[] argomenti = new Object[p.length];
            for (int i = 0; i < p.length; i++) {
                if (p[i] == String.class) {
                    argomenti[i] = i == 0 ? "com.android.shell" : null;
                } else if (p[i] == int.class) {
                    // utente 0, poi eventuale dispositivo virtuale 0 (quello predefinito)
                    argomenti[i] = 0;
                }
            }
            descrizione = m.invoke(appunti, argomenti);
            break;
        }
        if (descrizione == null) {
            return "vuoto";
        }
        Object extra = descrizione.getClass().getMethod("getExtras").invoke(descrizione);
        if (extra == null) {
            return "normale";
        }
        Object sensibile = extra.getClass().getMethod("getBoolean", String.class, boolean.class)
                .invoke(extra, "android.content.extra.IS_SENSITIVE", false);
        return Boolean.TRUE.equals(sensibile) ? "sensibile" : "normale";
    }

    /**
     * Lo sfondo della schermata Home del telefono, rimpicciolito a
     * {@code larghezza} pixel, in PNG base64 (per il drawer di Phonestra).
     */
    private static String sfondo(int larghezza) throws Exception {
        caratterePredefinito();
        // Il servizio vuole un pacchetto che appartenga all'uid della shell.
        Context ctx = Contesto.shell();
        Class<?> classe = Class.forName("android.app.WallpaperManager");
        Object gestore = classe.getMethod("getInstance", Context.class).invoke(null, ctx);
        Drawable d = (Drawable) classe.getMethod("getDrawable").invoke(gestore);
        if (d == null) {
            return "";
        }
        int l = (Integer) Drawable.class.getMethod("getIntrinsicWidth").invoke(d);
        int a = (Integer) Drawable.class.getMethod("getIntrinsicHeight").invoke(d);
        int altezza = l > 0 ? Math.max(1, a * larghezza / l) : larghezza * 2;
        Bitmap b = Bitmap.createBitmap(larghezza, altezza, Bitmap.Config.ARGB_8888);
        d.setBounds(0, 0, larghezza, altezza);
        d.draw(new Canvas(b));
        ByteArrayOutputStream o = new ByteArrayOutputStream();
        b.compress(Bitmap.CompressFormat.PNG, 100, o);
        return Base64.getEncoder().encodeToString(o.toByteArray());
    }

    private static String pulisci(String s) {
        return s.replace('\t', ' ').replace('\n', ' ').replace('\r', ' ').trim();
    }

    private static String png(Drawable d, int lato) {
        Bitmap b = Bitmap.createBitmap(lato, lato, Bitmap.Config.ARGB_8888);
        d.setBounds(0, 0, lato, lato);
        d.draw(new Canvas(b));
        ByteArrayOutputStream o = new ByteArrayOutputStream();
        b.compress(Bitmap.CompressFormat.PNG, 100, o);
        return Base64.getEncoder().encodeToString(o.toByteArray());
    }
}
