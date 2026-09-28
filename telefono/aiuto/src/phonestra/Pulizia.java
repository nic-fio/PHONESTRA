package phonestra;

import java.util.ArrayDeque;
import java.util.Deque;

/**
 * Cose da chiudere a fine prova (schermi virtuali, task, codificatori), in
 * ordine inverso a quello di creazione. Ogni voce si esegue una volta sola,
 * anche se la pulizia parte due volte (fine normale e guardiano del tempo), e
 * un errore in una voce non ferma le altre.
 */
final class Pulizia {
    /** Un'azione di pulizia: può lanciare, l'errore viene solo stampato. */
    interface Azione {
        void esegui() throws Exception;
    }

    private static final class Voce {
        final String nome;
        final Azione azione;

        Voce(String nome, Azione azione) {
            this.nome = nome;
            this.azione = azione;
        }
    }

    private final Deque<Voce> voci = new ArrayDeque<>();

    synchronized void aggiungi(String nome, Azione azione) {
        voci.push(new Voce(nome, azione));
    }

    /** Toglie senza eseguirla l'ultima voce con questo nome (già chiusa a mano). */
    synchronized void togli(String nome) {
        voci.removeIf(v -> v.nome.equals(nome));
    }

    /** Esegue subito e toglie la voce con questo nome, se c'è ancora. */
    void chiudi(String nome) {
        Voce trovata = null;
        synchronized (this) {
            for (Voce v : voci) {
                if (v.nome.equals(nome)) {
                    trovata = v;
                    break;
                }
            }
            if (trovata != null) {
                voci.remove(trovata);
            }
        }
        if (trovata != null) {
            esegui(trovata);
        }
    }

    void esegui() {
        while (true) {
            Voce v;
            synchronized (this) {
                v = voci.poll();
            }
            if (v == null) {
                return;
            }
            esegui(v);
        }
    }

    private static void esegui(Voce v) {
        try {
            v.azione.esegui();
            Sistema.scrivi("# pulizia: " + v.nome);
        } catch (Throwable e) {
            Sistema.scrivi("# pulizia: " + v.nome + " non riuscita: " + Nascoste.causa(e));
        }
    }
}
