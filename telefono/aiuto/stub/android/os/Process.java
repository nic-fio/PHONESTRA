package android.os;
public class Process {
    public static final int THREAD_PRIORITY_URGENT_AUDIO = -19;
    public static int myTid() { throw new RuntimeException(); }
    public static int myPid() { throw new RuntimeException(); }
    public static int myUid() { throw new RuntimeException(); }
    public static void setThreadPriority(int priorita) { throw new RuntimeException(); }
}
