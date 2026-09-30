from build import VERSION, c, p, pill, table

S1 = p("Aperti alla versione " + VERSION + "; lo stato aggiornato è in " + c("memoria/registro-problemi.md") + ".",
       lead=True) + table(["Problema", "Stato"], [
    ["Volume originale salvato come 15 invece del valore dell'utente: alla chiusura il telefono può restare al massimo",
     pill("accettato", "off") + " dall'utente: il volume si abbassa a mano (" + c("Collegamento::volume_originale") + ")"],
    ["Il margine dell'audio cresce ma non scende: " + c("Margine::scendi") + " non scatta perché i pacchetti AAC hanno "
     "dimensione quasi costante", pill("da fare", "wait") + " silenzio segnato dal telefono"],
    ["Col " + c("delayed ack") + " annunciato adbd rifiuta ogni canale", pill("spento", "off") + " da indagare ("
     + c("memoria/adb.md") + ")"],
    ["Memoria del servizio circa 145 MB (costo di partenza di ART)", pill("da misurare", "wait")],
    ["Perché l'ordine d'avvio dell'audio conta", pill("regola trovata", "ok") + " con le misure; meccanismo di "
     "Android da capire"],
    ["Caduta del Wi-Fi del PC, registrazione con audio AAC su telefoni diversi", pill("da provare", "wait")],
    ["Chiamata vera dopo la rc.7: pannello acceso alla chiamata in arrivo e rispento dopo la fine", pill("da provare", "wait")],
    ["«Ricevi file…»: scheda SD, annullamento, cartelle grandi, tema scuro", pill("da provare", "wait")],
    ["Avvisi del sistema: alla prima lettura dopo l'apertura del drawer le notifiche già presenti sul telefono "
     "fanno un avviso, perché le «già viste» si fissano quando l'elenco è ancora vuoto (" + c("avvisa_nuove")
     + " in " + c("cassetto.rs") + ")", pill("da verificare", "wait") + " probabile difetto, trovato rileggendo il codice"],
    ["Telefono bloccato: sui Samsung il blocco fa cadere il Debug wireless", pill("aggirato", "ok") + " il "
     "collegamento si riapre allo sblocco e le app tornano dov'erano"],
], "«TAB» — Problemi noti")

CHAPTER = ("Problemi noti", [
    ("Problemi aperti", S1),
])
