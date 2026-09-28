package phonestra;

import android.content.Context;
import android.media.AudioFormat;
import android.media.AudioRecord;
import android.media.MediaCodec;
import android.media.MediaFormat;
import android.media.MediaRecorder;

import java.io.BufferedOutputStream;
import java.io.DataOutputStream;
import java.io.FileDescriptor;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;

/**
 * Audio del telefono per Phonestra (memoria/api-android.md §1): cattura
 * l'uscita intera ({@code REMOTE_SUBMIX}, dopo il mixer), la comprime in AAC
 * e la scrive sull'uscita del processo finché il PC la legge.
 *
 * <p>Ogni pacchetto: 8 byte di orario in µs (bit 62 = configurazione del
 * codec), 4 byte di lunghezza, i dati. Gli orari vengono dal numero di campioni
 * letti, quindi sono regolari. La spedizione ha un thread suo: se il Wi-Fi
 * rallenta, la lettura dell'audio non si ferma.
 */
final class Audio {
    private static final int FREQUENZA = 48000;
    private static final int CANALI = 2;
    /** Campioni per canale in un blocco letto: quelli di un frame AAC. */
    private static final int BLOCCO = 1024;
    private static final int BYTE_PER_CAMPIONE = 2 * CANALI;
    private static final int BIT_RATE = 192000;
    private static final long CONFIGURAZIONE = 1L << 62;

    private Audio() {
    }

    static void cattura(Context shell) throws Exception {
        AudioRecord registratore = registratore(shell);
        MediaCodec codificatore = codificatore();
        BlockingQueue<byte[]> coda = new ArrayBlockingQueue<>(256);
        Thread spedizione = new Thread(() -> spedisci(coda), "spedizione");
        spedizione.setDaemon(true);
        spedizione.start();

        registratore.startRecording();
        codificatore.start();
        byte[] blocco = new byte[BLOCCO * BYTE_PER_CAMPIONE];
        MediaCodec.BufferInfo info = new MediaCodec.BufferInfo();
        long campioni = 0;
        try {
            while (spedizione.isAlive()) {
                int letti = leggiTutto(registratore, blocco);
                if (letti <= 0) {
                    throw new IOException("lettura dell'audio non riuscita: " + letti);
                }
                int indice = codificatore.dequeueInputBuffer(-1);
                ByteBuffer ingresso = codificatore.getInputBuffer(indice);
                ingresso.clear();
                ingresso.put(blocco, 0, letti);
                codificatore.queueInputBuffer(indice, 0, letti, campioni * 1_000_000L / FREQUENZA, 0);
                campioni += letti / BYTE_PER_CAMPIONE;
                svuota(codificatore, info, coda);
            }
        } finally {
            registratore.stop();
            registratore.release();
            codificatore.stop();
            codificatore.release();
        }
    }

    private static AudioRecord registratore(Context shell) throws Exception {
        AudioFormat formato = new AudioFormat.Builder()
                .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                .setSampleRate(FREQUENZA)
                .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
                .build();
        int minimo = AudioRecord.getMinBufferSize(FREQUENZA, AudioFormat.CHANNEL_IN_STEREO, AudioFormat.ENCODING_PCM_16BIT);
        // Mezzo secondo di riserva: assorbe le pause del thread di lettura.
        int riserva = Math.max(minimo * 4, FREQUENZA / 2 * BYTE_PER_CAMPIONE);
        AudioRecord.Builder b = new AudioRecord.Builder()
                .setAudioSource(MediaRecorder.AudioSource.REMOTE_SUBMIX)
                .setAudioFormat(formato)
                .setBufferSizeInBytes(riserva);
        // Il permesso CAPTURE_AUDIO_OUTPUT è della shell: il contesto deve essere il suo.
        AudioRecord.Builder.class.getMethod("setContext", Context.class).invoke(b, shell);
        AudioRecord r = b.build();
        if (r.getState() != AudioRecord.STATE_INITIALIZED) {
            r.release();
            throw new IOException("cattura dell'audio non disponibile");
        }
        return r;
    }

    private static MediaCodec codificatore() throws IOException {
        MediaFormat f = MediaFormat.createAudioFormat("audio/mp4a-latm", FREQUENZA, CANALI);
        f.setInteger("bitrate", BIT_RATE);
        f.setInteger("aac-profile", 2); // AAC-LC
        f.setInteger("max-input-size", BLOCCO * BYTE_PER_CAMPIONE);
        MediaCodec c = MediaCodec.createEncoderByType("audio/mp4a-latm");
        c.configure(f, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
        return c;
    }

    /** Legge un blocco intero (read può restituire meno del richiesto). */
    private static int leggiTutto(AudioRecord r, byte[] blocco) {
        int letti = 0;
        while (letti < blocco.length) {
            int n = r.read(blocco, letti, blocco.length - letti);
            if (n <= 0) {
                return letti > 0 ? letti : n;
            }
            letti += n;
        }
        return letti;
    }

    /** Mette in coda tutto quello che il codificatore ha pronto. */
    private static void svuota(MediaCodec c, MediaCodec.BufferInfo info, BlockingQueue<byte[]> coda) throws InterruptedException {
        while (true) {
            int indice = c.dequeueOutputBuffer(info, 0);
            if (indice < 0) {
                // Nessun dato pronto, o formato cambiato: si riprova al prossimo blocco.
                if (indice == MediaCodec.INFO_TRY_AGAIN_LATER) {
                    return;
                }
                continue;
            }
            if (info.size > 0) {
                ByteBuffer uscita = c.getOutputBuffer(indice);
                byte[] pacchetto = new byte[12 + info.size];
                boolean config = (info.flags & MediaCodec.BUFFER_FLAG_CODEC_CONFIG) != 0;
                ByteBuffer.wrap(pacchetto)
                        .putLong(config ? CONFIGURAZIONE : info.presentationTimeUs)
                        .putInt(info.size);
                uscita.position(info.offset);
                uscita.get(pacchetto, 12, info.size);
                if (!coda.offer(pacchetto)) {
                    // Il PC non legge da oltre 5 s: si perde il pacchetto più vecchio.
                    coda.poll();
                    coda.offer(pacchetto);
                }
            }
            c.releaseOutputBuffer(indice, false);
        }
    }

    /** Scrive i pacchetti sull'uscita del processo; finisce quando il PC chiude. */
    private static void spedisci(BlockingQueue<byte[]> coda) {
        try (DataOutputStream uscita = new DataOutputStream(
                new BufferedOutputStream(new FileOutputStream(FileDescriptor.out), 1 << 16))) {
            while (true) {
                uscita.write(coda.take());
                if (coda.isEmpty()) {
                    uscita.flush();
                }
            }
        } catch (IOException | InterruptedException e) {
            // Il PC ha chiuso il canale: il thread principale se ne accorge e finisce.
        }
    }
}
