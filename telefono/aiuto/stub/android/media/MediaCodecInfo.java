package android.media;
public final class MediaCodecInfo {
    public String getName() { throw new RuntimeException(); }
    public String getCanonicalName() { throw new RuntimeException(); }
    public boolean isAlias() { throw new RuntimeException(); }
    public boolean isEncoder() { throw new RuntimeException(); }
    public boolean isHardwareAccelerated() { throw new RuntimeException(); }
    public boolean isSoftwareOnly() { throw new RuntimeException(); }
    public boolean isVendor() { throw new RuntimeException(); }
    public String[] getSupportedTypes() { throw new RuntimeException(); }
    public CodecCapabilities getCapabilitiesForType(String tipo) { throw new RuntimeException(); }
    public static final class CodecCapabilities {
        public CodecProfileLevel[] profileLevels;
        public int getMaxSupportedInstances() { throw new RuntimeException(); }
        public AudioCapabilities getAudioCapabilities() { throw new RuntimeException(); }
        public VideoCapabilities getVideoCapabilities() { throw new RuntimeException(); }
        public EncoderCapabilities getEncoderCapabilities() { throw new RuntimeException(); }
    }
    public static final class CodecProfileLevel {
        public int profile;
        public int level;
    }
    public static final class AudioCapabilities {
        public android.util.Range<Integer> getBitrateRange() { throw new RuntimeException(); }
        public int getMaxInputChannelCount() { throw new RuntimeException(); }
        public int[] getSupportedSampleRates() { throw new RuntimeException(); }
        public android.util.Range<Integer>[] getSupportedSampleRateRanges() { throw new RuntimeException(); }
    }
    public static final class VideoCapabilities {
        public android.util.Range<Integer> getBitrateRange() { throw new RuntimeException(); }
        public android.util.Range<Integer> getSupportedWidths() { throw new RuntimeException(); }
        public android.util.Range<Integer> getSupportedHeights() { throw new RuntimeException(); }
        public android.util.Range<Integer> getSupportedFrameRates() { throw new RuntimeException(); }
        public int getWidthAlignment() { throw new RuntimeException(); }
        public int getHeightAlignment() { throw new RuntimeException(); }
    }
    public static final class EncoderCapabilities {
        public static final int BITRATE_MODE_CQ = 0;
        public static final int BITRATE_MODE_VBR = 1;
        public static final int BITRATE_MODE_CBR = 2;
        public static final int BITRATE_MODE_CBR_FD = 3;
        public boolean isBitrateModeSupported(int modo) { throw new RuntimeException(); }
    }
}
