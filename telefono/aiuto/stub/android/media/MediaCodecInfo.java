package android.media;
public final class MediaCodecInfo {
    public String getName() { throw new RuntimeException(); }
    public boolean isEncoder() { throw new RuntimeException(); }
    public boolean isHardwareAccelerated() { throw new RuntimeException(); }
    public boolean isSoftwareOnly() { throw new RuntimeException(); }
    public boolean isVendor() { throw new RuntimeException(); }
    public boolean isAlias() { throw new RuntimeException(); }
    public String[] getSupportedTypes() { throw new RuntimeException(); }
    public CodecCapabilities getCapabilitiesForType(String tipo) { throw new RuntimeException(); }
    public static class CodecProfileLevel {
        public int profile;
        public int level;
    }
    public static final class CodecCapabilities {
        public CodecProfileLevel[] profileLevels;
        public int getMaxSupportedInstances() { throw new RuntimeException(); }
        public boolean isFeatureSupported(String nome) { throw new RuntimeException(); }
        public VideoCapabilities getVideoCapabilities() { throw new RuntimeException(); }
        public EncoderCapabilities getEncoderCapabilities() { throw new RuntimeException(); }
    }
    public static final class EncoderCapabilities {
        public boolean isBitrateModeSupported(int modo) { throw new RuntimeException(); }
    }
    public static final class VideoCapabilities {
        public int getWidthAlignment() { throw new RuntimeException(); }
        public int getHeightAlignment() { throw new RuntimeException(); }
        public android.util.Range<Integer> getSupportedWidths() { throw new RuntimeException(); }
        public android.util.Range<Integer> getSupportedHeights() { throw new RuntimeException(); }
        public android.util.Range<Integer> getBitrateRange() { throw new RuntimeException(); }
        public boolean areSizeAndRateSupported(int l, int a, double fps) { throw new RuntimeException(); }
        public java.util.List<PerformancePoint> getSupportedPerformancePoints() { throw new RuntimeException(); }
        public static final class PerformancePoint {
            public PerformancePoint(int l, int a, int fps) { throw new RuntimeException(); }
            public boolean covers(PerformancePoint altro) { throw new RuntimeException(); }
        }
    }
}
