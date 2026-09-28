package phonestra;

import java.io.DataInputStream;
import java.io.EOFException;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;

/**
 * Formato dei messaggi del canale «comandi» (memoria/componente.md §3), uguale
 * a quello letto dal PC ({@code src/componente.rs}).
 *
 * <p>Intestazione di 8 byte, big-endian: {@code tipo u8 · bandiere u8 · id u16 ·
 * lunghezza u32}, poi {@code lunghezza} byte di contenuto. {@code id} lega una
 * risposta alla sua domanda (0 = messaggio spontaneo); la bandiera
 * {@link #RISPOSTA} dice che è una risposta.
 */
final class Protocollo {
    /** Versione del protocollo: cambia solo se un messaggio esistente cambia significato. */
    static final int VERSIONE = 1;

    // Tipi 0x01–0x0f: infrastruttura; 0x10–0x1f: prove e diagnosi; dal 0x20 i
    // pezzi (audio, video, input…).
    /** Servizio → PC, subito dopo l'apertura: righe «chiave=valore» (versione, telefono, autotest). */
    static final int CIAO = 0x01;
    /** Nei due sensi, ogni secondo, senza contenuto. */
    static final int BATTITO = 0x02;
    /** PC → servizio: chiudi; il servizio risponde FINE ed esce. */
    static final int FINE = 0x03;
    /** Risposta a una domanda non valida (tipo sconosciuto, errore): testo. */
    static final int ERRORE = 0x04;
    /** PC → servizio: prova innocua del custode (crea un file che solo il custode toglie). */
    static final int PROVA_CUSTODE = 0x10;

    /** Bandiera: il messaggio risponde a quello con lo stesso id. */
    static final int RISPOSTA = 0x01;

    /** Contenuto massimo accettato (un valore più grande è un flusso rovinato). */
    static final int MASSIMO = 16 * 1024 * 1024;

    /** Nel preambolo di ogni canale: 16 byte di segreto, poi il tipo. */
    static final int LUNGHEZZA_SEGRETO = 16;

    private Protocollo() {
    }

    /** Un messaggio del canale comandi. */
    static final class Messaggio {
        final int tipo;
        final int bandiere;
        final int id;
        final byte[] dati;

        Messaggio(int tipo, int bandiere, int id, byte[] dati) {
            this.tipo = tipo;
            this.bandiere = bandiere;
            this.id = id;
            this.dati = dati;
        }

        String testo() {
            return new String(dati, StandardCharsets.UTF_8);
        }

        boolean risposta() {
            return (bandiere & RISPOSTA) != 0;
        }
    }

    /** Il prossimo messaggio; {@link EOFException} se il canale è chiuso. */
    static Messaggio leggi(DataInputStream in) throws IOException {
        int tipo = in.readUnsignedByte();
        int bandiere = in.readUnsignedByte();
        int id = in.readUnsignedShort();
        int lunghezza = in.readInt();
        if (lunghezza < 0 || lunghezza > MASSIMO) {
            throw new IOException("messaggio troppo grande: " + (lunghezza & 0xffffffffL) + " byte");
        }
        byte[] dati = new byte[lunghezza];
        in.readFully(dati);
        return new Messaggio(tipo, bandiere, id, dati);
    }

    /** Intestazione e contenuto in un solo blocco (una sola scrittura sul socket). */
    static byte[] codifica(int tipo, int bandiere, int id, byte[] dati) {
        byte[] b = new byte[8 + dati.length];
        b[0] = (byte) tipo;
        b[1] = (byte) bandiere;
        b[2] = (byte) (id >> 8);
        b[3] = (byte) id;
        int n = dati.length;
        b[4] = (byte) (n >> 24);
        b[5] = (byte) (n >> 16);
        b[6] = (byte) (n >> 8);
        b[7] = (byte) n;
        System.arraycopy(dati, 0, b, 8, n);
        return b;
    }

    /**
     * Preambolo di un canale, mandato dal PC appena aperto il socket:
     * {@code segreto (16 byte) · lunghezza del tipo u8 · tipo (ASCII)}.
     * Restituisce il tipo, dopo aver controllato il segreto (confronto a tempo
     * costante); {@code null} se il segreto è sbagliato.
     */
    static String leggiPreambolo(InputStream flusso, byte[] segreto) throws IOException {
        DataInputStream in = new DataInputStream(flusso);
        byte[] ricevuto = new byte[LUNGHEZZA_SEGRETO];
        in.readFully(ricevuto);
        int n = in.readUnsignedByte();
        byte[] tipo = new byte[n];
        in.readFully(tipo);
        if (!java.security.MessageDigest.isEqual(ricevuto, segreto)) {
            return null;
        }
        return new String(tipo, StandardCharsets.US_ASCII);
    }

    /** Scrive un messaggio intero e lo spinge fuori subito. */
    static void scrivi(OutputStream out, int tipo, int bandiere, int id, byte[] dati) throws IOException {
        out.write(codifica(tipo, bandiere, id, dati));
        out.flush();
    }
}
