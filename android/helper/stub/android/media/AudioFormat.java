package android.media;
public final class AudioFormat {
    public static final int ENCODING_PCM_16BIT = 2;
    public static final int CHANNEL_IN_STEREO = 12;
    public static final class Builder {
        public Builder setEncoding(int e) { throw new RuntimeException(); }
        public Builder setSampleRate(int r) { throw new RuntimeException(); }
        public Builder setChannelMask(int m) { throw new RuntimeException(); }
        public AudioFormat build() { throw new RuntimeException(); }
    }
}
