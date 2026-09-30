from build import c, note, p, rif, table, term, tip, ui, ul


S1 = table(["Sintomo", "Causa probabile", "Rimedio"], [
    "Collegamento",
    ["Il drawer resta su " + ui("Collegamento a <nome>…"), "Il telefono è spento, bloccato, su un'altra rete Wi-Fi, "
     "o il " + ui("Debug wireless") + " si è spento (succede cambiando rete)", "Sbloccare il telefono, controllare la "
     "rete, riaccendere " + ui("Debug wireless") + " nelle " + ui("Opzioni sviluppatore") + ". Phonestra si collega da "
     "solo appena lo trova (" + rif("I collegamenti successivi") + ")."],
    ["Pillola arancione, " + ui("bloccato: sbloccalo"), "Il telefono si è bloccato", "Sbloccarlo: Phonestra si "
     "ricollega da solo (" + rif("Telefono bloccato") + ")."],
    ["Pillola arancione, " + ui("riconnessione…"), "Il collegamento è caduto (Wi-Fi debole, telefono bloccato)",
     "Aspettare qualche secondo, oppure " + ui("Riconnetti ora") + " o " + ui("Riconnetti") + "."],
    ["Pillola rossa, " + ui("Phonestra non parte sul telefono"), "La parte di Phonestra che mostra le app non si "
     "avvia sul telefono; il motivo è scritto tra parentesi", "Premere " + ui("Riconnetti ora") + "; se non basta, "
     "riavviare il telefono e aspettare che Phonestra si ricolleghi."],
    ["In " + ui("Aggiungi un telefono") + " il campo del codice resta grigio", "Phonestra non vede la schermata del "
     "codice: telefono su un'altra rete, o un firewall del PC che blocca la ricerca in rete", "Controllare la rete; "
     "chiudere e riaprire " + ui("Associa dispositivo con codice di associazione") + ". Se il PC ha un firewall, "
     "permettere la ricerca dei dispositivi in rete locale (mDNS, porta UDP 5353)."],
    [ui("Codice non accettato (…)"), "Codice sbagliato o scaduto", "Riaprire la schermata del codice sul telefono e "
     "scrivere il codice nuovo (" + rif("Il codice a 6 cifre") + ")."],
    [ui("Debug wireless") + " grigio o che si rispegne", "Una protezione del telefono lo impedisce ("
     + ui("Blocco automatico") + ", " + ui("Protezione avanzata") + "), o la rete non è una rete Wi-Fi",
     "Spegnere la protezione (voce 3 di " + ui("Aggiungi un telefono") + "); usare una rete Wi-Fi, non i dati mobili."],
    "App e finestre",
    ["Le app non compaiono: " + ui("App non lette"), "Il collegamento è caduto mentre si leggeva l'elenco",
     "Aspettare il ricollegamento, oppure " + ui("Riconnetti") + "."],
    ["Manca un'app appena installata sul telefono", "L'elenco si legge al collegamento", ui("Preferenze") + " › "
     + ui("Aggiorna ora") + "."],
    ["Finestra nera con " + ui("Schermata protetta"), "L'app non permette di mostrare quella schermata fuori dal "
     "telefono", "Usare il telefono in mano per quella schermata (" + rif("Schermate protette") + ")."],
    ["Le griglie del drawer sono sbiadite e non rispondono", "Il telefono non è utilizzabile in quel momento",
     "Guardare lo stato nella pillola e il velo sullo schermo del telefono."],
    ["Sui Xiaomi, Redmi e Poco le app si vedono ma non rispondono a mouse e tastiera", "Manca un'impostazione di "
     "sicurezza di Xiaomi", "Nelle " + ui("Opzioni sviluppatore") + " accendere " + ui("Debug USB (impostazioni di "
     "sicurezza)") + "."],
    ["I tasti non arrivano allo schermo del telefono nel drawer", "I tasti vanno alla ricerca delle app", "Fare "
     "prima un clic sullo schermo del telefono."],
    ["La finestra di un'app non si allarga né si ingrandisce", "L'app accetta solo la forma verticale", "È voluto ("
     + rif("Dimensioni, rotazione e app verticali") + ")."],
    "Audio, appunti, notifiche",
    ["Niente audio subito dopo il collegamento", "L'audio parte qualche secondo dopo le immagini", "Aspettare qualche "
     "secondo."],
    ["Un'app resta muta", "L'app vieta di catturare il suo audio, oppure è una chiamata", "Le chiamate si sentono "
     "dal telefono (" + rif("Le chiamate") + ")."],
    ["Niente audio da nessuna app", "Volume del PC basso, o uscita audio sbagliata", "Controllare il volume e "
     "l'uscita nel mixer del desktop."],
    ["Volume del telefono rimasto al massimo dopo la chiusura", "Problema noto", "Abbassarlo con i tasti del telefono."],
    ["Il testo copiato sul telefono non si incolla sul PC", "Su GNOME gli appunti cambiano solo con una finestra di "
     "Phonestra attiva", "Fare clic su una finestra di Phonestra e incollare di nuovo."],
    [ui("Password non inviata al telefono"), "Il testo viene da un gestore di password", "È voluto: scriverla a mano "
     "o copiarla sul telefono."],
    ["All'apertura arrivano avvisi di notifiche vecchie", "Problema noto", "Solo all'avvio: gli avvisi successivi "
     "riguardano le notifiche nuove."],
    ["Nessun avviso a comparsa", ui("Avviso a comparsa") + " spento, app silenziata, «Non disturbare» del desktop",
     "Controllare " + ui("Preferenze") + " › " + ui("NOTIFICHE") + " e il desktop."],
    "File",
    ["Installazione rifiutata", "Il telefono dice perché nel messaggio", "Vedere " + rif("Installare un'app") + "."],
    [ui("Cartella non leggibile") + " in " + ui("Ricevi file…"), ui("Il telefono non la mostra al PC."),
     "È una cartella privata di un'app o del sistema: non si può ricevere."],
    [ui("Aspetta la fine del trasferimento in corso"), "Un trasferimento è già in corso", "Aspettare che finisca, "
     "o annullarlo con la " + ui("×") + "."],
    "Avvio",
    ["Il file dell'AppImage non parte", "Non è eseguibile", "Renderlo eseguibile (" + rif("Scaricare Phonestra") + ")."],
    ["Riavviando Phonestra si apre il drawer già aperto", "Phonestra era già aperto", "È voluto: un solo Phonestra "
     "alla volta."],
], "«TAB» — Problemi frequenti")

S2 = table(["Messaggio", "Dove", "Significato"], [
    [ui("Collegamento a <nome>…"), "Pagina " + ui("App"), "Phonestra cerca il telefono."],
    [ui("Lettura delle app…"), "Pagina " + ui("App"), "Phonestra legge l'elenco delle app dal telefono."],
    [ui("Nessuna app trovata"), "Pagina " + ui("App"), "Nessuna app col nome cercato."],
    [ui("Telefono bloccato"), "Schermo del telefono nel drawer", "Il telefono va sbloccato."],
    [ui("Collegamento perso"), "Schermo del telefono nel drawer", "Phonestra riprova da solo."],
    [ui("Riconnessione…"), "Finestra di un'app", "Phonestra riprova da solo; l'app torna dov'era."],
    [ui("scollegato"), "Sottotitolo di una finestra", "Il collegamento è caduto."],
    [ui("Phonestra non parte sul telefono"), "Pillola, schermo nel drawer, finestre", "Vedere " + rif("Problemi frequenti")
     + "."],
    [ui("Il telefono non è collegato"), "Messaggio breve nel drawer", "L'azione richiede il collegamento."],
    [ui("Registrazione non avviata: …"), "Finestra di un'app", "La registrazione non è partita; segue il motivo."],
    [ui("Screenshot non salvato: …"), "Finestra di un'app", "Il file non si è potuto scrivere; segue il motivo."],
    [ui("Nome non salvato: …") + ", " + ui("Preferenza non salvata: …"), "Drawer", "Phonestra non riesce a "
     "scrivere nella sua cartella " + c("~/.config/Phonestra") + "."],
], "«TAB» — I messaggi di Phonestra")

S3 = p("Phonestra scrive un registro di quello che fa: collegamenti, cadute, schermo acceso e spento, errori. Serve "
       "a chi aiuta a capire un problema.", lead=True) + ul([
    "Avviando Phonestra da un terminale, le righe del registro compaiono nel terminale.",
    "Avviandolo con un doppio clic, di solito finiscono nel registro del sistema (il journal) e si leggono col "
    "comando qui sotto.",
]) + term("""
$ journalctl --user --since today | grep -i phonestra
""", "Le righe di oggi") + \
    note("il registro può contenere il nome del telefono e i nomi delle app. Prima di mandarlo a qualcuno, "
         "rileggerlo e togliere quello che non si vuole condividere.", "Prima di condividerlo.") + \
    tip("insieme al registro è utile una foto della finestra con il messaggio, e la versione di Phonestra ("
        + ui("Informazioni") + ").", "Cosa mandare.")

S4 = ul([
    "Un solo telefono attivo alla volta (" + rif("Più telefoni") + ").",
    "Il telefono deve restare acceso, sbloccato e sulla stessa rete Wi-Fi del PC.",
    "Le chiamate, anche quelle delle app, si sentono e si fanno dal telefono. Gli SMS si leggono e si scrivono con "
    "l'app dei messaggi, in una finestra, come le altre app.",
    "Il microfono e la webcam del PC non arrivano al telefono; il telefono non fa da webcam per il PC.",
    "Le schermate protette restano nere; le app che vietano la cattura dell'audio restano mute.",
    "Le notifiche nascoste in Phonestra restano sul telefono: Phonestra non cancella le notifiche del telefono.",
    "Le " + ui("Opzioni sviluppatore") + " e il " + ui("Debug wireless") + " si accendono solo a mano, sul telefono: "
    "Android non permette di farlo da un programma.",
    "Si installano solo file " + c(".apk") + " singoli.",
    "Phonestra è fatto per Linux su processori Intel o AMD a 64 bit.",
])

CHAPTER = ("Risoluzione dei problemi", [
    ("Problemi frequenti", S1),
    ("I messaggi di Phonestra", S2),
    ("Il registro di Phonestra", S3),
    ("Limiti da conoscere", S4),
])
