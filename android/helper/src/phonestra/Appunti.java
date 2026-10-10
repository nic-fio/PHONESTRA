// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

package phonestra;

import android.content.IOnPrimaryClipChangedListener;

import java.lang.reflect.Method;
import java.util.List;

/**
 * Gli appunti del telefono per il modulo input (notes/component.md, «Input»;
 * study/input.md §6): lettura, scrittura, segno «sensibile» e ascoltatore
 * delle copie.
 *
 * <p>Si parla direttamente con {@code IClipboard} (il servizio «clipboard»)
 * come pacchetto {@code com.android.shell}: niente {@code ClipboardManager},
 * quindi niente Looper principale da far girare (l'ascoltatore è un Binder e
 * viene chiamato su un thread del Binder) e niente passaggio dal servizio
 * Samsung {@code semclipboard}, che col contesto sbagliato rifiuta la
 * scrittura (problema noto dei Samsung). La shell può leggere in background
 * ({@code READ_CLIPBOARD_IN_BACKGROUND}); a telefono bloccato la lettura dà
 * {@code null}.
 *
 * <p>Le firme cambiano tra le versioni (Android 14–16:
 * {@code getPrimaryClip(pacchetto, attributionTag, utente, dispositivo)},
 * {@code setPrimaryClip(clip, pacchetto, attributionTag, utente, dispositivo)}…):
 * si prende la variante più lunga i cui parametri, dopo quelli fissi, sono
 * solo testi e interi, e si riempiono così: il primo testo è il pacchetto, gli
 * altri {@code null}; gli interi 0 (utente 0, dispositivo predefinito).
 */
final class Appunti {
    /** Etichetta dei clip messi da Phonestra: riconosce i propri senza leggerne il testo. */
    static final String ETICHETTA = "Phonestra";
    /** {@code ClipDescription.EXTRA_IS_SENSITIVE} (Android 13+). */
    static final String SENSIBILE = "android.content.extra.IS_SENSITIVE";

    private static Object servizio;

    private Appunti() {
    }

    private static synchronized Object servizio() throws Exception {
        if (servizio == null) {
            servizio = Nascoste.appunti();
        }
        return servizio;
    }

    /**
     * Chiama il metodo {@code nome} di IClipboard coi parametri {@code fissi} in
     * testa e gli altri riempiti come detto sopra.
     */
    private static Object chiama(String nome, Object... fissi) throws Exception {
        Object s = servizio();
        Method scelto = null;
        for (Method m : Nascoste.metodi(s.getClass(), nome)) {
            Class<?>[] t = m.getParameterTypes();
            if (t.length < fissi.length || (scelto != null && t.length <= scelto.getParameterTypes().length)) {
                continue;
            }
            boolean adatto = true;
            for (int i = 0; i < t.length && adatto; i++) {
                if (i < fissi.length) {
                    adatto = fissi[i] == null ? !t[i].isPrimitive() : t[i].isInstance(fissi[i]);
                } else {
                    adatto = t[i] == String.class || t[i] == int.class;
                }
            }
            if (adatto) {
                scelto = m;
            }
        }
        if (scelto == null) {
            throw new NoSuchMethodException("IClipboard." + nome + " (varianti: " + Nascoste.firme(s.getClass(), nome) + ")");
        }
        Class<?>[] t = scelto.getParameterTypes();
        Object[] argomenti = new Object[t.length];
        boolean pacchetto = false;
        for (int i = 0; i < t.length; i++) {
            if (i < fissi.length) {
                argomenti[i] = fissi[i];
            } else if (t[i] == int.class) {
                argomenti[i] = 0;
            } else if (!pacchetto) {
                argomenti[i] = Contesto.SHELL;
                pacchetto = true;
            }
        }
        scelto.setAccessible(true);
        try {
            return scelto.invoke(s, argomenti);
        } catch (java.lang.reflect.InvocationTargetException e) {
            Throwable c = e.getCause();
            throw c instanceof Exception ? (Exception) c : new RuntimeException(c);
        }
    }

    /** Il clip attuale ({@code ClipData}), {@code null} se vuoto o telefono bloccato. */
    static Object clip() throws Exception {
        return chiama("getPrimaryClip");
    }

    /** La descrizione del clip attuale: non fa comparire l'avviso di accesso agli appunti. */
    static Object descrizione() throws Exception {
        return chiama("getPrimaryClipDescription");
    }

    /** Se la descrizione porta il segno «sensibile» (password, gestori di password). */
    static boolean sensibile(Object descrizione) throws Exception {
        Object extra = descrizione.getClass().getMethod("getExtras").invoke(descrizione);
        if (extra == null) {
            return false;
        }
        Object v = extra.getClass().getMethod("getBoolean", String.class, boolean.class).invoke(extra, SENSIBILE, false);
        return Boolean.TRUE.equals(v);
    }

    /** Etichetta della descrizione ({@code null} se non c'è). */
    static String etichetta(Object descrizione) throws Exception {
        Object e = descrizione.getClass().getMethod("getLabel").invoke(descrizione);
        return e == null ? null : e.toString();
    }

    /** Il testo del primo elemento del clip; {@code null} se il clip non è testo. */
    static String testo(Object clip) throws Exception {
        if (clip == null || (Integer) clip.getClass().getMethod("getItemCount").invoke(clip) == 0) {
            return null;
        }
        Object elemento = clip.getClass().getMethod("getItemAt", int.class).invoke(clip, 0);
        Object t = elemento.getClass().getMethod("getText").invoke(elemento);
        return t == null ? null : t.toString();
    }

    /** Un clip di testo semplice con questa etichetta; {@code sensibile} aggiunge il segno alla descrizione. */
    static Object nuovo(String etichetta, String testo, boolean sensibile) throws Exception {
        Class<?> classe = Class.forName("android.content.ClipData");
        Object clip = classe.getMethod("newPlainText", CharSequence.class, CharSequence.class).invoke(null, etichetta, testo);
        if (sensibile) {
            Object descrizione = classe.getMethod("getDescription").invoke(clip);
            Class<?> fagotto = Class.forName("android.os.PersistableBundle");
            Object extra = fagotto.getConstructor().newInstance();
            fagotto.getMethod("putBoolean", String.class, boolean.class).invoke(extra, SENSIBILE, true);
            descrizione.getClass().getMethod("setExtras", fagotto).invoke(descrizione, extra);
        }
        return clip;
    }

    /** Mette il clip negli appunti. */
    static void imposta(Object clip) throws Exception {
        chiama("setPrimaryClip", clip);
    }

    /** Svuota gli appunti. */
    static void svuota() throws Exception {
        chiama("clearPrimaryClip");
    }

    static void ascolta(IOnPrimaryClipChangedListener ascoltatore) throws Exception {
        chiama("addPrimaryClipChangedListener", ascoltatore);
    }

    static void smetti(IOnPrimaryClipChangedListener ascoltatore) throws Exception {
        chiama("removePrimaryClipChangedListener", ascoltatore);
    }

    /** Le varianti dei metodi usati, per la diagnosi (per esempio «getPrimaryClip=4»). */
    static String firme() throws Exception {
        Object s = servizio();
        StringBuilder r = new StringBuilder();
        for (String nome : List.of("getPrimaryClip", "setPrimaryClip", "addPrimaryClipChangedListener")) {
            r.append(r.length() > 0 ? " " : "").append(nome).append('=').append(Nascoste.firme(s.getClass(), nome));
        }
        return r.toString();
    }
}
