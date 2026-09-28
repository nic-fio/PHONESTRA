package android.media;
public final class MediaCodec {
    public static final int CONFIGURE_FLAG_ENCODE = 1;
    public static final int BUFFER_FLAG_CODEC_CONFIG = 2;
    public static final int INFO_TRY_AGAIN_LATER = -1;
    public static MediaCodec createEncoderByType(String tipo) throws java.io.IOException { throw new RuntimeException(); }
    public void configure(MediaFormat f, android.view.Surface s, MediaCrypto c, int flags) { throw new RuntimeException(); }
    public void start() { throw new RuntimeException(); }
    public void stop() { throw new RuntimeException(); }
    public void release() { throw new RuntimeException(); }
    public int dequeueInputBuffer(long attesa) { throw new RuntimeException(); }
    public java.nio.ByteBuffer getInputBuffer(int indice) { throw new RuntimeException(); }
    public void queueInputBuffer(int indice, int inizio, int quanti, long pts, int flags) { throw new RuntimeException(); }
    public int dequeueOutputBuffer(BufferInfo info, long attesa) { throw new RuntimeException(); }
    public java.nio.ByteBuffer getOutputBuffer(int indice) { throw new RuntimeException(); }
    public void releaseOutputBuffer(int indice, boolean mostra) { throw new RuntimeException(); }
    public static final class BufferInfo {
        public int offset;
        public int size;
        public long presentationTimeUs;
        public int flags;
    }
}
