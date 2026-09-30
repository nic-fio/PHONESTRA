from build import c, key, note, p, rif, steps, table, tip, ui, ul, warn


S1 = p("Per funzionare Phonestra cambia alcune cose sul telefono, solo finché è collegato. Alla chiusura le rimette "
       "com'erano, anche se il collegamento cade all'improvviso.", lead=True) + \
    table(["Che cosa", "Durante il collegamento", "Alla fine"], [
        ["Schermo del telefono", "Spento, col telefono sveglio e sbloccato (" + rif("Lo schermo del telefono") + ")",
         "Riacceso"],
        ["Tempo di spegnimento dello schermo", "Al massimo, perché il telefono non si addormenti", "Il valore di prima; "
         "se nel frattempo lo si è cambiato sul telefono, resta quello nuovo"],
        ["Volume multimediale", "Al massimo (il telefono resta muto: l'audio esce dal PC)", "Il valore di prima"],
        ["Audio del telefono", "Esce dal PC", "Torna all'altoparlante del telefono; la musica in corso va in pausa"],
        ["App aperte nelle finestre", "Girano su schermi in più, invisibili sul telefono", "Chiuse e tolte dalle "
         "recenti"],
        ["La parte di Phonestra sul telefono", "Un piccolo programma copiato in una cartella temporanea del telefono "
         "(" + c("/data/local/tmp") + ") e avviato a ogni collegamento", "Cancellato"],
    ], "«TAB» — Che cosa Phonestra cambia sul telefono e come lo rimette") + ul([
        "Sul telefono non si installa nessuna app, e non resta niente dopo la chiusura.",
        "Se Phonestra si chiude male (PC spento di colpo, Wi-Fi perso), il telefono si rimette a posto da solo "
        "pochi secondi dopo. Se qualcosa resta cambiato, Phonestra lo rimette al collegamento successivo: i valori di "
        "prima li ricorda sul PC.",
        "Dopo la chiusura il telefono resta sbloccato con lo schermo acceso: si spegne e si blocca da solo, col suo "
        "tempo di spegnimento, come sempre.",
    ]) + warn("in qualche caso il volume multimediale può restare al massimo dopo la chiusura (" + rif("Il volume")
              + "): basta abbassarlo con i tasti del telefono.", "Volume.")

S2 = p("Alcune cose accese per il primo collegamento restano accese anche dopo la chiusura di Phonestra, così la "
       "volta dopo non c'è niente da rifare.", lead=True) + \
    table(["Che cosa", "Perché resta", "Come spegnerlo"], [
        [ui("Opzioni sviluppatore"), "Servono al Debug wireless.", "Nelle Impostazioni del telefono, interruttore in "
         "cima alle " + ui("Opzioni sviluppatore") + "."],
        [ui("Debug wireless"), "Serve a Phonestra per trovare e raggiungere il telefono. Phonestra non lo spegne alla "
         "chiusura: la voce " + ui("Spegni il Debug wireless alla chiusura") + " del menu del telefono è "
         + ui("In arrivo") + ".", "Nelle " + ui("Opzioni sviluppatore") + ". Per usare di nuovo Phonestra andrà "
         "riacceso, senza rifare il codice."],
        ["Autorizzazione del PC", "Il telefono ricorda questo PC tra i " + ui("Dispositivi associati") + ". Phonestra "
         "toglie la scadenza automatica che Android dà alle autorizzazioni non usate da qualche giorno.",
         ui("Debug wireless") + " › " + ui("Dispositivi associati") + ": togliere il PC."],
    ], "«TAB» — Che cosa resta acceso sul telefono") + \
    note("con il Debug wireless acceso, il telefono si annuncia sulla rete Wi-Fi. Solo i PC associati col codice "
         "possono collegarsi. Su reti che non sono di casa (albergo, ufficio, luoghi pubblici) conviene spegnerlo "
         "quando non si usa Phonestra.", "Reti pubbliche.")

S3 = ul([
    "<b>Niente internet.</b> Phonestra parla solo col telefono, sulla rete di casa. Non manda dati a server esterni, "
    "non ha pubblicità né statistiche d'uso. L'unica eccezione è " + ui("Chiedi a Google ↗") + " in " + ui("Aggiungi "
    "un telefono") + ", che apre il browser su una ricerca, solo se lo si preme.",
    "<b>Collegamento cifrato.</b> Tra PC e telefono passa tutto cifrato, con la stessa protezione del Debug wireless "
    "di Android.",
    "<b>La chiave di Phonestra.</b> Il PC si fa riconoscere dal telefono con una chiave segreta, nel file "
    + c("~/.config/Phonestra/adbkey") + ". Chi ha quel file può collegarsi al telefono come Phonestra: non va copiato "
    "né condiviso.",
    "<b>Notifiche.</b> Phonestra legge titolo e testo delle notifiche del telefono per mostrarle sul PC. Non le "
    "conserva: restano solo mentre Phonestra è aperto. Con " + ui("Solo il nome dell'app") + " gli avvisi del "
    "desktop non mostrano mittente e testo.",
    "<b>Appunti.</b> Le password copiate sul telefono non passano al PC, e quelle copiate sul PC da un gestore di "
    "password non passano al telefono (" + rif("Gli appunti") + ").",
    "<b>Schermo del PC.</b> Mentre Phonestra è aperto, chi guarda il PC vede le app e le notifiche del telefono.",
    "<b>Schermate protette.</b> Le schermate che le app proteggono (banche, password) non arrivano mai al PC.",
]) + tip("per lasciare il PC per un po', basta chiudere Phonestra: il telefono torna com'era e sul PC non resta "
         "niente di aperto.", "Lasciare il PC.")

S4 = table(["Cartella", "Contenuto", "Si può cancellare?"], [
    [c("~/.config/Phonestra/adbkey"), "La chiave segreta di Phonestra", "Sì, ma poi ogni telefono va associato di "
     "nuovo"],
    [c("~/.config/Phonestra/telefoni.toml"), "I telefoni collegati: nome, modello, preferiti, i valori di tempo di "
     "spegnimento e volume da rimettere", "Sì: Phonestra riparte da " + ui("Aggiungi un telefono")],
    [c("~/.config/Phonestra/preferenze.toml"), "Le preferenze", "Sì: tornano quelle iniziali"],
    [c("~/.config/Phonestra/icone/"), "Le icone delle app, per gli avvisi del desktop", "Sì"],
    [c("~/.cache/Phonestra/"), "File di lavoro di Phonestra", "Sì, senza perdere niente"],
    [c("Immagini/Phonestra"), "Gli screenshot", "Sono i propri file"],
    [c("Video/Phonestra"), "Le registrazioni", "Sono i propri file"],
    [c("Scaricati"), "I file ricevuti dal telefono (se non si è scelta un'altra cartella)", "Sono i propri file"],
], "«TAB» — Le cartelle di Phonestra sul PC") + \
    p("Phonestra non scrive altrove: niente icone nel menu, niente servizi che partono con il PC, niente modifiche al "
      "sistema. Cancellare " + c("~/.config/Phonestra") + " riporta Phonestra come al primo avvio.") + \
    note(c("~") + " è la cartella personale. I nomi " + c("Immagini") + ", " + c("Video") + " e " + c("Scaricati")
         + " sono quelli di un desktop in italiano: Phonestra usa le cartelle che il desktop ha scelto per immagini, "
         "video e download.", "Nomi delle cartelle.")

S5 = steps([
    "Chiudere Phonestra, così il telefono torna com'era.",
    "Cancellare il file " + c("Phonestra-<versione>-x86_64.AppImage") + ".",
    "Cancellare le cartelle " + c("~/.config/Phonestra") + " e " + c("~/.cache/Phonestra") + " (nel file manager sono "
    "cartelle nascoste: si vedono con " + "«Mostra file nascosti»" + " o " + key("Ctrl", "H") + ").",
    "Se non servono più, cancellare anche " + c("Immagini/Phonestra") + " e " + c("Video/Phonestra") + ".",
    "Sul telefono: " + ui("Debug wireless") + " › " + ui("Dispositivi associati") + ", togliere il PC; poi spegnere "
    + ui("Debug wireless") + " e, se si vuole, le " + ui("Opzioni sviluppatore") + ".",
]) + p("Dopo questi passi di Phonestra non resta niente, né sul PC né sul telefono.")

CHAPTER = ("Telefono, PC e privacy", [
    ("Cosa cambia sul telefono", S1),
    ("Cosa resta acceso sul telefono", S2),
    ("Privacy e sicurezza", S3),
    ("Le cartelle sul PC", S4),
    ("Togliere Phonestra", S5),
])
