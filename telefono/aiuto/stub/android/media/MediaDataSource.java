package android.media;
public abstract class MediaDataSource implements java.io.Closeable {
    public abstract int readAt(long posizione, byte[] b, int inizio, int n) throws java.io.IOException;
    public abstract long getSize() throws java.io.IOException;
}
