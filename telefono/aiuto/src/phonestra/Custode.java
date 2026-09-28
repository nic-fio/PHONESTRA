package phonestra;

import java.io.File;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Il custode: un processo a parte che rimette a posto il telefono quando il
 * servizio muore, per qualsiasi causa (fine normale, battito mancato,
 * {@code kill -9}, crash nativo). memoria/componente.md §5.
 *
 * <p>È uno script di {@code sh} (mksh, la shell di Android) avviato con
 * {@code setsid}: entra in una sessione sua e ignora SIGHUP/SIGTERM, quindi non
 * muore insieme al servizio. Legge dal suo ingresso (un pipe che solo il
 * servizio tiene aperto) l'elenco delle azioni di ripristino; quando il
 * servizio muore il kernel chiude il pipe, lo script legge la fine del file ed
 * esegue le azioni, poi cancella le copie del jar e termina.
 *
 * <p>Perché {@code sh} e non un secondo {@code app_process}: parte in pochi
 * millisecondi e pesa ~2 MB invece dei 30–50 di ART, non ha bisogno del jar
 * (che il servizio può cancellare subito), e le azioni di ripristino sono
 * comunque comandi di shell ({@code settings}, {@code cmd media_session},
 * {@code am}, {@code rm}), che funzionano anche senza il contesto Android.
 *
 * <p>Protocollo sul pipe: il servizio manda ogni volta l'elenco intero, tra una
 * riga {@code #inizio} e una {@code #fine}; il custode lo adotta solo quando
 * arriva {@code #fine}. Un elenco a metà (servizio morto mentre scriveva) non
 * sostituisce il precedente. Una riga = un comando, eseguito con {@code sh -c};
 * un comando che fallisce non ferma i successivi.
 */
final class Custode {
    /** Nome del processo del custode (è il suo {@code $0}): serve a trovarlo con {@code ps}. */
    static final String NOME = "phonestra-custode";

    /**
     * Lo script. {@code $1} è il jar del servizio (cancellato solo se ha il nome
     * atteso). Alla fine toglie anche le copie dimenticate dell'aiutante e di
     * servizi precedenti più vecchie di un minuto: un aiutante interrotto di
     * colpo lascia la sua (prove-collegamento §43). Il minuto evita di togliere
     * il jar a un processo che sta partendo proprio adesso; a un processo già
     * partito il file non serve più (il dex è già aperto).
     *
     * <p>Tutto su una riga e col nome in testa: {@code ps} lo mostra su una
     * riga sola e il nome resta visibile anche se la riga viene accorciata.
     * L'a capo per unire le righe dell'elenco si costruisce con {@code printf}.
     */
    static final String SCRIPT = String.join(" ",
            ": " + "phonestra-custode;",
            "trap '' HUP INT TERM PIPE;",
            "nl=$(printf '\\n_'); nl=${nl%_};",
            "azioni=; nuovo=; dentro=0;",
            "while IFS= read -r riga; do",
            "case $riga in",
            "'#inizio') nuovo=; dentro=1 ;;",
            "'#fine') azioni=$nuovo; dentro=0 ;;",
            "*) if [ $dentro = 1 ]; then nuovo=\"$nuovo$riga$nl\"; fi ;;",
            "esac;",
            "done;",
            "printf '%s' \"$azioni\" | while IFS= read -r comando; do",
            "sh -c \"$comando\" </dev/null >/dev/null 2>&1;",
            "done;",
            "case $1 in /data/local/tmp/phonestra-servizio-*.jar) rm -f \"$1\" ;; esac;",
            "case $2 in /data/local/tmp/phonestra-custode-*.jar) rm -f \"$2\" ;; esac;",
            "find /data/local/tmp -maxdepth 1 \\( -name 'phonestra-servizio-*.jar' -o -name 'phonestra-aiuto.jar.*'"
                    + " -o -name 'phonestra-custode-*.jar' \\)"
                    + " -mmin +1 -delete 2>/dev/null;",
            "exit 0");

    private static final class Azione {
        final int ordine;
        final String comando;

        Azione(int ordine, String comando) {
            this.ordine = ordine;
            this.comando = comando;
        }
    }

    private final Process processo;
    private final OutputStream verso;
    private final boolean conSetsid;
    private final Map<String, Azione> azioni = new LinkedHashMap<>();

    private Custode(Process processo, boolean conSetsid) {
        this.processo = processo;
        this.verso = processo.getOutputStream();
        this.conSetsid = conSetsid;
    }

    /**
     * Avvia il custode. L'uscita va a /dev/null: se ereditasse quella del
     * servizio terrebbe aperto il canale d'avvio anche dopo la sua morte.
     */
    static Custode avvia(String jar, String jarCustode) throws IOException {
        File nulla = new File("/dev/null");
        List<String> comando = new ArrayList<>();
        comando.add("setsid");
        comando.add("sh");
        comando.add("-c");
        comando.add(SCRIPT);
        comando.add(NOME);
        comando.add(jar != null ? jar : "");
        comando.add(jarCustode != null ? jarCustode : "");
        try {
            return new Custode(new ProcessBuilder(comando).redirectOutput(nulla).redirectError(nulla).start(), true);
        } catch (IOException e) {
            // Senza setsid (non dovrebbe mancare: è in toybox) resta il trap su SIGHUP.
            System.err.println("custode senza setsid: " + Nascoste.causa(e));
            comando.remove(0);
            return new Custode(new ProcessBuilder(comando).redirectOutput(nulla).redirectError(nulla).start(), false);
        }
    }

    /** Per l'autotest: come è partito e se è ancora vivo. */
    String stato() {
        return (processo.isAlive() ? "ok" : "morto") + (conSetsid ? " setsid" : " senza-setsid");
    }

    /**
     * Aggiunge o sostituisce l'azione {@code nome}. Alla morte del servizio le
     * azioni si eseguono per {@code ordine} crescente (a parità, l'ultima
     * aggiunta per prima). Il comando è una riga di shell: niente a capo.
     */
    synchronized void imposta(String nome, int ordine, String comando) throws IOException {
        if (comando.indexOf('\n') >= 0 || comando.indexOf('\r') >= 0 || comando.startsWith("#")) {
            throw new IllegalArgumentException("comando di ripristino non valido: " + nome);
        }
        azioni.remove(nome);
        azioni.put(nome, new Azione(ordine, comando));
        manda();
    }

    /** Toglie l'azione {@code nome} (il servizio ha già rimesso a posto da sé). */
    synchronized void togli(String nome) throws IOException {
        if (azioni.remove(nome) != null) {
            manda();
        }
    }

    /** Il testo che il custode riceve per l'elenco attuale (separato per le prove). */
    synchronized String elenco() {
        // Ordine crescente; a parità, dall'ultima aggiunta (come la Pulizia delle
        // prove): si parte dall'ordine inverso e l'ordinamento è stabile.
        List<Azione> ordinate = new ArrayList<>(azioni.values());
        Collections.reverse(ordinate);
        ordinate.sort((a, b) -> Integer.compare(a.ordine, b.ordine));
        StringBuilder s = new StringBuilder("#inizio\n");
        for (Azione a : ordinate) {
            s.append(a.comando).append('\n');
        }
        return s.append("#fine\n").toString();
    }

    private void manda() throws IOException {
        verso.write(elenco().getBytes(StandardCharsets.UTF_8));
        verso.flush();
    }
}
