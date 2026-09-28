package android.os;
// Valori letti sul telefono: inizializzati con un metodo perché javac non li
// copi come costanti nelle nostre classi.
public class Build {
    public static final String MANUFACTURER = valore();
    public static final String MODEL = valore();
    public static class VERSION {
        public static final String RELEASE = valore();
        public static final int SDK_INT = Integer.parseInt(valore());
    }
    private static String valore() { throw new RuntimeException(); }
}
