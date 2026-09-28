package android.media;
public class AudioRecord {
    public static final int STATE_INITIALIZED = 1;
    public static int getMinBufferSize(int rate, int channels, int encoding) { throw new RuntimeException(); }
    public int getState() { throw new RuntimeException(); }
    public void startRecording() { throw new RuntimeException(); }
    public int read(byte[] dati, int inizio, int quanti) { throw new RuntimeException(); }
    public void stop() { throw new RuntimeException(); }
    public void release() { throw new RuntimeException(); }
    public static class Builder {
        public Builder setAudioSource(int s) { throw new RuntimeException(); }
        public Builder setAudioFormat(AudioFormat f) { throw new RuntimeException(); }
        public Builder setBufferSizeInBytes(int b) { throw new RuntimeException(); }
        public AudioRecord build() { throw new RuntimeException(); }
    }
}
