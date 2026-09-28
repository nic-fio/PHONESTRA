package android.app;
// Classe nascosta di Android: ITaskStackListener.Stub con tutti i metodi vuoti.
public abstract class TaskStackListener {
    public TaskStackListener() { throw new RuntimeException(); }
    public void onTaskStackChanged() { throw new RuntimeException(); }
    public void onTaskCreated(int task, android.content.ComponentName componente) { throw new RuntimeException(); }
    public void onTaskRemoved(int task) { throw new RuntimeException(); }
    public void onTaskMovedToFront(ActivityManager.RunningTaskInfo info) { throw new RuntimeException(); }
    public void onTaskMovedToBack(ActivityManager.RunningTaskInfo info) { throw new RuntimeException(); }
    public void onTaskRemovalStarted(ActivityManager.RunningTaskInfo info) { throw new RuntimeException(); }
    public void onTaskDisplayChanged(int task, int display) { throw new RuntimeException(); }
    public void onTaskFocusChanged(int task, boolean fuoco) { throw new RuntimeException(); }
    public void onTaskRequestedOrientationChanged(int task, int orientamento) { throw new RuntimeException(); }
    public void onActivityRequestedOrientationChanged(int task, int orientamento) { throw new RuntimeException(); }
    public void onActivityRotation(int display) { throw new RuntimeException(); }
    public void onActivityLaunchOnSecondaryDisplayFailed(ActivityManager.RunningTaskInfo info, int display) { throw new RuntimeException(); }
    public void onActivityLaunchOnSecondaryDisplayRerouted(ActivityManager.RunningTaskInfo info, int display) { throw new RuntimeException(); }
    public void onBackPressedOnTaskRoot(ActivityManager.RunningTaskInfo info) { throw new RuntimeException(); }
}
