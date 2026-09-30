from build import c, code, note, p, rif, steps, warn

S1 = p("Un messaggio nuovo tocca i due lati: la classe Java del pezzo e il modulo Rust che lo usa. Questi sono "
       "i passi, nell'ordine.", lead=True) + steps([
    "Scegli il numero nella fascia del pezzo (video " + c("0x40–0x4f") + ", input " + c("0x50–0x5f") + ") o apri una "
    "fascia nuova di 16 per un pezzo nuovo.",
    "<b>Telefono</b>: la costante nella classe del pezzo (" + c("static final int NOME = 0x47;") + " in "
    + c("Video.java") + "), il " + c("case") + " nello " + c("switch") + " di " + c("Servizio.comandi") + " (o "
    + c("Input.nostro") + " per l'input), il gestore. Se è una domanda, rispondi con lo stesso " + c("id") + " e la "
    "bandiera " + c("RISPOSTA") + ", o con " + c("ERRORE") + ".",
    "Ricompila il jar con " + c("costruisci.sh") + ".",
    "<b>PC</b>: la costante in " + c("componente::tipo") + " (o " + c("input_nostro::tipo") + "), la codifica del "
    "contenuto come funzione pura, il metodo che la usa (" + c("Condiviso::domanda") + " per una domanda, "
    + c("Mittente::manda") + " per un evento senza risposta).",
    "Aggiungi il nome alla lista del test " + c("nessun_tipo_usato_da_due_moduli") + ": controlla che il numero sia "
    "unico, nella fascia giusta e uguale in Java e in Rust.",
    "Un test della codifica con i byte attesi, poi una prova con " + c("phonestra-prova") + ".",
    "Descrivi il messaggio nella tabella del suo capitolo in " + c("docs/sorgenti/tecnico/") + ".",
]) + note("un jar vecchio risponde " + c("ERRORE") + " «tipo sconosciuto» a un messaggio nuovo: il PC deve saperlo "
          "trattare. Cambia " + c("PROTOCOLLO") + " solo se un messaggio esistente cambia significato.", "Compatibilità.")

S2 = p("Un pezzo con un flusso suo, come l'audio e il video, ha bisogno di un canale proprio, aperto col "
       "preambolo.", lead=True) + steps([
    "<b>Telefono</b>: un gestore " + c("static void gestisci(LocalSocket s, String tipo)") + " e la sua voce in "
    + c("Servizio.TIPI") + ". Il gestore gira sul suo thread: cattura tutti gli errori (anche gli " + c("Error")
    + " delle API nascoste) e chiude il socket con " + c("shutdownInput") + "/" + c("shutdownOutput") + " prima di "
    + c("close") + ", altrimenti una lettura in corso in un altro thread tiene il socket aperto.",
    "<b>PC</b>: " + c("Condiviso::apritore().apri(\"tipo\")") + " dà un " + c("Canale") + " col preambolo già mandato.",
    "Se il pezzo cambia qualcosa sul telefono, registra l'azione inversa presso il custode.",
])

S3 = p("Ogni cosa che un pezzo cambia sul telefono deve avere la sua azione inversa presso il custode del "
       "servizio (" + rif("I due custodi") + ").", lead=True) + code("""
// Quando cambi qualcosa sul telefono:
Servizio.custode().imposta("mia-azione", 450, "settings put system qualcosa 1");
// Quando l'hai rimesso a posto da te:
Servizio.custode().togli("mia-azione");
""", "java", "Un'azione per il custode") + \
    p("Il comando è una riga di shell eseguita con " + c("sh -c") + " dopo la morte del servizio, senza contesto "
      "Android. Scegli l'ordine in modo che le azioni si facciano nella sequenza giusta (pannello 400, frequenza 410, "
      "task 500) e provala con " + c("kill -9") + " sul servizio (" + c("phonestra-prova shell 'kill -9 <pid>'") + ").") + \
    warn("se l'azione cambia un'impostazione dell'utente, rimettila solo se è ancora il valore di Phonestra: "
         "l'utente può averla cambiata nel frattempo (è la regola del tempo di spegnimento, prove §56).",
         "Il valore dell'utente vince.")

CHAPTER = ("Estendere Phonestra", [
    ("Un messaggio nuovo sul canale comandi", S1),
    ("Un tipo di canale nuovo", S2),
    ("Un'azione per il custode", S3),
])
