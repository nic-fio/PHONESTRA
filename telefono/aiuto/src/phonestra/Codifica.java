package phonestra;

import android.media.MediaCodec;
import android.media.MediaCodecInfo;
import android.media.MediaCodecList;
import android.media.MediaFormat;
import android.os.Bundle;
import android.view.Surface;

import java.nio.ByteBuffer;
import java.util.Locale;

/**
 * Un codificatore video hardware che legge da una Surface (quella dello schermo
 * virtuale) e consegna i pacchetti a chi lo usa ({@link Uscita}), su un thread
 * suo. memoria/componente.md, «Video».
 *
 * <p>Formato come quello misurato con {@link VideoProva} (prove §43: 60
 * fotogrammi/s a 1120×1992, fotogramma chiave in ~40 ms): 8 Mbit/s, 60 fps
 * dichiarati, fotogramma chiave ogni 10 s, fotogramma ripetuto dopo 100 ms di
 * schermo fermo, priorità tempo reale, colori a gamma limitata; in più
 * {@code prepend-sps-pps-to-idr-frames}, così ogni fotogramma chiave chiesto
 * col comando porta davanti i parametri (se il codificatore non lo accetta si
 * riprova senza, e i parametri li rimanda {@link SessioneVideo}).
 *
 * <p>Una misura = un codificatore: cambiare misura vuol dire crearne uno nuovo
 * ({@link SessioneVideo#cambiaMisura}); il fotogramma chiave invece si chiede
 * senza ricreare niente ({@link #chiave()}).
 */
final class Codifica {
    private static final int COLOR_FORMAT_SURFACE = 0x7F000789;
    static final int BITRATE = 8_000_000;

    /** Chi riceve i pacchetti del codificatore (dal thread di lettura). */
    interface Uscita {
        /** Un pacchetto: parametri ({@code config}) o fotogramma; {@code dati} è una copia. */
        void pacchetto(Codifica da, boolean config, boolean chiave, long ptsUs, byte[] dati);

        /** Il codificatore si è fermato per un errore (non per {@link #chiudi()}). */
        void errore(Codifica da, String causa);
    }

    final String mime;
    final String nome;
    final int larghezza;
    final int altezza;
    /** Se i parametri (SPS/PPS) arrivano davanti a ogni fotogramma chiave. */
    final boolean anteponi;
    final Surface superficie;
    private final MediaCodec codec;
    private Thread lettore;
    private volatile boolean fermo;

    private Codifica(MediaCodec codec, String mime, int larghezza, int altezza, boolean anteponi) {
        this.codec = codec;
        this.mime = mime;
        this.nome = codec.getName();
        this.larghezza = larghezza;
        this.altezza = altezza;
        this.anteponi = anteponi;
        this.superficie = codec.createInputSurface();
    }

    /** «h264»/«avc» → video/avc, «h265»/«hevc» → video/hevc. */
    static String mime(String codec) {
        switch (codec.trim().toLowerCase(Locale.ROOT)) {
            case "h264":
            case "avc":
                return "video/avc";
            case "h265":
            case "hevc":
                return "video/hevc";
            default:
                throw new IllegalArgumentException("codec sconosciuto: " + codec);
        }
    }

    /** Nome breve del codec per il PC («h264», «h265»). */
    static String breve(String mime) {
        return mime.equals("video/hevc") ? "h265" : "h264";
    }

    /** Primo codificatore hardware (non alias) per il tipo; {@code null} se non ce n'è. */
    static MediaCodecInfo hardware(String mime) {
        for (MediaCodecInfo info : new MediaCodecList(MediaCodecList.REGULAR_CODECS).getCodecInfos()) {
            if (!info.isEncoder() || !info.isHardwareAccelerated() || info.isAlias()) {
                continue;
            }
            for (String t : info.getSupportedTypes()) {
                if (t.equalsIgnoreCase(mime)) {
                    return info;
                }
            }
        }
        return null;
    }

    /**
     * Misura arrotondata per difetto a multipli di 8 e all'allineamento chiesto
     * dal codificatore (misure non allineate: clic scartati in silenzio, prove §18).
     */
    static int[] allinea(String mime, int larghezza, int altezza) {
        int al = 8;
        int aa = 8;
        MediaCodecInfo info = hardware(mime);
        if (info != null) {
            MediaCodecInfo.VideoCapabilities v = info.getCapabilitiesForType(mime).getVideoCapabilities();
            al = mcm(al, Math.max(1, v.getWidthAlignment()));
            aa = mcm(aa, Math.max(1, v.getHeightAlignment()));
        }
        return new int[] {allineaUno(larghezza, al), allineaUno(altezza, aa)};
    }

    static int allineaUno(int valore, int passo) {
        return Math.max(passo, valore / passo * passo);
    }

    private static int mcm(int a, int b) {
        int x = a;
        int y = b;
        while (y != 0) {
            int t = x % y;
            x = y;
            y = t;
        }
        return a / x * b;
    }

    /**
     * Crea e configura il codificatore (non ancora avviato): prima l'hardware
     * con i parametri davanti ai fotogrammi chiave, poi senza, poi quello
     * predefinito di Android per il tipo.
     */
    static Codifica crea(String mime, int larghezza, int altezza) throws Exception {
        MediaCodecInfo info = hardware(mime);
        Exception ultimo = null;
        String[] nomi = info != null ? new String[] {info.getName(), null} : new String[] {null};
        for (String nome : nomi) {
            for (boolean anteponi : new boolean[] {true, false}) {
                MediaCodec c = nome != null ? MediaCodec.createByCodecName(nome) : MediaCodec.createEncoderByType(mime);
                try {
                    c.configure(formato(mime, larghezza, altezza, anteponi), null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
                    return new Codifica(c, mime, larghezza, altezza, anteponi);
                } catch (Exception e) {
                    ultimo = e;
                    c.release();
                }
            }
        }
        throw new IllegalStateException("nessun codificatore " + mime + " per " + larghezza + "×" + altezza + ": "
                + (ultimo != null ? Nascoste.causa(ultimo) : "?"));
    }

    private static MediaFormat formato(String mime, int larghezza, int altezza, boolean anteponi) {
        MediaFormat f = MediaFormat.createVideoFormat(mime, larghezza, altezza);
        f.setInteger("bitrate", BITRATE);
        f.setInteger("frame-rate", 60);
        f.setInteger("color-format", COLOR_FORMAT_SURFACE);
        f.setInteger("i-frame-interval", 10);
        f.setLong("repeat-previous-frame-after", 100_000L);
        f.setInteger("priority", 0);
        f.setInteger("color-range", 2); // COLOR_RANGE_LIMITED
        if (anteponi) {
            f.setInteger("prepend-sps-pps-to-idr-frames", 1);
        }
        return f;
    }

    /** Avvia la codifica e il thread che consegna i pacchetti a {@code uscita}. */
    void avvia(Uscita uscita) {
        codec.start();
        lettore = new Thread(() -> leggi(uscita), "codifica " + larghezza + "x" + altezza);
        lettore.setDaemon(true);
        lettore.start();
    }

    private void leggi(Uscita uscita) {
        MediaCodec.BufferInfo info = new MediaCodec.BufferInfo();
        try {
            while (!fermo) {
                int i = codec.dequeueOutputBuffer(info, 100_000);
                if (i < 0) {
                    continue;
                }
                try {
                    if (info.size > 0) {
                        ByteBuffer b = codec.getOutputBuffer(i);
                        byte[] dati = new byte[info.size];
                        b.position(info.offset);
                        b.get(dati, 0, info.size);
                        boolean config = (info.flags & MediaCodec.BUFFER_FLAG_CODEC_CONFIG) != 0;
                        boolean chiave = (info.flags & MediaCodec.BUFFER_FLAG_KEY_FRAME) != 0;
                        uscita.pacchetto(this, config, chiave, info.presentationTimeUs, dati);
                    }
                } finally {
                    codec.releaseOutputBuffer(i, false);
                }
            }
        } catch (Exception e) {
            if (!fermo) {
                uscita.errore(this, Nascoste.causa(e));
            }
        }
    }

    /** Fotogramma chiave appena possibile ({@code PARAMETER_KEY_REQUEST_SYNC_FRAME}). */
    void chiave() {
        Bundle b = new Bundle();
        b.putInt(MediaCodec.PARAMETER_KEY_REQUEST_SYNC_FRAME, 0);
        codec.setParameters(b);
    }

    /** Ferma il thread, il codificatore e la sua Surface; si può chiamare più volte. */
    void chiudi() {
        synchronized (this) {
            if (fermo) {
                return;
            }
            fermo = true;
        }
        if (lettore != null && lettore != Thread.currentThread()) {
            try {
                lettore.join(1000);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
        }
        try {
            codec.stop();
        } catch (Exception e) {
            // mai avviato o già in errore: si libera comunque
        }
        codec.release();
        superficie.release();
    }
}
