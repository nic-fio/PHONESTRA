from build import note, p, rif, table, tip, ul, warn

S1 = p("Finché Phonestra è collegato, l'audio delle app del telefono esce dalle casse (o dalle cuffie) del PC. Il "
       "telefono resta muto.", lead=True) + ul([
    "Vale per tutte le app: musica, video, giochi, suoni delle notifiche, navigatore.",
    "L'audio comincia pochi secondi dopo il collegamento: nei primi istanti può mancare.",
    "Audio e video restano in sincrono, anche nelle registrazioni (" + rif("Registrare lo schermo") + ").",
    "Alla chiusura di Phonestra, la musica o il video che suonavano vanno in pausa: il telefono non riparte a "
    "suonare da solo dall'altoparlante.",
]) + note("alcune app vietano di catturare il loro audio. Il loro suono non arriva al PC: con Phonestra collegato "
          "restano mute.", "App che vietano la cattura.")

S2 = p("Il volume si regola sul PC, come per ogni altro programma: con i tasti del volume della tastiera o con il "
       "mixer del desktop, dove Phonestra compare come un programma che suona.", lead=True) + \
    p("Durante il collegamento Phonestra porta al massimo il volume multimediale del telefono: alcune app, con il "
      "volume del telefono a zero, non fanno partire l'audio. Il telefono non suona lo stesso, perché l'audio esce "
      "solo dal PC. Alla chiusura Phonestra rimette il volume di prima.") + \
    warn("in qualche caso, alla chiusura, il volume multimediale del telefono può restare al massimo. È un problema "
         "noto: se succede, basta abbassarlo con i tasti del telefono.", "Volume rimasto alto.") + \
    tip("i tasti del volume del telefono non servono mentre si usa Phonestra: conta il volume del PC.",
        "Tasti del telefono.")

S3 = p("Le chiamate restano sul telefono: si risponde e si parla col telefono in mano.", lead=True) + \
    table(["Tipo di chiamata", "Dove si sente la voce", "Dove si parla"], [
        ["Telefonata normale", "Dal telefono", "Nel microfono del telefono"],
        ["Chiamata o videochiamata di un'app (per esempio WhatsApp) aperta in una finestra di Phonestra",
         "Dal telefono", "Nel microfono del telefono"],
    ], "«TAB» — Le chiamate con Phonestra") + ul([
        "Quando arriva una chiamata, Phonestra riaccende lo schermo del telefono, così si può rispondere dal "
        "telefono (" + rif("Le chiamate in arrivo") + ").",
        "Android non permette di portare al PC la voce delle chiamate: anche con Phonestra si sente dal telefono.",
        "Il microfono del PC non arriva al telefono.",
    ])

S4 = p("Le app aperte in una finestra di Phonestra usano la fotocamera e il microfono <b>del telefono</b>, come "
       "sempre.", lead=True) + ul([
    "Una videochiamata aperta in una finestra funziona: si vede sul PC, ma riprende e ascolta dal telefono.",
    "Per fotografare con l'app " + "Fotocamera" + " in una finestra, si punta il telefono e si scatta col clic.",
    "Il telefono non diventa una webcam o un microfono per i programmi del PC.",
])

CHAPTER = ("Audio, chiamate e fotocamera", [
    ("L'audio dal PC", S1),
    ("Il volume", S2),
    ("Le chiamate", S3),
    ("Fotocamera e microfono", S4),
])
