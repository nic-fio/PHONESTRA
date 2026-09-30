package android.view;
public final class MotionEvent extends InputEvent {
    public static MotionEvent obtain(long giu, long tempo, int azione, int quanti, PointerProperties[] proprieta,
            PointerCoords[] coordinate, int meta, int pulsanti, float precisioneX, float precisioneY, int dispositivo,
            int bordi, int sorgente, int bandiere) { throw new RuntimeException(); }
    public void recycle() { throw new RuntimeException(); }
    public static final class PointerProperties {
        public int id;
        public int toolType;
        public PointerProperties() { throw new RuntimeException(); }
    }
    public static final class PointerCoords {
        public float x;
        public float y;
        public float pressure;
        public float size;
        public PointerCoords() { throw new RuntimeException(); }
        public void setAxisValue(int asse, float valore) { throw new RuntimeException(); }
    }
}
