package android.content;
public class Intent {
    public static final String ACTION_MAIN = "android.intent.action.MAIN";
    public static final String ACTION_VIEW = "android.intent.action.VIEW";
    public static final String CATEGORY_LAUNCHER = "android.intent.category.LAUNCHER";
    public static final int FLAG_ACTIVITY_NEW_TASK = 0x10000000;
    public Intent(String azione) { throw new RuntimeException(); }
    public Intent(String azione, android.net.Uri dati) { throw new RuntimeException(); }
    public Intent addCategory(String categoria) { throw new RuntimeException(); }
    public Intent addFlags(int flag) { throw new RuntimeException(); }
    public Intent setPackage(String pacchetto) { throw new RuntimeException(); }
    public ComponentName getComponent() { throw new RuntimeException(); }
}
