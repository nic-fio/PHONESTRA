package android.content;
// Interfaccia AIDL nascosta: l'ascoltatore degli appunti (Appunti.java) estende Stub.
// Sul telefono Stub estende android.os.Binder: qui basta il nome per javac.
public interface IOnPrimaryClipChangedListener {
    void dispatchPrimaryClipChanged();
    abstract class Stub implements IOnPrimaryClipChangedListener {
        public Stub() { throw new RuntimeException(); }
    }
}
