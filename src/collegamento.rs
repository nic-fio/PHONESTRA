//! Il collegamento al telefono attivo, uno per processo, condiviso da drawer e
//! finestre delle app (SPECIFICHE §5.3, §5.7, §5.9):
//!
//! - lo trova in rete e si ricollega da solo quando cade (debug riavviato allo
//!   sblocco, Wi-Fi perso), subito con [`Collegamento::riconnetti_ora`];
//! - controlla ogni 3 s che risponda, se è bloccato e le sue notifiche;
//!   ogni 30 s batteria e rete;
//! - allunga il tempo di spegnimento dello schermo finché Phonestra è aperto e
//!   lo rimette com'era alla fine;
//! - conta le sessioni aperte, per il pannello del telefono;
//! - avvia una volta per collegamento il componente nostro sul telefono,
//!   condiviso da audio, appunti, finestre e drawer; se non parte o continua a
//!   fermarsi lo dice all'utente ([`Collegamento::guasto`]);
//! - fa suonare l'audio del telefono dalle casse del PC e porta al PC le
//!   copie fatte sul telefono.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::sync::{Notify, watch};

use crate::adb::Adb;
use crate::componente::{Componente, Condiviso};
use crate::appunti;
use crate::configurazione::{self, Telefoni};
use crate::notifiche::{self, Info, Notifica};
use crate::rete;

/// Tempo di spegnimento dello schermo (ms) finché Phonestra è aperto: a
/// telefono addormentato e bloccato le app nei display virtuali non ricevono
/// input (SPECIFICHE §5.9).
pub const SPEGNIMENTO_LUNGO: u64 = 30 * 60 * 1000;

/// Quanto aspettare lo specchio del drawer prima di avviare comunque l'audio.
const ATTESA_SPECCHIO: Duration = Duration::from_secs(10);
/// Attesa dopo lo specchio prima di avviare la cattura audio (§49).
const ASSESTAMENTO: Duration = Duration::from_secs(5);
/// Avvii non riusciti o cadute del componente nostro (servizio che muore col
/// telefono ancora collegato) dopo cui si smette di riprovare e lo si dice
/// all'utente, fino al prossimo collegamento o a «Riconnetti ora».
const CADUTE_MASSIME: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stato {
    Cerco,
    Collegato,
    /// Collegato, ma il telefono è bloccato: le app non ricevono input.
    Bloccato,
    /// Collegamento perso: si riprova da soli.
    Perso,
    /// Phonestra si sta chiudendo o si è chiuso.
    Chiuso,
}

pub struct Collegamento {
    pub seriale: String,
    /// Nome del telefono mostrato all'utente.
    pub nome: String,
    adb: watch::Sender<Option<Adb>>,
    componente: watch::Sender<Option<Condiviso>>,
    /// Il componente nostro non parte o continua a fermarsi: cosa dire all'utente.
    guasto: watch::Sender<Option<String>>,
    /// «Riconnetti ora» col componente guasto: si riprova ad avviarlo.
    riprova: Notify,
    stato: watch::Sender<Stato>,
    riconnetti: Notify,
    chiusura: watch::Sender<bool>,
    sessioni: AtomicUsize,
    /// Specchi dello schermo principale aperti col componente (conta le aperture).
    specchi: watch::Sender<u64>,
    info: watch::Sender<Info>,
    notifiche: watch::Sender<Vec<Notifica>>,
    densita: AtomicU32,
    /// Copie fatte sul telefono da mettere negli appunti del PC (numero
    /// progressivo: la stessa copia ripetuta conta di nuovo).
    appunti: watch::Sender<(u64, String)>,
    /// Testi messi da Phonestra negli appunti del telefono, con l'ora: se il
    /// telefono li rimanda come copia, non devono tornare al PC.
    inviati: std::sync::Mutex<Vec<(String, std::time::Instant)>>,
    /// Lato corto / lato lungo dello schermo del telefono (× 10000; 0 se
    /// ancora sconosciuto).
    proporzione: AtomicU32,
    /// L'utente ha sbloccato il telefono a mano durante l'uso: lo sta usando in
    /// mano, il pannello resta acceso finché non torna a usarlo dal PC.
    a_mano: std::sync::atomic::AtomicBool,
    /// Il telefono si è bloccato (o il collegamento è caduto) dopo il primo
    /// collegamento: al ritorno lo ha sbloccato l'utente.
    bloccato_durante_uso: std::sync::atomic::AtomicBool,
}

impl Collegamento {
    /// Il telefono configurato (per ora il primo: un solo telefono attivo).
    pub fn primo_telefono() -> Result<Arc<Self>> {
        let t = Telefoni::carica()?
            .elenco
            .into_iter()
            .next()
            .context("nessun telefono configurato: prima «phonestra-prova prepara» col cavo")?;
        Ok(Arc::new(Self {
            seriale: t.seriale,
            nome: t.nome,
            adb: watch::channel(None).0,
            componente: watch::channel(None).0,
            guasto: watch::channel(None).0,
            riprova: Notify::new(),
            stato: watch::channel(Stato::Cerco).0,
            riconnetti: Notify::new(),
            chiusura: watch::channel(false).0,
            sessioni: AtomicUsize::new(0),
            specchi: watch::channel(0).0,
            info: watch::channel(Info::default()).0,
            notifiche: watch::channel(Vec::new()).0,
            densita: AtomicU32::new(0),
            appunti: watch::channel((0, String::new())).0,
            inviati: std::sync::Mutex::new(Vec::new()),
            proporzione: AtomicU32::new(0),
            a_mano: std::sync::atomic::AtomicBool::new(false),
            bloccato_durante_uso: std::sync::atomic::AtomicBool::new(false),
        }))
    }

    /// Il collegamento ADB quando c'è (`None` mentre si cerca o è perso).
    pub fn adb(&self) -> watch::Receiver<Option<Adb>> {
        self.adb.subscribe()
    }

    /// Uno specchio dello schermo principale (drawer) è stato aperto col
    /// componente: la cattura audio va (ri)avviata dopo (prove §49).
    pub fn specchio_aperto(&self) {
        self.specchi.send_modify(|n| *n += 1);
    }

    /// Il componente nostro del collegamento, per video, input e appunti:
    /// `None` finché non è partito (o mentre riparte, o se è guasto).
    pub fn componente(&self) -> watch::Receiver<Option<Condiviso>> {
        self.componente.subscribe()
    }

    /// Il componente nostro non parte sul telefono (o continua a fermarsi):
    /// `Some` con la spiegazione per l'utente. Si riprova al prossimo
    /// collegamento o con [`Collegamento::riconnetti_ora`].
    pub fn guasto(&self) -> watch::Receiver<Option<String>> {
        self.guasto.subscribe()
    }

    pub fn stato(&self) -> watch::Receiver<Stato> {
        self.stato.subscribe()
    }

    /// Batteria e rete del telefono (aggiornate ogni 30 s).
    pub fn info(&self) -> watch::Receiver<Info> {
        self.info.subscribe()
    }

    /// Notifiche attive sul telefono, la più recente per prima.
    pub fn notifiche(&self) -> watch::Receiver<Vec<Notifica>> {
        self.notifiche.subscribe()
    }

    /// Densità dello schermo del telefono (dpi), letta al collegamento; 0 se
    /// ancora sconosciuta.
    pub fn densita(&self) -> u32 {
        self.densita.load(Ordering::SeqCst)
    }

    /// Lato corto / lato lungo dello schermo del telefono (0,46 finché non si
    /// conosce: la forma dei telefoni più comuni).
    pub fn proporzione(&self) -> f32 {
        match self.proporzione.load(Ordering::SeqCst) {
            0 => 0.46,
            p => p as f32 / 10000.0,
        }
    }

    /// Copie fatte sul telefono, da mettere negli appunti del PC.
    pub fn appunti(&self) -> watch::Receiver<(u64, String)> {
        self.appunti.subscribe()
    }

    pub(crate) fn appunti_dal_telefono(&self, testo: String) {
        self.appunti.send_modify(|(n, t)| {
            *n += 1;
            *t = testo;
        });
    }

    /// Phonestra sta per mettere `testo` negli appunti del telefono.
    pub fn ricorda_inviato(&self, testo: &str) {
        let mut inviati = self.inviati.lock().unwrap();
        inviati.retain(|(_, quando)| quando.elapsed() < Duration::from_secs(5));
        inviati.push((testo.to_string(), std::time::Instant::now()));
    }

    /// Se la copia arrivata dal telefono l'ha appena messa Phonestra.
    pub(crate) fn e_un_rimbalzo(&self, testo: &str) -> bool {
        let inviati = self.inviati.lock().unwrap();
        inviati.iter().any(|(t, quando)| t == testo && quando.elapsed() < Duration::from_secs(5))
    }

    /// «Riconnetti ora»: nuovo tentativo subito, senza aspettare; col
    /// componente guasto, nuovo tentativo di avviarlo.
    pub fn riconnetti_ora(&self) {
        self.riconnetti.notify_one();
        self.riprova.notify_waiters();
    }

    /// Chiude tutto e rimette il telefono com'era; lo stato diventa `Chiuso`.
    pub fn chiudi(&self) {
        eprintln!("[collegamento] chiusura richiesta");
        self.chiusura.send_replace(true);
    }

    /// Se il telefono è in mano all'utente: le sessioni non spengono il pannello.
    pub fn pannello_a_mano(&self) -> bool {
        self.a_mano.load(Ordering::SeqCst)
    }

    /// L'utente usa il telefono dal PC (clic, tasti): se lo stava usando in
    /// mano, ora il pannello si può rispegnere. Restituisce `true` in quel caso.
    pub fn usa_dal_pc(&self) -> bool {
        self.a_mano.swap(false, Ordering::SeqCst)
    }

    /// Il telefono torna usabile: se si era bloccato durante l'uso, l'ha
    /// sbloccato l'utente a mano.
    fn sbloccato(&self) {
        if self.bloccato_durante_uso.swap(false, Ordering::SeqCst) {
            self.a_mano.store(true, Ordering::SeqCst);
            eprintln!("[collegamento] sbloccato a mano: il pannello resta acceso");
        }
    }

    /// Una finestra ha avviato una sessione sul telefono.
    pub fn sessione_aperta(&self) {
        self.sessioni.fetch_add(1, Ordering::SeqCst);
    }

    /// Sessioni aperte in questo momento.
    pub fn sessioni(&self) -> usize {
        self.sessioni.load(Ordering::SeqCst)
    }

    /// Una sessione è finita.
    pub fn sessione_chiusa(&self) {
        self.sessioni.fetch_sub(1, Ordering::SeqCst);
    }

    /// Mantiene il collegamento finché non si chiama [`Collegamento::chiudi`].
    pub async fn mantieni(self: Arc<Self>) {
        let mut chiusura = self.chiusura.subscribe();
        let mut originale: Option<Originali> = None;
        let mut attesa = 2;
        while !*chiusura.borrow() {
            self.stato.send_replace(Stato::Cerco);
            match tokio::time::timeout(Duration::from_secs(30), self.apri()).await {
                Ok(Ok(adb)) => {
                    attesa = 2;
                    if let Err(e) = self.usa(adb, &mut originale, &mut chiusura).await {
                        eprintln!("[collegamento] {e:#}");
                    }
                    self.adb.send_replace(None);
                    self.componente.send_replace(None);
                    self.guasto.send_replace(None);
                }
                Ok(Err(e)) => eprintln!("[collegamento] non riuscito: {e:#}"),
                Err(_) => eprintln!("[collegamento] il telefono non risponde"),
            }
            if *chiusura.borrow() {
                break;
            }
            // Sui Samsung il blocco fa cadere il collegamento: al ritorno il
            // telefono l'ha sbloccato l'utente (o è tornato il Wi-Fi).
            if self.adb.borrow().is_none() && *self.stato.borrow() != Stato::Cerco {
                self.bloccato_durante_uso.store(true, Ordering::SeqCst);
            }
            self.stato.send_replace(Stato::Perso);
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(attesa)) => {}
                _ = self.riconnetti.notified() => {}
                _ = async { chiusura.wait_for(|c| *c).await.is_ok() } => {}
            }
            attesa = (attesa * 2).min(10);
        }
        eprintln!("[collegamento] chiuso");
        self.stato.send_replace(Stato::Chiuso);
    }

    /// Trova il telefono in rete e apre il collegamento cifrato.
    async fn apri(&self) -> Result<Adb> {
        let ultimo = Telefoni::carica()?.elenco.into_iter().find(|t| t.seriale == self.seriale).and_then(|t| t.ultimo_indirizzo);
        let seriale = self.seriale.clone();
        let indirizzo = tokio::task::spawn_blocking(move || rete::indirizzo_attivo(&seriale, ultimo)).await??;
        Telefoni::ricorda_indirizzo(&self.seriale, indirizzo)?;
        Adb::wifi(indirizzo, &configurazione::chiave()?).await
    }

    /// Un collegamento riuscito: lo pubblica e lo controlla finché cade o si chiude.
    async fn usa(self: &Arc<Self>, adb: Adb, originali: &mut Option<Originali>, chiusura: &mut watch::Receiver<bool>) -> Result<()> {
        // Letti una volta sola per tutti i ricollegamenti: dopo una caduta il
        // telefono potrebbe avere ancora i valori della sessione precedente.
        let Originali { spegnimento: originale, volume } = match *originali {
            Some(o) => o,
            None => *originali.insert(Originali {
                spegnimento: self.spegnimento_originale(&adb).await?,
                volume: self.volume_originale(&adb).await?,
            }),
        };
        // Densità attuale (quella scelta dall'utente se l'ha cambiata).
        let densita = adb.esegui("wm density; wm size").await?;
        let valore = |nome: &str| densita.lines().find_map(|r| r.strip_prefix(nome)).map(str::trim);
        if let Some(d) = valore("Override density:").or_else(|| valore("Physical density:")).and_then(|v| v.parse::<u32>().ok()) {
            self.densita.store(d, Ordering::SeqCst);
        }
        // Forma dello schermo, per la colonna delle app solo verticali.
        let misura = valore("Override size:").or_else(|| valore("Physical size:")).and_then(|v| v.split_once('x'));
        if let Some((Ok(l), Ok(a))) = misura.map(|(l, a)| (l.parse::<u32>(), a.parse::<u32>()))
            && l > 0
            && a > 0
        {
            self.proporzione.store(l.min(a) * 10000 / l.max(a), Ordering::SeqCst);
        }

        // Custode: allunga il tempo di spegnimento e porta al massimo il volume
        // multimediale, poi rimette tutti e due quando il canale si chiude,
        // anche se il PC sparisce (Wi-Fi perso, PC spento): `cat` finisce
        // quando adbd chiude il suo ingresso. Senza `trap` adbd lo termina col
        // segnale e il ripristino non avviene (provato). Il volume: col valore
        // a 0 l'app di Facebook non avvia l'audio dei reel (provato il 28 set);
        // l'audio esce comunque solo dal PC finché dura il collegamento. Il
        // volume si rimette prima del tempo di spegnimento, che la chiusura
        // controlla per sapere che il custode ha finito.
        let (alza, rimetti) = match volume {
            Some(Volume { attuale, massimo }) => (
                format!("{VOLUME} --set {massimo} >/dev/null 2>&1; "),
                format!("{VOLUME} --set {attuale} >/dev/null 2>&1; "),
            ),
            None => (String::new(), String::new()),
        };
        let custode = adb
            .apri(&format!(
                "exec:trap '' HUP TERM PIPE; settings put system screen_off_timeout {SPEGNIMENTO_LUNGO}; {alza}\
                 cat >/dev/null; {rimetti}settings put system screen_off_timeout {originale}"
            ))
            .await
            .context("custode del tempo di spegnimento e del volume")?;
        self.sbloccato();
        self.adb.send_replace(Some(adb.clone()));
        self.stato.send_replace(Stato::Collegato);

        // Componente nostro (video, input, audio, appunti), finché dura il
        // collegamento; l'audio del telefono suona dalle casse del PC.
        let (ferma_componente, ferma) = watch::channel(false);
        let mut componente = FermaAllaFine(tokio::spawn(self.clone().gira_componente(adb.clone(), ferma)));

        // Copie fatte sul telefono → appunti del PC.
        let appunti = {
            let io = self.clone();
            tokio::spawn(async move {
                if let Err(e) = appunti::ascolta(&io).await {
                    eprintln!("[appunti] {e:#}");
                }
            })
        };
        let _ferma_appunti = FermaAllaFine(appunti);

        let mut era_bloccato = false;
        let mut giro = 0u32;
        loop {
            // Batteria e rete subito e poi ogni 30 s.
            if giro.is_multiple_of(10) {
                let domanda = adb.esegui(notifiche::COMANDO_INFO);
                if let Ok(Ok(uscita)) = tokio::time::timeout(Duration::from_secs(5), domanda).await {
                    self.info.send_if_modified(|i| {
                        let nuova = notifiche::leggi_info(&uscita);
                        let cambiata = *i != nuova;
                        *i = nuova;
                        cambiata
                    });
                }
            }
            giro += 1;
            // Blocco e notifiche in un solo comando: fa anche da controllo
            // che il telefono risponda. Le schermate protette le segnala il
            // componente nostro, sessione per sessione.
            let comando =
                format!("dumpsys window | grep -m1 -o 'isKeyguardShowing=[a-z]*'; {}", notifiche::COMANDO_NOTIFICHE);
            let domanda = adb.esegui(&comando);
            let Ok(Ok(risposta)) = tokio::time::timeout(Duration::from_secs(5), domanda).await else {
                bail!("il telefono non risponde più");
            };
            let bloccato = risposta.lines().next().is_some_and(|r| r.ends_with("true"));
            if bloccato != era_bloccato {
                if bloccato {
                    self.bloccato_durante_uso.store(true, Ordering::SeqCst);
                } else {
                    self.sbloccato();
                }
                self.stato.send_replace(if bloccato { Stato::Bloccato } else { Stato::Collegato });
                era_bloccato = bloccato;
            }
            self.notifiche.send_if_modified(|n| {
                let nuove = notifiche::leggi(&risposta);
                let cambiate = *n != nuove;
                *n = nuove;
                cambiate
            });
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(3)) => {}
                _ = async { chiusura.wait_for(|c| *c).await.is_ok() } => break,
            }
        }

        // Chiusura: prima le finestre chiudono le loro sessioni, poi il custode
        // rimette il tempo di spegnimento; si verifica che l'abbia fatto.
        eprintln!("[collegamento] chiusura: {} sessioni aperte", self.sessioni());
        // Musica o video ancora in riproduzione: in pausa prima di staccare
        // l'audio, altrimenti ripartirebbero dall'altoparlante del telefono.
        let pausa = "dumpsys media_session | grep -q 'state=PLAYING' && cmd media_session dispatch pause";
        if let Ok(Ok(_)) = tokio::time::timeout(Duration::from_secs(3), adb.esegui(pausa)).await {
            eprintln!("[collegamento] chiusura: riproduzione in pausa (se c'era)");
        }
        self.adb.send_replace(None);
        for _ in 0..50 {
            if self.sessioni() == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        // Il componente nostro si chiude dopo le sessioni (che gli chiedono di
        // togliere le app dalle recenti), prima del custode.
        ferma_componente.send_replace(true);
        if tokio::time::timeout(Duration::from_secs(12), &mut componente.0).await.is_err() {
            eprintln!("[collegamento] chiusura: il componente non si è chiuso in tempo");
        }
        eprintln!("[collegamento] chiusura: sessioni rimaste {}, chiudo il custode", self.sessioni());
        custode.chiudi().await?;
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if adb.esegui("settings get system screen_off_timeout").await?.trim() == originale.to_string() {
                Telefoni::ricorda_spegnimento(&self.seriale, None)?;
                Telefoni::ricorda_volume(&self.seriale, None)?;
                return Ok(());
            }
        }
        // Riserva: li si rimette direttamente.
        if let Some(Volume { attuale, .. }) = volume {
            adb.esegui(&format!("{VOLUME} --set {attuale}")).await?;
        }
        adb.esegui(&format!("settings put system screen_off_timeout {originale}")).await?;
        if adb.esegui("settings get system screen_off_timeout").await?.trim() == originale.to_string() {
            eprintln!("[collegamento] il custode non ha ripristinato: fatto direttamente");
            Telefoni::ricorda_spegnimento(&self.seriale, None)?;
            Telefoni::ricorda_volume(&self.seriale, None)?;
            return Ok(());
        }
        bail!("tempo di spegnimento non ripristinato: lo si rimette al prossimo avvio")
    }

    /// Il componente nostro del collegamento: lo avvia, lo pubblica per
    /// finestre, drawer e appunti e ci fa suonare l'audio; se muore col
    /// telefono ancora collegato lo riavvia. Se non parte o continua a fermarsi
    /// ([`CADUTE_MASSIME`] volte) lo dice all'utente ([`Collegamento::guasto`])
    /// e aspetta il prossimo collegamento o «Riconnetti ora». Finisce quando
    /// `ferma` diventa vero, chiudendo il componente in ordine (il custode sul
    /// telefono riaccende il pannello).
    async fn gira_componente(self: Arc<Self>, adb: Adb, mut ferma: watch::Receiver<bool>) {
        let mut cadute = 0;
        let mut ultimo_errore = String::new();
        loop {
            if cadute >= CADUTE_MASSIME {
                eprintln!("[componente] non riuscito {cadute} volte: fermo fino a «Riconnetti ora» o al prossimo collegamento");
                self.guasto.send_replace(Some(ultimo_errore.clone()));
                tokio::select! {
                    _ = self.riprova.notified() => {}
                    _ = async { ferma.wait_for(|f| *f).await.is_ok() } => return,
                }
                eprintln!("[componente] nuovo tentativo chiesto dall'utente");
                cadute = 0;
                self.guasto.send_replace(None);
            }
            let avvio = tokio::select! {
                r = Componente::avvia(&adb) => r,
                _ = async { ferma.wait_for(|f| *f).await.is_ok() } => return,
            };
            let servizio = match avvio {
                Ok(c) => Condiviso::avvia(c),
                Err(e) => {
                    cadute += 1;
                    eprintln!("[componente] non avviato ({cadute}ª volta): {e:#}");
                    ultimo_errore = format!("non parte: {e:#}");
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                        _ = async { ferma.wait_for(|f| *f).await.is_ok() } => return,
                    }
                    continue;
                }
            };
            // Gli specchi contati finora sono dei collegamenti (o dei servizi)
            // precedenti: l'audio deve aspettare uno specchio aperto con questo
            // servizio. Letto prima di pubblicarlo, perché il drawer può
            // aprire lo specchio subito.
            let specchi_prima = *self.specchi.borrow();
            self.componente.send_replace(Some(servizio.clone()));
            let audio = async {
                // La cattura audio parte DOPO lo specchio dello schermo principale
                // e riparte quando lo specchio si ricrea: uno specchio nato dopo la
                // cattura fa interrompere l'audio dei reel di Facebook nelle
                // finestre (prove §48–49, verificato a prove alternate).
                let mut specchi = self.specchi.subscribe();
                let aperto = tokio::time::timeout(ATTESA_SPECCHIO, specchi.wait_for(|n| *n > specchi_prima)).await.is_ok();
                crate::diagnosi(&format!(
                    "audio: {}, cattura tra {} s",
                    if aperto { "specchio aperto" } else { "specchio non arrivato" },
                    ASSESTAMENTO.as_secs()
                ));
                // E dopo che il collegamento si è assestato (sessioni iniziali,
                // elenco delle app, sfondo): subito dopo lo specchio i vuoti
                // restavano (36), con 8 s di attesa no (13 in 3 istanti), §49.
                tokio::time::sleep(ASSESTAMENTO).await;
                loop {
                    specchi.borrow_and_update();
                    tokio::select! {
                        esito = crate::audio_nostro::riproduci(servizio.apritore()) => {
                            if let Err(e) = esito {
                                eprintln!("[audio] {e:#}");
                            }
                            break;
                        }
                        r = specchi.changed() => {
                            if r.is_err() {
                                break;
                            }
                            crate::diagnosi("audio: specchio ricreato, riavvio la cattura");
                            // Lascia al telefono il tempo di togliere la cattura vecchia.
                            tokio::time::sleep(Duration::from_millis(300)).await;
                        }
                    }
                }
                // Senza audio il componente serve ancora a finestre e drawer.
                std::future::pending::<()>().await
            };
            tokio::select! {
                _ = audio => {}
                _ = servizio.finito() => {
                    // Servizio finito da solo, o telefono perso (allora cade
                    // anche il collegamento e questo compito viene fermato).
                    cadute += 1;
                    eprintln!("[componente] il servizio sul telefono si è fermato ({cadute}ª volta): lo riavvio");
                    ultimo_errore = "si è fermato più volte da solo".into();
                    self.componente.send_replace(None);
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                        _ = async { ferma.wait_for(|f| *f).await.is_ok() } => return,
                    }
                    continue;
                }
                _ = async { ferma.wait_for(|f| *f).await.is_ok() } => {
                    match servizio.chiudi().await {
                        Ok(p) => crate::diagnosi(&format!("componente chiuso: {p:?}")),
                        Err(e) => eprintln!("[componente] chiusura: {e:#}"),
                    }
                    self.componente.send_replace(None);
                    return;
                }
            }
        }
    }

    /// Tempo di spegnimento dell'utente, salvato in `telefoni.toml`: se
    /// Phonestra è caduto lasciandolo allungato, vale quello salvato.
    async fn spegnimento_originale(&self, adb: &Adb) -> Result<u64> {
        let salvato =
            Telefoni::carica()?.elenco.into_iter().find(|t| t.seriale == self.seriale).and_then(|t| t.spegnimento_originale);
        let attuale: u64 =
            adb.esegui("settings get system screen_off_timeout").await?.trim().parse().unwrap_or(SPEGNIMENTO_LUNGO);
        let originale = match salvato {
            Some(v) if attuale == SPEGNIMENTO_LUNGO || attuale < 5000 => v,
            _ => attuale,
        };
        Telefoni::ricorda_spegnimento(&self.seriale, Some(originale))?;
        Ok(originale)
    }

    /// Volume multimediale dell'utente, salvato in `telefoni.toml` come il
    /// tempo di spegnimento: se Phonestra è caduto lasciandolo al massimo,
    /// vale quello salvato. `None` se il telefono non lo dice.
    async fn volume_originale(&self, adb: &Adb) -> Result<Option<Volume>> {
        let Some(mut letto) = Volume::leggi(&adb.esegui(&format!("{VOLUME} --get")).await?) else {
            return Ok(None);
        };
        let salvato = Telefoni::carica()?.elenco.into_iter().find(|t| t.seriale == self.seriale).and_then(|t| t.volume_originale);
        if let Some(v) = salvato
            && letto.attuale == letto.massimo
        {
            letto.attuale = v.min(letto.massimo);
        }
        Telefoni::ricorda_volume(&self.seriale, Some(letto.attuale))?;
        Ok(Some(letto))
    }
}

/// Valori dell'utente che il collegamento cambia e poi rimette.
#[derive(Clone, Copy)]
struct Originali {
    spegnimento: u64,
    volume: Option<Volume>,
}

/// Comando per il volume multimediale (flusso 3, `STREAM_MUSIC`).
const VOLUME: &str = "cmd media_session volume --stream 3";

#[derive(Clone, Copy, Debug, PartialEq)]
struct Volume {
    attuale: u32,
    massimo: u32,
}

impl Volume {
    /// Da «[V] volume is 7 in range [0..15]».
    fn leggi(risposta: &str) -> Option<Self> {
        let (_, resto) = risposta.split_once("volume is ")?;
        let (attuale, resto) = resto.split_once(" in range [")?;
        let (_, massimo) = resto.split_once("..")?;
        let massimo = massimo.split_once(']')?.0;
        Some(Volume { attuale: attuale.trim().parse().ok()?, massimo: massimo.trim().parse().ok()? })
    }
}

/// Ferma un compito quando esce di scena (anche per un errore con `?`).
struct FermaAllaFine(tokio::task::JoinHandle<()>);

impl Drop for FermaAllaFine {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn legge_volume_e_massimo() {
        let v = Volume::leggi("[V] Connecting to AudioService\n[V] volume is 7 in range [0..15]\n");
        assert_eq!(v, Some(Volume { attuale: 7, massimo: 15 }));
        assert_eq!(Volume::leggi("cmd: Failure calling service"), None);
    }
}
