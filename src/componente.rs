//! Il componente nostro sul telefono, lato PC: il servizio di lunga durata
//! (`phonestra.Servizio` nell'aiutante), un processo per collegamento. Qui
//! l'infrastruttura: avvio, canali, segreto, battito, custode, autotest,
//! smistamento dei messaggi ([`Condiviso`]). Architettura e formato in `notes/component.md`.
//!
//! 1. Il jar si copia in `/data/local/tmp/phonestra-servizio-<casuale>.jar` e
//!    il servizio parte con `shell,v2,raw:` (niente terminale, errori separati,
//!    codice d'uscita; se il canale cade, adbd manda SIGHUP al servizio).
//! 2. Il segreto (16 byte casuali, in esadecimale) va sull'ingresso del
//!    processo, non fra gli argomenti.
//! 3. Il servizio stampa la riga di pronto col nome del suo socket astratto;
//!    ogni canale si apre con `localabstract:<nome>` e comincia col preambolo
//!    (segreto e tipo).
//! 4. Sul canale `comandi` arriva il `CIAO`, poi il battito nei due sensi ogni
//!    secondo: 5 s di silenzio e ciascuna parte considera l'altra sparita.
//!
//! In Phonestra un solo servizio per collegamento serve audio, video e input:
//! [`Condiviso`] lo tiene in un compito che **smista** i messaggi del canale
//! comandi (`Smistamento`): le risposte a chi ha fatto la domanda, gli eventi
//! di una sessione video a quella sessione, gli altri messaggi spontanei a chi
//! si è iscritto al loro tipo (per esempio, in futuro, gli appunti).

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinHandle;

use crate::adb::shell::{Evento, ShellV2};
use crate::adb::{Adb, Canale, Chiusore, sync};
use crate::t;

/// Versione del protocollo del canale comandi (uguale a `Protocollo.VERSIONE`).
pub const PROTOCOLLO: u32 = 1;

/// Tipi dei messaggi del canale comandi (`Protocollo.java`). 0x01–0x0f
/// infrastruttura, 0x10–0x1f prove e diagnosi, dal 0x20 i pezzi futuri.
pub mod tipo {
    /// Servizio → PC, subito dopo l'apertura: righe `chiave=valore`.
    pub const CIAO: u8 = 0x01;
    /// Nei due sensi, ogni secondo, senza contenuto.
    pub const BATTITO: u8 = 0x02;
    /// PC → servizio: chiudi (il servizio risponde FINE ed esce).
    pub const FINE: u8 = 0x03;
    /// Risposta a una domanda non valida: testo.
    pub const ERRORE: u8 = 0x04;
    /// PC → servizio: prova innocua del custode.
    pub const PROVA_CUSTODE: u8 = 0x10;

    // Video (0x40–0x4f): `video_nostro`, `Video.java`. Contenuti: righe `chiave=valore`.
    /// PC → servizio: nuova sessione video (schermo per un'app o specchio).
    pub const VIDEO_APRI: u8 = 0x40;
    /// PC → servizio: chiudi la sessione (`togli_task=1`: via dalle recenti).
    pub const VIDEO_CHIUDI: u8 = 0x41;
    /// PC → servizio: avvia un'app (o le sue «Informazioni app») sullo schermo.
    pub const VIDEO_AVVIA_APP: u8 = 0x42;
    /// PC → servizio: nuova misura dello schermo.
    pub const VIDEO_RIDIMENSIONA: u8 = 0x43;
    /// PC → servizio: fotogramma chiave appena possibile.
    pub const VIDEO_CHIAVE: u8 = 0x44;
    /// PC → servizio: pannello fisico acceso o spento.
    pub const VIDEO_PANNELLO: u8 = 0x45;
    /// Servizio → PC, spontaneo: evento di una sessione (orientamento, schermata protetta…).
    pub const VIDEO_EVENTO: u8 = 0x46;
}

/// Bandiera: il messaggio risponde a quello con lo stesso id.
pub const RISPOSTA: u8 = 0x01;

/// Contenuto massimo accettato in un messaggio.
const MASSIMO: usize = 16 * 1024 * 1024;

pub const BATTITO_OGNI: Duration = Duration::from_secs(1);
pub const LIMITE_SILENZIO: Duration = Duration::from_secs(5);
const ATTESA_PRONTO: Duration = Duration::from_secs(20);
const ATTESA_CIAO: Duration = Duration::from_secs(10);
const ATTESA_RISPOSTA: Duration = Duration::from_secs(5);

/// Nome del processo del servizio (`--nice-name`) e prefisso del suo jar.
pub const NOME_SERVIZIO: &str = "phonestra-servizio";
/// Nome del processo del custode (il suo `$0`).
pub const NOME_CUSTODE: &str = "phonestra-custode";
/// File che il servizio crea per la prova del custode e che solo il custode toglie.
pub const FILE_PROVA_CUSTODE: &str = "/data/local/tmp/phonestra-prova-custode";
/// Riga di pronto del servizio (seguita da `protocollo=… socket=… pid=…`).
const PRONTO: &str = "phonestra-servizio pronto";
const ERRORE_AVVIO: &str = "phonestra-servizio errore";

/// Un messaggio del canale comandi: intestazione di 8 byte big-endian
/// (`tipo u8 · bandiere u8 · id u16 · lunghezza u32`) e contenuto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Messaggio {
    pub tipo: u8,
    pub bandiere: u8,
    /// Lega una risposta alla sua domanda; 0 = messaggio spontaneo.
    pub id: u16,
    pub dati: Vec<u8>,
}

impl Messaggio {
    pub fn new(tipo: u8, dati: impl Into<Vec<u8>>) -> Self {
        Self { tipo, bandiere: 0, id: 0, dati: dati.into() }
    }

    pub fn risposta(&self) -> bool {
        self.bandiere & RISPOSTA != 0
    }

    pub fn testo(&self) -> String {
        String::from_utf8_lossy(&self.dati).into_owned()
    }

    pub fn in_byte(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(8 + self.dati.len());
        v.push(self.tipo);
        v.push(self.bandiere);
        v.extend_from_slice(&self.id.to_be_bytes());
        v.extend_from_slice(&(self.dati.len() as u32).to_be_bytes());
        v.extend_from_slice(&self.dati);
        v
    }
}

/// Ricompone i messaggi dai blocchi ADB, che possono tagliarli in qualsiasi punto.
#[derive(Debug, Default)]
pub struct Decodificatore {
    ricevuti: Vec<u8>,
}

impl Decodificatore {
    pub fn aggiungi(&mut self, blocco: &[u8]) {
        self.ricevuti.extend_from_slice(blocco);
    }

    /// Il prossimo messaggio completo; errore se l'intestazione è impossibile
    /// (flusso rovinato: il canale va chiuso).
    pub fn prossimo(&mut self) -> Result<Option<Messaggio>> {
        if self.ricevuti.len() < 8 {
            return Ok(None);
        }
        let lunghezza = u32::from_be_bytes(self.ricevuti[4..8].try_into().unwrap()) as usize;
        if lunghezza > MASSIMO {
            bail!("messaggio del componente troppo grande ({lunghezza} byte)");
        }
        if self.ricevuti.len() < 8 + lunghezza {
            return Ok(None);
        }
        let m = Messaggio {
            tipo: self.ricevuti[0],
            bandiere: self.ricevuti[1],
            id: u16::from_be_bytes([self.ricevuti[2], self.ricevuti[3]]),
            dati: self.ricevuti[8..8 + lunghezza].to_vec(),
        };
        self.ricevuti.drain(..8 + lunghezza);
        Ok(Some(m))
    }
}

/// Primo messaggio di ogni canale, dal PC: `segreto (16 byte) · lunghezza del
/// tipo u8 · tipo (ASCII)`. Tipi: `comandi`, in futuro `audio`, `video:<id>`.
pub fn preambolo(segreto: &[u8; 16], tipo: &str) -> Result<Vec<u8>> {
    if !tipo.is_ascii() || tipo.is_empty() || tipo.len() > 255 {
        bail!("tipo di canale non valido: {tipo:?}");
    }
    let mut v = segreto.to_vec();
    v.push(tipo.len() as u8);
    v.extend_from_slice(tipo.as_bytes());
    Ok(v)
}

/// La riga di pronto del servizio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pronto {
    pub protocollo: u32,
    /// Nome del socket astratto: `phonestra_` e 32 cifre esadecimali.
    pub socket: String,
    pub pid: Option<u32>,
}

/// `phonestra-servizio pronto protocollo=1 socket=phonestra_<32 hex> pid=<pid>`;
/// `None` per qualsiasi altra riga o per un nome di socket inatteso (finisce in
/// un servizio ADB: niente caratteri strani).
pub fn leggi_pronto(riga: &str) -> Option<Pronto> {
    let resto = riga.trim().strip_prefix(PRONTO)?;
    let (mut protocollo, mut socket, mut pid) = (None, None, None);
    for voce in resto.split_whitespace() {
        match voce.split_once('=')? {
            ("protocollo", v) => protocollo = v.parse().ok(),
            ("socket", v) => socket = Some(v.to_string()),
            ("pid", v) => pid = v.parse().ok(),
            _ => {}
        }
    }
    let socket = socket?;
    let cifre = socket.strip_prefix("phonestra_")?;
    if cifre.len() != 32 || !cifre.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
        return None;
    }
    Some(Pronto { protocollo: protocollo?, socket, pid })
}

/// Il `CIAO` del servizio: righe `chiave=valore` (versione del protocollo,
/// Android, modello, tempi, e `autotest.<nome>=ok …|manca …`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ciao {
    pub voci: Vec<(String, String)>,
}

impl Ciao {
    pub fn leggi(dati: &[u8]) -> Self {
        let testo = String::from_utf8_lossy(dati);
        let voci = testo
            .lines()
            .filter_map(|r| r.split_once('='))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect();
        Self { voci }
    }

    pub fn valore(&self, chiave: &str) -> Option<&str> {
        self.voci.iter().find(|(k, _)| k == chiave).map(|(_, v)| v.as_str())
    }

    pub fn protocollo(&self) -> Option<u32> {
        self.valore("protocollo")?.parse().ok()
    }

    /// Le voci dell'autotest, nell'ordine: (nome, esito).
    pub fn autotest(&self) -> impl Iterator<Item = (&str, &str)> {
        self.voci.iter().filter_map(|(k, v)| Some((k.strip_prefix("autotest.")?, v.as_str())))
    }

    /// Le funzioni che il telefono non ha (esito diverso da «ok …»).
    pub fn mancanti(&self) -> Vec<&str> {
        self.autotest().filter(|(_, v)| *v != "ok" && !v.starts_with("ok ")).map(|(k, _)| k).collect()
    }
}

/// Stato del battito visto dal PC: quando mandare il prossimo, quanto silenzio
/// c'è stato dall'altra parte.
#[derive(Debug, Clone)]
pub struct Battito {
    ultimo_ricevuto: Instant,
    ultimo_mandato: Option<Instant>,
    /// Messaggi arrivati dal servizio (battiti e altri: ogni messaggio vale come segno di vita).
    pub ricevuti: u64,
    pub mandati: u64,
    /// Il silenzio più lungo tra due messaggi del servizio.
    pub pausa_massima: Duration,
}

impl Battito {
    pub fn new(adesso: Instant) -> Self {
        Self { ultimo_ricevuto: adesso, ultimo_mandato: None, ricevuti: 0, mandati: 0, pausa_massima: Duration::ZERO }
    }

    pub fn ricevuto(&mut self, adesso: Instant) {
        self.pausa_massima = self.pausa_massima.max(adesso.saturating_duration_since(self.ultimo_ricevuto));
        self.ultimo_ricevuto = adesso;
        self.ricevuti += 1;
    }

    pub fn da_mandare(&self, adesso: Instant) -> bool {
        self.ultimo_mandato.is_none_or(|u| adesso.saturating_duration_since(u) >= BATTITO_OGNI)
    }

    pub fn mandato(&mut self, adesso: Instant) {
        self.ultimo_mandato = Some(adesso);
        self.mandati += 1;
    }

    pub fn silenzio(&self, adesso: Instant) -> Duration {
        adesso.saturating_duration_since(self.ultimo_ricevuto)
    }

    /// Il servizio tace da troppo: per il PC il telefono è perso.
    pub fn perso(&self, adesso: Instant) -> bool {
        self.silenzio(adesso) >= LIMITE_SILENZIO
    }
}

/// Stato del processo del servizio, dal canale `shell,v2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Processo {
    Vivo,
    /// Uscito con questo codice (vedi [`descrivi_uscita`]).
    Uscito(u8),
    /// Canale chiuso senza codice d'uscita.
    Chiuso,
}

/// Significato dei codici d'uscita del servizio (`Servizio.USCITA_*`).
pub fn descrivi_uscita(codice: u8) -> String {
    match codice {
        0 => "fine ordinata".into(),
        1 => "errore".into(),
        3 => "nessun messaggio dal PC per 5 s".into(),
        4 => "canale comandi chiuso".into(),
        5 => "nessun PC entro 10 s".into(),
        c if c > 128 => format!("ucciso dal segnale {}", c - 128),
        c => format!("codice {c}"),
    }
}

/// Il servizio avviato sul telefono, col canale comandi aperto e il battito attivo.
pub struct Componente {
    adb: Adb,
    socket: String,
    segreto: [u8; 16],
    /// Il `CIAO` ricevuto all'apertura (telefono e autotest).
    pub ciao: Ciao,
    /// Pid del servizio sul telefono (dalla riga di pronto).
    pub pid: Option<u32>,
    uscita: mpsc::UnboundedSender<Messaggio>,
    eventi: mpsc::UnboundedReceiver<Messaggio>,
    in_attesa: VecDeque<Messaggio>,
    battito: Arc<Mutex<Battito>>,
    battito_attivo: Arc<AtomicBool>,
    processo: watch::Receiver<Processo>,
    prossimo_id: u16,
    chiusori: Vec<Chiusore>,
    compiti: Vec<JoinHandle<()>>,
}

impl Componente {
    /// Copia il jar, avvia il servizio, apre il canale comandi e legge il `CIAO`.
    pub async fn avvia(adb: &Adb) -> Result<Self> {
        let segreto: [u8; 16] = rand::random();
        let jar = format!("/data/local/tmp/{NOME_SERVIZIO}-{:08x}.jar", rand::random::<u32>());
        sync::invia(adb, crate::app::AIUTO, &jar, 0o644).await.context(t!("copia del componente sul telefono"))?;
        // `exec`: il servizio prende il posto di sh, così il SIGHUP di adbd arriva a lui.
        let comando =
            format!("export CLASSPATH={jar}; exec app_process / --nice-name={NOME_SERVIZIO} phonestra.Aiuto servizio");
        let avvio = async {
            let mut shell = ShellV2::avvia(adb, &comando).await?;
            shell.scrivi(format!("{}\n", esadecimale(&segreto)).as_bytes()).await?;
            let pronto = tokio::time::timeout(ATTESA_PRONTO, aspetta_pronto(&mut shell))
                .await
                .map_err(|_| anyhow!(t!("il servizio non è partito entro {} s", ATTESA_PRONTO.as_secs())))??;
            anyhow::Ok((shell, pronto))
        };
        let (shell, pronto) = match avvio.await {
            Ok(v) => v,
            Err(e) => {
                // Il servizio cancella il jar appena partito: se non è partito resta.
                let _ = adb.esegui(&format!("rm -f {jar}")).await;
                return Err(e);
            }
        };
        if pronto.protocollo != PROTOCOLLO {
            let _ = shell.chiusore().chiudi().await;
            bail!(t!("protocollo del servizio {} invece di {}", pronto.protocollo, PROTOCOLLO));
        }
        let (tx_processo, processo) = watch::channel(Processo::Vivo);
        let mut chiusori = vec![shell.chiusore()];
        let compito_shell = tokio::spawn(segui_shell(shell, tx_processo));

        let mut comandi = match apri_socket(adb, &pronto.socket, &segreto, "comandi").await {
            Ok(c) => c,
            Err(e) => {
                compito_shell.abort();
                for c in chiusori {
                    let _ = c.chiudi().await;
                }
                return Err(e);
            }
        };
        chiusori.push(comandi.chiusore());
        let mut decodificatore = Decodificatore::default();
        let primo = tokio::time::timeout(ATTESA_CIAO, async {
            loop {
                if let Some(m) = decodificatore.prossimo()? {
                    return anyhow::Ok(m);
                }
                let blocco = comandi.leggi().await.ok_or_else(|| anyhow!(t!("il servizio ha chiuso il canale comandi")))?;
                decodificatore.aggiungi(&blocco);
            }
        })
        .await
        .map_err(|_| anyhow!(t!("nessun CIAO dal servizio entro {} s", ATTESA_CIAO.as_secs())))
        .and_then(|r| r)
        .and_then(|m| {
            if m.tipo != tipo::CIAO {
                bail!(t!("primo messaggio del servizio di tipo {} invece del CIAO", format!("{:#04x}", m.tipo)));
            }
            let ciao = Ciao::leggi(&m.dati);
            if ciao.protocollo() != Some(PROTOCOLLO) {
                bail!(t!("CIAO con protocollo {} invece di {}", format!("{:?}", ciao.valore("protocollo")), PROTOCOLLO));
            }
            Ok(ciao)
        });
        let ciao = match primo {
            Ok(c) => c,
            Err(e) => {
                // Chiudere il canale d'avvio manda SIGHUP al servizio: il custode ripulisce.
                compito_shell.abort();
                for c in chiusori {
                    let _ = c.chiudi().await;
                }
                return Err(e);
            }
        };

        let battito = Arc::new(Mutex::new(Battito::new(Instant::now())));
        let battito_attivo = Arc::new(AtomicBool::new(true));
        let (uscita, rx_uscita) = mpsc::unbounded_channel();
        let (tx_eventi, eventi) = mpsc::unbounded_channel();
        let compito_comandi = tokio::spawn(gira_comandi(
            comandi,
            decodificatore,
            rx_uscita,
            tx_eventi,
            battito.clone(),
            battito_attivo.clone(),
        ));
        Ok(Self {
            adb: adb.clone(),
            socket: pronto.socket,
            segreto,
            ciao,
            pid: pronto.pid,
            uscita,
            eventi,
            in_attesa: VecDeque::new(),
            battito,
            battito_attivo,
            processo,
            prossimo_id: 1,
            chiusori,
            compiti: vec![compito_shell, compito_comandi],
        })
    }

    /// Apre un altro canale del servizio (`audio`, `video:<id>`…), già
    /// presentato col segreto: i pezzi futuri ci leggono e scrivono il loro formato.
    pub async fn apri_canale(&self, tipo: &str) -> Result<Canale> {
        apri_socket(&self.adb, &self.socket, &self.segreto, tipo).await
    }

    /// Chi può aprire altri canali del servizio senza possedere il `Componente`.
    pub fn apritore(&self) -> Apritore {
        Apritore { adb: self.adb.clone(), socket: self.socket.clone(), segreto: self.segreto }
    }

    /// Manda un messaggio sul canale comandi; restituisce il suo id.
    pub fn manda(&mut self, tipo: u8, dati: impl Into<Vec<u8>>) -> Result<u16> {
        let id = self.prossimo_id;
        self.prossimo_id = self.prossimo_id.checked_add(1).unwrap_or(1);
        let m = Messaggio { tipo, bandiere: 0, id, dati: dati.into() };
        self.uscita.send(m).map_err(|_| anyhow!("canale comandi chiuso"))?;
        Ok(id)
    }

    /// Chi può mandare messaggi sul canale comandi senza possedere il
    /// `Componente` (per esempio l'input di ogni finestra).
    pub fn mittente(&self) -> Mittente {
        Mittente(self.uscita.clone())
    }

    /// Manda una domanda e aspetta la risposta (stesso id); i messaggi
    /// spontanei arrivati intanto restano per [`Componente::ricevi`].
    pub async fn richiesta(&mut self, tipo: u8, dati: impl Into<Vec<u8>>) -> Result<Messaggio> {
        let id = self.manda(tipo, dati)?;
        let limite = tokio::time::Instant::now() + ATTESA_RISPOSTA;
        loop {
            let m = tokio::time::timeout_at(limite, self.eventi.recv())
                .await
                .map_err(|_| anyhow!("nessuna risposta dal servizio entro {} s", ATTESA_RISPOSTA.as_secs()))?
                .ok_or_else(|| anyhow!("canale comandi chiuso"))?;
            if m.risposta() && m.id == id {
                if m.tipo == tipo::ERRORE {
                    bail!("il servizio risponde: {}", m.testo());
                }
                return Ok(m);
            }
            self.in_attesa.push_back(m);
        }
    }

    /// Il prossimo messaggio spontaneo del servizio (non i battiti); `None`
    /// quando il canale comandi è chiuso.
    pub async fn ricevi(&mut self) -> Option<Messaggio> {
        if let Some(m) = self.in_attesa.pop_front() {
            return Some(m);
        }
        self.eventi.recv().await
    }

    /// Prova del custode: il servizio crea [`FILE_PROVA_CUSTODE`] e affida al
    /// custode il compito di toglierlo. Restituisce il percorso.
    pub async fn prova_custode(&mut self) -> Result<String> {
        Ok(self.richiesta(tipo::PROVA_CUSTODE, Vec::new()).await?.testo())
    }

    /// Solo per le prove: smette di mandare il battito senza chiudere niente,
    /// come un PC sparito.
    pub fn sospendi_battito(&self) {
        self.battito_attivo.store(false, Ordering::Relaxed);
    }

    pub fn battito(&self) -> Battito {
        self.battito.lock().unwrap().clone()
    }

    pub fn processo(&self) -> Processo {
        *self.processo.borrow()
    }

    /// Aspetta al massimo `limite` che il processo del servizio finisca.
    pub async fn attendi_uscita(&mut self, limite: Duration) -> Processo {
        let _ = tokio::time::timeout(limite, self.processo.wait_for(|p| *p != Processo::Vivo)).await;
        self.processo()
    }

    /// Chiusura in ordine: `FINE`, attesa dell'uscita del servizio (il custode
    /// intanto ripristina), chiusura dei canali.
    pub async fn chiudi(mut self) -> Result<Processo> {
        if self.processo() == Processo::Vivo
            && let Err(e) = self.richiesta(tipo::FINE, Vec::new()).await
        {
            eprintln!("[servizio] FINE senza risposta: {e:#}");
        }
        let esito = self.attendi_uscita(Duration::from_secs(5)).await;
        for c in self.chiusori.drain(..) {
            let _ = c.chiudi().await;
        }
        Ok(esito)
    }
}

/// Messaggi spontanei (id 0, nessuna risposta) sul canale comandi, nello
/// stesso ordine di quelli del [`Componente`]. Si può clonare.
#[derive(Clone)]
pub struct Mittente(mpsc::UnboundedSender<Messaggio>);

impl Mittente {
    /// Errore se il canale comandi è chiuso (servizio finito o telefono perso).
    pub fn manda(&self, tipo: u8, dati: impl Into<Vec<u8>>) -> Result<()> {
        self.0.send(Messaggio::new(tipo, dati)).map_err(|_| anyhow!("canale comandi chiuso"))
    }

    /// Per le prove automatiche: i messaggi finiscono in `tx` invece che al telefono.
    #[cfg(test)]
    pub(crate) fn per_prova(tx: mpsc::UnboundedSender<Messaggio>) -> Self {
        Self(tx)
    }
}

/// Apre i canali del servizio (`audio`, `video:<id>`…) già presentati col
/// segreto. Si può clonare: ogni apertura è indipendente dal canale comandi.
#[derive(Clone)]
pub struct Apritore {
    adb: Adb,
    socket: String,
    segreto: [u8; 16],
}

impl Apritore {
    pub async fn apri(&self, tipo: &str) -> Result<Canale> {
        apri_socket(&self.adb, &self.socket, &self.segreto, tipo).await
    }
}

impl Drop for Componente {
    /// Senza [`Componente::chiudi`]: si fermano i compiti (e il battito), il
    /// servizio si chiude da solo dopo 5 s.
    fn drop(&mut self) {
        for c in &self.compiti {
            c.abort();
        }
    }
}

async fn apri_socket(adb: &Adb, socket: &str, segreto: &[u8; 16], tipo: &str) -> Result<Canale> {
    let mut c = adb
        .apri(&format!("localabstract:{socket}"))
        .await
        .with_context(|| t!("apertura del canale «{}» del servizio", tipo))?;
    c.scrivi(&preambolo(segreto, tipo)?).await?;
    Ok(c)
}

/// Legge l'uscita del servizio fino alla riga di pronto.
async fn aspetta_pronto(shell: &mut ShellV2) -> Result<Pronto> {
    let (mut uscita, mut errori) = (Vec::new(), Vec::new());
    while let Some(evento) = shell.leggi().await {
        match evento {
            Evento::Uscita(d) => {
                uscita.extend_from_slice(&d);
                while let Some(fine) = uscita.iter().position(|&b| b == b'\n') {
                    let riga: Vec<u8> = uscita.drain(..=fine).collect();
                    let riga = String::from_utf8_lossy(&riga).trim().to_string();
                    if let Some(p) = leggi_pronto(&riga) {
                        return Ok(p);
                    }
                    if let Some(motivo) = riga.strip_prefix(ERRORE_AVVIO) {
                        bail!(t!("il servizio non è partito: {}", motivo.trim()));
                    }
                    if !riga.is_empty() {
                        eprintln!("[servizio] {riga}");
                    }
                }
            }
            Evento::Errori(d) => errori.extend_from_slice(&d),
            Evento::Codice(c) => bail!(t!(
                "il servizio è uscito prima di essere pronto ({}): {}",
                descrivi_uscita(c),
                String::from_utf8_lossy(&errori).trim()
            )),
        }
    }
    bail!(t!("canale del servizio chiuso prima della riga di pronto: {}", String::from_utf8_lossy(&errori).trim()))
}

/// Dopo il pronto: i messaggi d'errore del servizio vanno nel log, il codice
/// d'uscita nello stato del processo.
async fn segui_shell(mut shell: ShellV2, processo: watch::Sender<Processo>) {
    let mut resto = Vec::new();
    while let Some(evento) = shell.leggi().await {
        match evento {
            Evento::Uscita(d) | Evento::Errori(d) => {
                resto.extend_from_slice(&d);
                while let Some(fine) = resto.iter().position(|&b| b == b'\n') {
                    let riga: Vec<u8> = resto.drain(..=fine).collect();
                    eprintln!("[servizio] {}", String::from_utf8_lossy(&riga).trim_end());
                }
            }
            Evento::Codice(c) => {
                processo.send_replace(Processo::Uscito(c));
            }
        }
    }
    processo.send_if_modified(|p| {
        let vivo = *p == Processo::Vivo;
        if vivo {
            *p = Processo::Chiuso;
        }
        vivo
    });
}

/// Il canale comandi: messaggi in arrivo agli `eventi`, quelli da mandare
/// dalla coda, battito ogni secondo. Finisce quando il canale si chiude o il
/// servizio tace per 5 s.
async fn gira_comandi(
    mut canale: Canale,
    mut decodificatore: Decodificatore,
    mut da_mandare: mpsc::UnboundedReceiver<Messaggio>,
    eventi: mpsc::UnboundedSender<Messaggio>,
    battito: Arc<Mutex<Battito>>,
    attivo: Arc<AtomicBool>,
) {
    let mut orologio = tokio::time::interval(Duration::from_millis(200));
    orologio.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            blocco = canale.leggi() => {
                let Some(blocco) = blocco else {
                    eprintln!("[servizio] canale comandi chiuso dal telefono");
                    return;
                };
                decodificatore.aggiungi(&blocco);
                loop {
                    match decodificatore.prossimo() {
                        Ok(Some(m)) => {
                            battito.lock().unwrap().ricevuto(Instant::now());
                            if m.tipo != tipo::BATTITO {
                                let _ = eventi.send(m);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            eprintln!("[servizio] {e:#}");
                            return;
                        }
                    }
                }
            }
            m = da_mandare.recv() => {
                let Some(m) = m else { return };
                if let Err(e) = canale.scrivi(&m.in_byte()).await {
                    eprintln!("[servizio] scrittura sul canale comandi: {e:#}");
                    return;
                }
            }
            _ = orologio.tick() => {
                let adesso = Instant::now();
                let (manda, perso) = {
                    let b = battito.lock().unwrap();
                    (attivo.load(Ordering::Relaxed) && b.da_mandare(adesso), b.perso(adesso))
                };
                if perso {
                    eprintln!("[servizio] nessun messaggio dal telefono da {} s: collegamento perso", LIMITE_SILENZIO.as_secs());
                    return;
                }
                if manda {
                    if canale.scrivi(&Messaggio::new(tipo::BATTITO, Vec::new()).in_byte()).await.is_err() {
                        return;
                    }
                    battito.lock().unwrap().mandato(adesso);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Il servizio condiviso da audio, video e input di un collegamento.

/// Tempo massimo per la risposta a una domanda fatta con [`Condiviso`].
const ATTESA_RISPOSTA_CONDIVISO: Duration = Duration::from_secs(5);
/// Eventi di sessioni non ancora note (la risposta di apertura può arrivare
/// dopo i primi eventi): si tengono per questo tempo, poi si scartano.
const ATTESA_ORFANI: Duration = Duration::from_secs(2);
const MASSIMO_ORFANI: usize = 64;

/// Tipi dei messaggi spontanei che appartengono a una sessione: il contenuto è
/// a righe `chiave=valore` con `id=<sessione>` (oggi solo gli eventi del video).
const EVENTI_DI_SESSIONE: &[u8] = &[tipo::VIDEO_EVENTO];

/// Il valore di `chiave` in un contenuto a righe `chiave=valore`.
pub fn valore_coppia<'a>(dati: &'a [u8], chiave: &str) -> Option<&'a str> {
    std::str::from_utf8(dati)
        .ok()?
        .lines()
        .filter_map(|r| r.split_once('='))
        .find(|(k, _)| k.trim() == chiave)
        .map(|(_, v)| v.trim())
}

/// La risposta a una domanda che apre una sessione, con gli eventi della sessione.
pub type Apertura = (Messaggio, mpsc::UnboundedReceiver<Messaggio>);

/// Chi aspetta una risposta.
enum Attesa {
    Semplice(oneshot::Sender<Result<Messaggio>>),
    /// La risposta porta `id=<sessione>`: da lì gli eventi della sessione
    /// vanno al ricevitore consegnato con la risposta.
    Sessione(oneshot::Sender<Result<Apertura>>),
}

impl Attesa {
    fn fallisci(self, errore: anyhow::Error) {
        match self {
            Attesa::Semplice(r) => {
                let _ = r.send(Err(errore));
            }
            Attesa::Sessione(r) => {
                let _ = r.send(Err(errore));
            }
        }
    }
}

/// Lo smistamento dei messaggi del canale comandi, senza rete (per le prove):
/// risposte per id, eventi per sessione, messaggi spontanei per tipo.
#[derive(Default)]
struct Smistamento {
    attese: HashMap<u16, (Instant, Attesa)>,
    sessioni: HashMap<u32, mpsc::UnboundedSender<Messaggio>>,
    iscritti: HashMap<u8, Vec<mpsc::UnboundedSender<Messaggio>>>,
    orfani: VecDeque<(Instant, u32, Messaggio)>,
}

impl Smistamento {
    fn attendi(&mut self, id: u16, adesso: Instant, attesa: Attesa) {
        self.attese.insert(id, (adesso + ATTESA_RISPOSTA_CONDIVISO, attesa));
    }

    fn iscrivi(&mut self, tipo: u8, tx: mpsc::UnboundedSender<Messaggio>) {
        self.iscritti.entry(tipo).or_default().push(tx);
    }

    fn dimentica(&mut self, sessione: u32) {
        self.sessioni.remove(&sessione);
    }

    /// Un messaggio arrivato dal servizio (i battiti non arrivano qui).
    fn ricevuto(&mut self, m: Messaggio, adesso: Instant) {
        if m.risposta() {
            self.risposta(m);
        } else if EVENTI_DI_SESSIONE.contains(&m.tipo) {
            self.evento_di_sessione(m, adesso);
        } else {
            match self.iscritti.get_mut(&m.tipo) {
                Some(elenco) => {
                    elenco.retain(|tx| !tx.is_closed());
                    for tx in elenco.iter() {
                        let _ = tx.send(m.clone());
                    }
                }
                None => {
                    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
                        eprintln!("[servizio] messaggio di tipo {:#04x} senza destinatario", m.tipo);
                    }
                }
            }
        }
    }

    fn risposta(&mut self, m: Messaggio) {
        let Some((_, attesa)) = self.attese.remove(&m.id) else {
            // Risposta a un messaggio mandato senza aspettarla.
            if m.tipo == tipo::ERRORE {
                eprintln!("[servizio] il telefono risponde: {}", m.testo());
            }
            return;
        };
        if m.tipo == tipo::ERRORE {
            attesa.fallisci(anyhow!(t!("il telefono risponde: {}", m.testo())));
            return;
        }
        match attesa {
            Attesa::Semplice(r) => {
                let _ = r.send(Ok(m));
            }
            Attesa::Sessione(r) => {
                let Some(id) = valore_coppia(&m.dati, "id").and_then(|v| v.parse::<u32>().ok()) else {
                    let _ = r.send(Err(anyhow!("risposta di apertura senza «id»")));
                    return;
                };
                let (tx, rx) = mpsc::unbounded_channel();
                // Gli eventi arrivati prima della risposta.
                for (_, _, e) in self.orfani.iter().filter(|(_, s, _)| *s == id) {
                    let _ = tx.send(e.clone());
                }
                self.orfani.retain(|(_, s, _)| *s != id);
                self.sessioni.insert(id, tx);
                let _ = r.send(Ok((m, rx)));
            }
        }
    }

    fn evento_di_sessione(&mut self, m: Messaggio, adesso: Instant) {
        let Some(id) = valore_coppia(&m.dati, "id").and_then(|v| v.parse::<u32>().ok()) else {
            eprintln!("[servizio] evento senza sessione: {}", m.testo().replace('\n', " "));
            return;
        };
        let fine = valore_coppia(&m.dati, "evento") == Some("fine");
        match self.sessioni.get(&id) {
            Some(tx) => {
                let _ = tx.send(m);
            }
            None => {
                self.orfani.retain(|(t, _, _)| adesso.saturating_duration_since(*t) < ATTESA_ORFANI);
                if self.orfani.len() >= MASSIMO_ORFANI {
                    self.orfani.pop_front();
                }
                self.orfani.push_back((adesso, id, m));
            }
        }
        if fine {
            self.sessioni.remove(&id);
        }
    }

    /// Le domande senza risposta entro il tempo falliscono.
    fn scadute(&mut self, adesso: Instant) {
        let scadute: Vec<u16> = self.attese.iter().filter(|(_, (t, _))| *t <= adesso).map(|(id, _)| *id).collect();
        for id in scadute {
            if let Some((_, a)) = self.attese.remove(&id) {
                a.fallisci(anyhow!(t!("nessuna risposta dal telefono entro {} s", ATTESA_RISPOSTA_CONDIVISO.as_secs())));
            }
        }
    }
}

enum Richiesta {
    /// Messaggio sul canale comandi; con `risposta` si aspetta la risposta.
    Domanda { tipo: u8, dati: Vec<u8>, risposta: Option<oneshot::Sender<Result<Messaggio>>> },
    /// Domanda che apre una sessione (`VIDEO_APRI`).
    ApriSessione { tipo: u8, dati: Vec<u8>, risposta: oneshot::Sender<Result<Apertura>> },
    /// La sessione è chiusa: niente più eventi.
    Dimentica(u32),
    Iscrivi { tipo: u8, tx: mpsc::UnboundedSender<Messaggio> },
    Chiudi(oneshot::Sender<Result<Processo>>),
}

/// Il servizio di un collegamento, condiviso da audio, video e input (e dopo
/// appunti): si clona. Un compito possiede il [`Componente`] e smista i
/// messaggi; quando l'ultima copia sparisce il servizio si chiude.
#[derive(Clone)]
pub struct Condiviso {
    richieste: mpsc::UnboundedSender<Richiesta>,
    mittente: Mittente,
    apritore: Apritore,
    attivo: watch::Receiver<bool>,
    /// Il `CIAO` del servizio (telefono e autotest).
    pub ciao: Arc<Ciao>,
}

impl Condiviso {
    /// Prende il componente già avviato e comincia a smistarne i messaggi.
    pub fn avvia(componente: Componente) -> Self {
        let (richieste, rx) = mpsc::unbounded_channel();
        let (tx_attivo, attivo) = watch::channel(true);
        let condiviso = Self {
            richieste,
            mittente: componente.mittente(),
            apritore: componente.apritore(),
            attivo,
            ciao: Arc::new(componente.ciao.clone()),
        };
        tokio::spawn(smista(componente, rx, tx_attivo));
        condiviso
    }

    fn invia(&self, r: Richiesta) -> Result<()> {
        self.richieste.send(r).map_err(|_| anyhow!(t!("componente del telefono chiuso")))
    }

    /// Nome del telefono (dal `CIAO`).
    pub fn nome_dispositivo(&self) -> String {
        self.ciao.valore("modello").unwrap_or("telefono").to_string()
    }

    /// Il canale comandi è ancora aperto (servizio vivo, telefono raggiungibile).
    pub fn vivo(&self) -> bool {
        *self.attivo.borrow()
    }

    /// Aspetta che il canale comandi si chiuda (servizio finito o telefono perso).
    pub async fn finito(&self) {
        let mut attivo = self.attivo.clone();
        let _ = attivo.wait_for(|a| !*a).await;
    }

    /// Per i messaggi senza risposta attesa (tocchi, tasti): vanno diretti in
    /// coda al canale comandi.
    pub fn mittente(&self) -> Mittente {
        self.mittente.clone()
    }

    /// Apre un altro canale del servizio (`audio`, `video:<id>`…).
    pub fn apritore(&self) -> &Apritore {
        &self.apritore
    }

    /// Manda senza aspettare la risposta: un errore del telefono finisce nel log.
    pub fn manda(&self, tipo: u8, dati: Vec<u8>) -> Result<()> {
        self.invia(Richiesta::Domanda { tipo, dati, risposta: None })
    }

    /// Manda e aspetta la risposta (al massimo 5 s).
    pub async fn domanda(&self, tipo: u8, dati: Vec<u8>) -> Result<Messaggio> {
        let (tx, rx) = oneshot::channel();
        self.invia(Richiesta::Domanda { tipo, dati, risposta: Some(tx) })?;
        rx.await.map_err(|_| anyhow!(t!("componente del telefono chiuso")))?
    }

    /// Domanda che apre una sessione: la risposta porta `id=<sessione>`, e gli
    /// eventi di quella sessione arrivano al ricevitore restituito.
    pub async fn apri_sessione(&self, tipo: u8, dati: Vec<u8>) -> Result<Apertura> {
        let (tx, rx) = oneshot::channel();
        self.invia(Richiesta::ApriSessione { tipo, dati, risposta: tx })?;
        rx.await.map_err(|_| anyhow!(t!("componente del telefono chiuso")))?
    }

    /// La sessione è chiusa: i suoi eventi non servono più.
    pub fn dimentica(&self, sessione: u32) {
        let _ = self.invia(Richiesta::Dimentica(sessione));
    }

    /// I messaggi spontanei di tipo `tipo` (per esempio le copie fatte sul telefono).
    pub fn iscrivi(&self, tipo: u8) -> Result<mpsc::UnboundedReceiver<Messaggio>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.invia(Richiesta::Iscrivi { tipo, tx })?;
        Ok(rx)
    }

    /// Chiude il servizio in ordine (`FINE`: il custode ripristina) per tutti
    /// quelli che lo usano.
    pub async fn chiudi(&self) -> Result<Processo> {
        let (tx, rx) = oneshot::channel();
        self.invia(Richiesta::Chiudi(tx))?;
        rx.await.map_err(|_| anyhow!("componente del telefono già chiuso"))?
    }
}

/// Il compito che possiede il [`Componente`]: domande in uscita, risposte ed
/// eventi in arrivo.
async fn smista(mut c: Componente, mut richieste: mpsc::UnboundedReceiver<Richiesta>, attivo: watch::Sender<bool>) {
    let mut s = Smistamento::default();
    let mut orologio = tokio::time::interval(Duration::from_millis(500));
    orologio.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            r = richieste.recv() => {
                let Some(r) = r else {
                    // Nessuno usa più il servizio: si chiude.
                    attivo.send_replace(false);
                    let _ = c.chiudi().await;
                    return;
                };
                match r {
                    Richiesta::Domanda { tipo, dati, risposta } => match (c.manda(tipo, dati), risposta) {
                        (Ok(id), Some(r)) => s.attendi(id, Instant::now(), Attesa::Semplice(r)),
                        (Err(e), Some(r)) => {
                            let _ = r.send(Err(e));
                        }
                        _ => {}
                    },
                    Richiesta::ApriSessione { tipo, dati, risposta } => match c.manda(tipo, dati) {
                        Ok(id) => s.attendi(id, Instant::now(), Attesa::Sessione(risposta)),
                        Err(e) => {
                            let _ = risposta.send(Err(e));
                        }
                    },
                    Richiesta::Dimentica(id) => s.dimentica(id),
                    Richiesta::Iscrivi { tipo, tx } => s.iscrivi(tipo, tx),
                    Richiesta::Chiudi(r) => {
                        attivo.send_replace(false);
                        let _ = r.send(c.chiudi().await);
                        return;
                    }
                }
            }
            m = c.ricevi() => {
                let Some(m) = m else { break };
                s.ricevuto(m, Instant::now());
            }
            _ = orologio.tick() => s.scadute(Instant::now()),
        }
    }
    // Canale comandi chiuso (servizio finito o telefono perso): le attese
    // falliscono, gli eventi finiscono; resta solo da raccogliere il codice d'uscita.
    attivo.send_replace(false);
    drop(s);
    while let Some(r) = richieste.recv().await {
        let chiuso = || anyhow!(t!("componente del telefono chiuso"));
        match r {
            Richiesta::Chiudi(r) => {
                let _ = r.send(c.chiudi().await);
                return;
            }
            Richiesta::Domanda { risposta: Some(r), .. } => {
                let _ = r.send(Err(chiuso()));
            }
            Richiesta::ApriSessione { risposta, .. } => {
                let _ = risposta.send(Err(chiuso()));
            }
            _ => {}
        }
    }
}

fn esadecimale(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Quello che del componente resta sul telefono (per le prove di chiusura).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Residui {
    /// Righe di `ps` del servizio o del custode ancora vivi.
    pub processi: Vec<String>,
    /// File del componente in `/data/local/tmp` (jar del servizio, file di prova).
    pub file: Vec<String>,
    /// Altri file `phonestra-*` (aiutante, prove video, copie lasciate da versioni
    /// vecchie): solo per informazione.
    pub altri_file: Vec<String>,
}

impl Residui {
    pub fn pulito(&self) -> bool {
        self.processi.is_empty() && self.file.is_empty()
    }
}

/// Processi e file del componente rimasti sul telefono.
pub async fn residui(adb: &Adb) -> Result<Residui> {
    // Le parentesi quadre evitano che grep trovi sé stesso (e la shell di questo comando).
    let ps = adb.esegui("ps -A -o PID,RSS,ARGS | grep -e '[p]honestra-servizio' -e '[p]honestra-custode'").await?;
    let ls = adb.esegui("ls /data/local/tmp").await?;
    Ok(leggi_residui(&ps, &ls))
}

fn leggi_residui(ps: &str, ls: &str) -> Residui {
    let processi = ps.lines().map(str::trim).filter(|r| !r.is_empty()).map(String::from).collect();
    let prova = FILE_PROVA_CUSTODE.rsplit('/').next().unwrap_or_default();
    let (mut file, mut altri_file) = (Vec::new(), Vec::new());
    for nome in ls.split_whitespace().filter(|n| n.starts_with("phonestra-")) {
        if nome.starts_with(&format!("{NOME_SERVIZIO}-")) || nome == prova {
            file.push(nome.to_string());
        } else {
            altri_file.push(nome.to_string());
        }
    }
    Residui { processi, file, altri_file }
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn messaggi_andata_e_ritorno_a_pezzi() {
        let mut a = Messaggio::new(tipo::CIAO, b"protocollo=1\n".to_vec());
        a.id = 0x1234;
        let b = Messaggio { tipo: tipo::FINE, bandiere: RISPOSTA, id: 7, dati: Vec::new() };
        let mut flusso = a.in_byte();
        assert_eq!(&flusso[..8], &[0x01, 0x00, 0x12, 0x34, 0, 0, 0, 13]);
        flusso.extend(b.in_byte());
        let mut d = Decodificatore::default();
        let mut letti = Vec::new();
        for pezzo in flusso.chunks(5) {
            d.aggiungi(pezzo);
            while let Some(m) = d.prossimo().unwrap() {
                letti.push(m);
            }
        }
        assert_eq!(letti, vec![a, b.clone()]);
        assert!(letti[1].risposta());
    }

    #[test]
    fn messaggio_troppo_grande() {
        let mut d = Decodificatore::default();
        d.aggiungi(&[0x02, 0, 0, 0, 0x7f, 0xff, 0xff, 0xff]);
        assert!(d.prossimo().is_err());
    }

    #[test]
    fn preambolo_del_canale() {
        let segreto = [0xab; 16];
        let p = preambolo(&segreto, "comandi").unwrap();
        assert_eq!(&p[..16], &segreto);
        assert_eq!(p[16], 7);
        assert_eq!(&p[17..], b"comandi");
        assert!(preambolo(&segreto, "").is_err());
        assert!(preambolo(&segreto, "vìdeo").is_err());
    }

    #[test]
    fn riga_di_pronto() {
        let nome = "phonestra_0123456789abcdef0123456789abcdef";
        let p = leggi_pronto(&format!("phonestra-servizio pronto protocollo=1 socket={nome} pid=4321\n")).unwrap();
        assert_eq!(p, Pronto { protocollo: 1, socket: nome.into(), pid: Some(4321) });
        // Righe diverse, nomi di socket che finirebbero male in un servizio ADB.
        assert_eq!(leggi_pronto("phonestra-servizio errore qualcosa"), None);
        assert_eq!(leggi_pronto("phonestra-servizio pronto protocollo=1 socket=phonestra_12;rm"), None);
        assert_eq!(leggi_pronto("phonestra-servizio pronto protocollo=1 socket=altro_0123456789abcdef0123456789abcdef"), None);
        assert_eq!(leggi_pronto(&format!("phonestra-servizio pronto socket={nome}")), None);
        assert_eq!(
            leggi_pronto("phonestra-servizio pronto protocollo=1 socket=phonestra_0123456789ABCDEF0123456789ABCDEF"),
            None
        );
    }

    #[test]
    fn ciao_e_autotest() {
        let testo = "protocollo=1\nandroid=16\nmodello=SM-X\nautotest.contesto=ok pacchetto=com.android.shell\n\
                     autotest.audio_policy=manca ClassNotFoundException: x\nautotest.custode=ok setsid\nrigasenzauguale\n";
        let c = Ciao::leggi(testo.as_bytes());
        assert_eq!(c.protocollo(), Some(1));
        assert_eq!(c.valore("android"), Some("16"));
        assert_eq!(c.autotest().count(), 3);
        assert_eq!(c.mancanti(), vec!["audio_policy"]);
    }

    #[test]
    fn battito_nel_tempo() {
        let t0 = Instant::now();
        let mut b = Battito::new(t0);
        assert!(b.da_mandare(t0));
        b.mandato(t0);
        assert!(!b.da_mandare(t0 + Duration::from_millis(999)));
        assert!(b.da_mandare(t0 + Duration::from_millis(1000)));
        b.ricevuto(t0 + Duration::from_millis(1200));
        b.ricevuto(t0 + Duration::from_millis(3700));
        assert_eq!(b.pausa_massima, Duration::from_millis(2500));
        assert_eq!(b.ricevuti, 2);
        assert!(!b.perso(t0 + Duration::from_millis(8699)));
        assert!(b.perso(t0 + Duration::from_millis(8700)));
    }

    #[test]
    fn residui_sul_telefono() {
        let ps = "  4321  51234 phonestra-servizio\n 4330 1800 sh -c trap … phonestra-custode /data/local/tmp/x.jar\n";
        let ls = "phonestra-servizio-0a1b2c3d.jar\nphonestra-prova-custode\nphonestra-server-1234.jar\naltro.txt\n";
        let r = leggi_residui(ps, ls);
        assert_eq!(r.processi.len(), 2);
        assert_eq!(r.file, vec!["phonestra-servizio-0a1b2c3d.jar", "phonestra-prova-custode"]);
        assert_eq!(r.altri_file, vec!["phonestra-server-1234.jar"]);
        assert!(!r.pulito());
        assert!(leggi_residui("", "altro\n").pulito());
    }

    #[test]
    fn codici_d_uscita() {
        assert_eq!(descrivi_uscita(3), "nessun messaggio dal PC per 5 s");
        assert_eq!(descrivi_uscita(137), "ucciso dal segnale 9");
    }

    fn risposta(tipo: u8, id: u16, testo: &str) -> Messaggio {
        Messaggio { tipo, bandiere: RISPOSTA, id, dati: testo.as_bytes().to_vec() }
    }

    fn evento(testo: &str) -> Messaggio {
        Messaggio::new(tipo::VIDEO_EVENTO, testo.as_bytes().to_vec())
    }

    #[test]
    fn smistamento_delle_risposte() {
        let t0 = Instant::now();
        let mut s = Smistamento::default();
        let (tx1, mut rx1) = oneshot::channel();
        let (tx2, mut rx2) = oneshot::channel();
        s.attendi(1, t0, Attesa::Semplice(tx1));
        s.attendi(2, t0, Attesa::Semplice(tx2));
        // Risposte fuori ordine, una risposta non attesa, un ERRORE.
        s.ricevuto(risposta(tipo::VIDEO_CHIAVE, 9, ""), t0);
        s.ricevuto(risposta(tipo::ERRORE, 2, "sessione sconosciuta"), t0);
        s.ricevuto(risposta(tipo::VIDEO_PANNELLO, 1, "schermi=1"), t0);
        assert_eq!(rx1.try_recv().unwrap().unwrap().testo(), "schermi=1");
        let errore = rx2.try_recv().unwrap().unwrap_err().to_string();
        assert!(errore.contains("sessione sconosciuta"), "{errore}");
        assert!(s.attese.is_empty());
        // Scadenza dopo 5 s.
        let (tx3, mut rx3) = oneshot::channel();
        s.attendi(3, t0, Attesa::Semplice(tx3));
        s.scadute(t0 + Duration::from_millis(4999));
        assert!(rx3.try_recv().is_err());
        s.scadute(t0 + ATTESA_RISPOSTA_CONDIVISO);
        assert!(rx3.try_recv().unwrap().is_err());
    }

    #[test]
    fn smistamento_degli_eventi_per_sessione() {
        let t0 = Instant::now();
        let mut s = Smistamento::default();
        // Un evento della sessione 4 arriva prima della risposta di apertura.
        s.ricevuto(evento("evento=protetta\nid=4\ndisplay=57\nprotetta=0\n"), t0);
        let (tx, mut rx) = oneshot::channel();
        s.attendi(7, t0, Attesa::Sessione(tx));
        s.ricevuto(risposta(tipo::VIDEO_APRI, 7, "id=4\ndisplay=57\n"), t0);
        let (m, mut eventi) = rx.try_recv().unwrap().unwrap();
        assert_eq!(valore_coppia(&m.dati, "display"), Some("57"));
        assert_eq!(valore_coppia(&eventi.try_recv().unwrap().dati, "evento"), Some("protetta"));
        // Altre sessioni non ricevono gli eventi della 4.
        let (tx, mut rx) = oneshot::channel();
        s.attendi(8, t0, Attesa::Sessione(tx));
        s.ricevuto(risposta(tipo::VIDEO_APRI, 8, "id=5\n"), t0);
        let (_, mut altri) = rx.try_recv().unwrap().unwrap();
        s.ricevuto(evento("evento=orientamento\nid=4\ndisplay=57\nverticale=1\n"), t0);
        s.ricevuto(evento("evento=fine\nid=4\nmotivo=x\n"), t0);
        assert_eq!(valore_coppia(&eventi.try_recv().unwrap().dati, "evento"), Some("orientamento"));
        assert_eq!(valore_coppia(&eventi.try_recv().unwrap().dati, "evento"), Some("fine"));
        assert!(altri.try_recv().is_err());
        // Dopo «fine» la sessione è dimenticata: il ricevitore si chiude.
        assert!(matches!(eventi.try_recv(), Err(mpsc::error::TryRecvError::Disconnected)));
        s.dimentica(5);
        assert!(matches!(altri.try_recv(), Err(mpsc::error::TryRecvError::Disconnected)));
        // Gli eventi orfani troppo vecchi si scartano.
        s.ricevuto(evento("evento=protetta\nid=9\nprotetta=1\n"), t0);
        s.ricevuto(evento("evento=protetta\nid=10\nprotetta=1\n"), t0 + ATTESA_ORFANI);
        assert_eq!(s.orfani.iter().map(|(_, id, _)| *id).collect::<Vec<_>>(), vec![10]);
        // Apertura senza id: errore.
        let (tx, mut rx) = oneshot::channel();
        s.attendi(11, t0, Attesa::Sessione(tx));
        s.ricevuto(risposta(tipo::VIDEO_APRI, 11, "display=1\n"), t0);
        assert!(rx.try_recv().unwrap().is_err());
    }

    #[test]
    fn smistamento_per_tipo() {
        let t0 = Instant::now();
        let mut s = Smistamento::default();
        let (tx, mut appunti) = mpsc::unbounded_channel();
        s.iscrivi(0x58, tx);
        let (tx, chiuso) = mpsc::unbounded_channel();
        s.iscrivi(0x58, tx);
        drop(chiuso);
        s.ricevuto(Messaggio::new(0x58, vec![1, b'x']), t0);
        // Un tipo senza iscritti si scarta senza disturbare gli altri.
        s.ricevuto(Messaggio::new(0x7f, vec![]), t0);
        assert_eq!(appunti.try_recv().unwrap().dati, vec![1, b'x']);
        assert!(appunti.try_recv().is_err());
        assert_eq!(s.iscritti[&0x58].len(), 1);
    }

    #[test]
    fn coppie_nei_messaggi() {
        assert_eq!(valore_coppia(b"evento=fine\nid=3\nmotivo=a=b\n", "motivo"), Some("a=b"));
        assert_eq!(valore_coppia(b"id=3", "evento"), None);
        assert_eq!(valore_coppia(&[0xff, b'='], "x"), None);
    }
}
