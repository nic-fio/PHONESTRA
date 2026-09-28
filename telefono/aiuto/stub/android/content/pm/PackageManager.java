package android.content.pm;
public abstract class PackageManager {
    public static final int PERMISSION_GRANTED = 0;
    public abstract java.util.List<ResolveInfo> queryIntentActivities(android.content.Intent intent, int flag);
    public abstract android.content.Intent getLaunchIntentForPackage(String pacchetto);
    public abstract int checkPermission(String permesso, String pacchetto);
}
