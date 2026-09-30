package phonestra;

import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.Matrix;
import android.media.ExifInterface;
import android.media.MediaMetadataRetriever;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.util.Base64;

/**
 * Miniature di foto e video del telefono per la finestra «Ricevi file…» del
 * PC: {@code miniature <lato> <percorso in base64>…} stampa una riga per
 * file, {@code indice \t JPEG in base64}, oppure {@code indice \t -} se la
 * miniatura non si può fare. I percorsi arrivano in base64 perché possono
 * contenere spazi e apostrofi.
 *
 * <p>Niente {@code ThumbnailUtils}: fuori da un'app vera cerca il
 * PackageManager dell'applicazione corrente e va in NullPointerException.
 * Le foto si decodificano già ridotte ({@code inSampleSize}) e si ruotano come
 * dice l'EXIF; dei video si prende il fotogramma chiave più vicino all'inizio.
 */
final class Miniature {
    private Miniature() {
    }

    static void stampa(String[] args) throws Exception {
        int lato = Integer.parseInt(args[1]);
        PrintStream uscita = new PrintStream(System.out, false, "UTF-8");
        for (int i = 2; i < args.length; i++) {
            String riga = "-";
            try {
                String percorso = new String(Base64.getDecoder().decode(args[i]), StandardCharsets.UTF_8);
                Bitmap b = video(percorso) ? fotogramma(percorso, lato) : foto(percorso, lato);
                if (b != null) {
                    ByteArrayOutputStream o = new ByteArrayOutputStream();
                    b.compress(Bitmap.CompressFormat.JPEG, 80, o);
                    b.recycle();
                    riga = Base64.getEncoder().encodeToString(o.toByteArray());
                }
            } catch (Exception e) {
                System.err.println((i - 2) + ": " + e);
            }
            uscita.print((i - 2) + "\t" + riga + "\n");
            // Riga per riga: il PC mostra le miniature man mano.
            uscita.flush();
        }
    }

    private static boolean video(String nome) {
        String n = nome.toLowerCase();
        return n.endsWith(".mp4") || n.endsWith(".mkv") || n.endsWith(".webm") || n.endsWith(".3gp") || n.endsWith(".mov");
    }

    /** La foto ridotta a circa {@code lato} pixel sul lato corto, dritta; {@code null} se non è un'immagine. */
    private static Bitmap foto(String percorso, int lato) throws Exception {
        BitmapFactory.Options o = new BitmapFactory.Options();
        o.inJustDecodeBounds = true;
        BitmapFactory.decodeFile(percorso, o);
        if (o.outWidth <= 0 || o.outHeight <= 0) {
            return null;
        }
        int campione = 1;
        while (Math.min(o.outWidth, o.outHeight) / (campione * 2) >= lato) {
            campione *= 2;
        }
        o.inJustDecodeBounds = false;
        o.inSampleSize = campione;
        Bitmap b = BitmapFactory.decodeFile(percorso, o);
        if (b == null) {
            return null;
        }
        int gradi = 0;
        try {
            switch (new ExifInterface(percorso).getAttributeInt(ExifInterface.TAG_ORIENTATION, 1)) {
                case 6: gradi = 90; break;
                case 3: gradi = 180; break;
                case 8: gradi = 270; break;
                default: break;
            }
        } catch (Exception e) {
            // PNG e altri formati senza EXIF: restano come sono.
        }
        return riduci(b, lato, gradi);
    }

    private static Bitmap fotogramma(String percorso, int lato) throws Exception {
        MediaMetadataRetriever r = new MediaMetadataRetriever();
        try {
            r.setDataSource(percorso);
            Bitmap b = r.getScaledFrameAtTime(0, MediaMetadataRetriever.OPTION_CLOSEST_SYNC, lato * 2, lato * 2);
            return b == null ? null : riduci(b, lato, 0);
        } finally {
            r.release();
        }
    }

    /** Lato corto a {@code lato} pixel (se più grande) e rotazione. */
    private static Bitmap riduci(Bitmap b, int lato, int gradi) {
        float scala = Math.min(1f, (float) lato / Math.min(b.getWidth(), b.getHeight()));
        if (scala == 1f && gradi == 0) {
            return b;
        }
        Matrix m = new Matrix();
        m.postScale(scala, scala);
        m.postRotate(gradi);
        Bitmap r = Bitmap.createBitmap(b, 0, 0, b.getWidth(), b.getHeight(), m, true);
        if (r != b) {
            b.recycle();
        }
        return r;
    }
}
