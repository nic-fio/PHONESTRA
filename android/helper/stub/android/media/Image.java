package android.media;
public abstract class Image implements AutoCloseable {
    public abstract int getWidth();
    public abstract int getHeight();
    public abstract Plane[] getPlanes();
    public abstract void close();
    public abstract static class Plane {
        public abstract int getRowStride();
        public abstract int getPixelStride();
        public abstract java.nio.ByteBuffer getBuffer();
    }
}
