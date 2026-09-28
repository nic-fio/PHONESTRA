package android.content;
public abstract class Context {
    public static final String DISPLAY_SERVICE = "display";
    public abstract android.content.pm.PackageManager getPackageManager();
    public abstract Object getSystemService(String nome);
    public abstract String getPackageName();
}
