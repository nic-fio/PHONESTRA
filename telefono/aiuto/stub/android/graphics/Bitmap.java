package android.graphics;
public final class Bitmap {
    public enum Config { ARGB_8888 }
    public enum CompressFormat { PNG, JPEG }
    public static Bitmap createBitmap(int l, int a, Config c) { throw new RuntimeException(); }
    public static Bitmap createBitmap(int[] colori, int l, int a, Config c) { throw new RuntimeException(); }
    public static Bitmap createBitmap(Bitmap b, int x, int y, int l, int a, Matrix m, boolean filtro) { throw new RuntimeException(); }
    public Bitmap copy(Config c, boolean modificabile) { throw new RuntimeException(); }
    public int getWidth() { throw new RuntimeException(); }
    public int getHeight() { throw new RuntimeException(); }
    public void getPixels(int[] pixel, int inizio, int passo, int x, int y, int l, int a) { throw new RuntimeException(); }
    public boolean compress(CompressFormat f, int q, java.io.OutputStream o) { throw new RuntimeException(); }
    public void recycle() { throw new RuntimeException(); }
}
