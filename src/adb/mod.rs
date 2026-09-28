//! Livello ADB di Phonestra, con più canali aperti insieme.
//!
//! `adb_client` legge tutti i canali dallo stesso collegamento senza smistare i
//! messaggi, quindi non regge video + comandi + audio in parallelo. Qui un
//! compito di lettura smista ogni messaggio al canale giusto (per id locale) e
//! ogni canale rispetta il controllo di flusso di ADB:
//! - col *delayed ack* (se il telefono lo annuncia, da Android 14 sempre) più
//!   `WRTE` in volo per canale finché c'è saldo, con finestre per canale
//!   ([`Trasporto`], [`OpzioniCanale`], `memoria/adb.md`);
//! - senza, un solo `WRTE` in volo, il successivo solo dopo l'`OKAY`.

pub mod abbina;
pub mod flusso;
mod messaggio;
pub mod misura;
#[cfg(test)]
mod prove;
pub mod sync;
mod tls;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio_rustls::TlsConnector;

use flusso::Saldo;
use messaggio::{CLSE, CNXN, MAX_DATI, Messaggio, OKAY, STLS, VERSIONE, VERSIONE_STLS, WRTE};

type Scrittore = Box<dyn AsyncWrite + Send + Unpin>;

/// Parametri del trasporto, uguali per tutto il collegamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trasporto {
    /// Annunciare il *delayed ack* nel CNXN (si usa solo se anche il telefono
    /// lo annuncia).
    pub delayed_ack: bool,
    /// Dimensione massima dei dati di un messaggio che dichiariamo nel CNXN:
    /// vale per tutti e due i versi (adbd usa il minimo tra i due CNXN).
    pub max_payload: u32,
    /// Finestra predefinita per canale, col *delayed ack*: byte che il
    /// telefono può mandarci senza conferma, e byte che noi mandiamo al più
    /// senza conferma.
    pub finestra: u32,
}

impl Trasporto {
    /// 64 KiB: tutti i canali condividono una sola connessione TCP+TLS e un
    /// `WRTE` occupa il filo per intero; a 40 Mbit/s 64 KiB sono ~13 ms di
    /// attesa al massimo per l'audio o un comando che arriva dietro, contro
    /// ~200 ms di un blocco da 1 MiB. I 24 byte di intestazione costano lo
    /// 0,04 %. Senza *delayed ack* invece un blocco piccolo limita un canale a
    /// 64 KiB per andata e ritorno: per questo sui telefoni che non lo offrono
    /// il valore si può alzare (`PHONESTRA_ADB_PAYLOAD`).
    pub const MAX_PAYLOAD: u32 = 64 * 1024;
    /// 256 KiB: quattro blocchi in volo. A 100 ms di andata e ritorno (Wi-Fi in
    /// risparmio energetico) un canale arriva a ~2,5 MB/s, più del video di
    /// Phonestra; a 5 MB/s un canale pieno fa aspettare gli altri al più
    /// ~50 ms. Una finestra da 32 MiB come quella di adbd lascerebbe
    /// accumulare secondi di video in coda (SPECIFICHE §14: mai ritardo).
    pub const FINESTRA: u32 = 256 * 1024;

    /// I valori predefiniti, cambiati dalle variabili d'ambiente per le prove:
    /// `PHONESTRA_ADB_DELAYED_ACK=0`, `PHONESTRA_ADB_PAYLOAD=<byte|64k|1m>`,
    /// `PHONESTRA_ADB_FINESTRA=<byte|256k|1m>`.
    pub fn da_ambiente() -> Self {
        let var = |nome: &str| std::env::var(nome).ok();
        let predefinito = Self::default();
        Self {
            delayed_ack: var("PHONESTRA_ADB_DELAYED_ACK").map_or(predefinito.delayed_ack, |v| v.trim() != "0"),
            max_payload: var("PHONESTRA_ADB_PAYLOAD")
                .and_then(|v| flusso::dimensione(&v))
                .unwrap_or(predefinito.max_payload),
            finestra: var("PHONESTRA_ADB_FINESTRA")
                .and_then(|v| flusso::dimensione(&v))
                .unwrap_or(predefinito.finestra),
        }
        .normalizzato()
    }

    /// Valori dentro i limiti del protocollo: blocchi tra 4 KiB e 1 MiB,
    /// finestra positiva e rappresentabile in un int32 (gli `OKAY`).
    pub fn normalizzato(self) -> Self {
        Self {
            delayed_ack: self.delayed_ack,
            max_payload: self.max_payload.clamp(4096, MAX_DATI),
            finestra: self.finestra.clamp(1, i32::MAX as u32),
        }
    }
}

impl Default for Trasporto {
    fn default() -> Self {
        // Spento finché non funziona sul telefono vero: il 28 set, attivo, adbd
        // rifiutava ogni OPEN (memoria/adb.md). Si prova con
        // PHONESTRA_ADB_DELAYED_ACK=1 e PHONESTRA_ADB_PAYLOAD=64k.
        Self { delayed_ack: false, max_payload: 1024 * 1024, finestra: Self::FINESTRA }
    }
}

/// Scelte per un singolo canale (contano solo col *delayed ack*).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpzioniCanale {
    /// Finestra di questo canale al posto di [`Trasporto::finestra`].
    pub finestra: Option<u32>,
    /// Confermare i dati solo quando [`Canale::leggi`] li consegna, invece
    /// che appena arrivano: la contropressione arriva fino al processo sul
    /// telefono (la coda sul PC resta entro la finestra), ma un canale che
    /// nessuno legge ferma chi scrive sul telefono.
    pub conferma_alla_lettura: bool,
}

/// Un canale aperto, lato lettore.
struct Voce {
    dati: mpsc::UnboundedSender<Vec<u8>>,
    /// Byte confermati dagli `OKAY` del telefono (0 senza *delayed ack*).
    conferme: mpsc::UnboundedSender<i32>,
    alla_lettura: bool,
}

/// Risposta a un `OPEN`: id del telefono e saldo concesso (col *delayed ack*).
type Apertura = Option<(u32, Option<i32>)>;

/// Stato dei canali lato lettore.
struct Registro {
    /// Aperture in attesa dell'OKAY (o del CLSE di rifiuto).
    in_apertura: HashMap<u32, oneshot::Sender<Apertura>>,
    /// Canali aperti: dati in arrivo e conferme dei nostri WRTE.
    aperti: HashMap<u32, Voce>,
}

/// Un collegamento ADB autenticato con il telefono.
#[derive(Clone)]
pub struct Adb {
    scrittore: Arc<Mutex<Scrittore>>,
    registro: Arc<std::sync::Mutex<Registro>>,
    prossimo_id: Arc<AtomicU32>,
    max_dati: usize,
    delayed_ack: bool,
    trasporto: Trasporto,
    /// Messaggi brevi (conferme alla lettura) scritti da un compito a parte,
    /// così [`Canale::leggi`] non aspetta la rete e resta annullabile.
    posta: mpsc::UnboundedSender<Messaggio>,
    pub dispositivo: String,
}

/// Come mandare i `WRTE` di un canale.
enum Invio {
    /// Senza *delayed ack*: ogni `WRTE` aspetta il suo `OKAY`.
    UnoAllaVolta,
    /// Col *delayed ack*: si manda finché il saldo è positivo.
    ASaldo(Saldo),
}

/// Un canale aperto verso un servizio del telefono (`shell:…`, `sync:`,
/// `localabstract:…`).
pub struct Canale {
    adb: Adb,
    locale: u32,
    remoto: u32,
    dati: mpsc::UnboundedReceiver<Vec<u8>>,
    conferme: mpsc::UnboundedReceiver<i32>,
    avanzo: Vec<u8>,
    invio: Invio,
    alla_lettura: bool,
}

impl Adb {
    /// Collegamento Wi-Fi cifrato all'indirizzo del Debug wireless, col
    /// trasporto predefinito (o quello delle variabili d'ambiente di prova).
    pub async fn wifi(indirizzo: SocketAddr, chiave: &Path) -> Result<Self> {
        Self::wifi_con(indirizzo, chiave, Trasporto::da_ambiente()).await
    }

    /// Come [`Adb::wifi`], con un trasporto scelto (misure).
    pub async fn wifi_con(indirizzo: SocketAddr, chiave: &Path, trasporto: Trasporto) -> Result<Self> {
        let trasporto = trasporto.normalizzato();
        let config = tls::configurazione_client(chiave)?;
        let mut tcp = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(indirizzo))
            .await
            .context("tempo scaduto")?
            .with_context(|| format!("collegamento a {indirizzo} non riuscito"))?;
        tcp.set_nodelay(true)?;
        // Il telefono legge le nostre funzioni da questo CNXN, prima del TLS.
        Messaggio::new(CNXN, VERSIONE, trasporto.max_payload, flusso::banner_host(trasporto.delayed_ack))
            .scrivi(&mut tcp)
            .await?;
        let risposta = Messaggio::leggi(&mut tcp).await?;
        if risposta.comando != STLS {
            bail!("il telefono ha risposto {} invece di STLS", messaggio::nome(risposta.comando));
        }
        Messaggio::new(STLS, VERSIONE_STLS, 0, Vec::new()).scrivi(&mut tcp).await?;
        let nome = rustls::pki_types::ServerName::IpAddress(indirizzo.ip().into());
        let mut cifrato = TlsConnector::from(config)
            .connect(nome, tcp)
            .await
            .context("handshake TLS rifiutato (Phonestra è autorizzato con «Consenti sempre»?)")?;
        let benvenuto = Messaggio::leggi(&mut cifrato).await?;
        if benvenuto.comando != CNXN {
            bail!("atteso CNXN dopo TLS, arrivato {}", messaggio::nome(benvenuto.comando));
        }
        // adbd risponde col minimo tra il suo massimo e il nostro.
        let max_dati = benvenuto.arg1.clamp(4096, MAX_DATI).min(trasporto.max_payload) as usize;
        let dispositivo = String::from_utf8_lossy(&benvenuto.dati).trim_end_matches('\0').to_string();
        let delayed_ack = trasporto.delayed_ack && flusso::offre_delayed_ack(&dispositivo);
        let (lettore, scrittore) = tokio::io::split(cifrato);
        Ok(Self::avvia(lettore, scrittore, max_dati, delayed_ack, trasporto, dispositivo))
    }

    fn avvia<T>(
        lettore: ReadHalf<T>,
        scrittore: WriteHalf<T>,
        max_dati: usize,
        delayed_ack: bool,
        trasporto: Trasporto,
        dispositivo: String,
    ) -> Self
    where
        T: AsyncRead + AsyncWrite + Send + 'static,
    {
        let scrittore: Arc<Mutex<Scrittore>> = Arc::new(Mutex::new(Box::new(scrittore)));
        let (posta, mut da_spedire) = mpsc::unbounded_channel::<Messaggio>();
        let per_la_posta = scrittore.clone();
        tokio::spawn(async move {
            while let Some(m) = da_spedire.recv().await {
                let mut s = per_la_posta.lock().await;
                if m.scrivi(&mut *s).await.is_err() {
                    break;
                }
            }
        });
        let adb = Self {
            scrittore,
            registro: Arc::new(std::sync::Mutex::new(Registro { in_apertura: HashMap::new(), aperti: HashMap::new() })),
            prossimo_id: Arc::new(AtomicU32::new(1)),
            max_dati,
            delayed_ack,
            trasporto,
            posta,
            dispositivo,
        };
        let copia = adb.clone();
        tokio::spawn(async move {
            if let Err(e) = copia.leggi_sempre(lettore).await {
                log_errore(&format!("collegamento ADB chiuso: {e:#}"));
            }
            // Collegamento perso: si chiudono tutti i canali.
            let mut r = copia.registro.lock().unwrap();
            r.in_apertura.clear();
            r.aperti.clear();
        });
        adb
    }

    async fn leggi_sempre<T: AsyncRead + Send>(&self, mut lettore: ReadHalf<T>) -> Result<()> {
        loop {
            let m = Messaggio::leggi(&mut lettore).await?;
            // arg1 = nostro id locale per OKAY/WRTE/CLSE
            match m.comando {
                OKAY => {
                    let confermati = match flusso::byte_confermati(&m) {
                        Ok(n) => n,
                        Err(e) => {
                            log_errore(&format!("{e:#}"));
                            continue;
                        }
                    };
                    let mut r = self.registro.lock().unwrap();
                    if let Some(tx) = r.in_apertura.remove(&m.arg1) {
                        let _ = tx.send(Some((m.arg0, confermati)));
                    } else if let Some(voce) = r.aperti.get(&m.arg1) {
                        let _ = voce.conferme.send(confermati.unwrap_or(0));
                    }
                }
                WRTE => {
                    let lunghezza = m.dati.len() as u32;
                    let esito = {
                        let r = self.registro.lock().unwrap();
                        r.aperti.get(&m.arg1).map(|voce| (voce.dati.send(m.dati).is_ok(), voce.alla_lettura))
                    };
                    // Con la conferma alla lettura l'OKAY lo manda `Canale::leggi`.
                    if esito == Some((true, false)) {
                        let byte = self.delayed_ack.then_some(lunghezza);
                        self.invia(flusso::conferma(m.arg1, m.arg0, byte)).await?;
                    }
                }
                CLSE => {
                    let mut r = self.registro.lock().unwrap();
                    if let Some(tx) = r.in_apertura.remove(&m.arg1) {
                        let _ = tx.send(None);
                    }
                    r.aperti.remove(&m.arg1);
                }
                _ => {}
            }
        }
    }

    async fn invia(&self, m: Messaggio) -> Result<()> {
        let mut s = self.scrittore.lock().await;
        m.scrivi(&mut *s).await
    }

    /// Il *delayed ack* è attivo su questo collegamento (annunciato da tutte e
    /// due le parti)?
    pub fn delayed_ack(&self) -> bool {
        self.delayed_ack
    }

    /// Dimensione massima dei dati di un messaggio, concordata col telefono.
    pub fn max_dati(&self) -> usize {
        self.max_dati
    }

    pub fn trasporto(&self) -> Trasporto {
        self.trasporto
    }

    /// Apre un canale verso un servizio del telefono.
    pub async fn apri(&self, servizio: &str) -> Result<Canale> {
        self.apri_con(servizio, OpzioniCanale::default()).await
    }

    /// Come [`Adb::apri`], con finestra e modo di conferma scelti.
    pub async fn apri_con(&self, servizio: &str, opzioni: OpzioniCanale) -> Result<Canale> {
        let finestra = opzioni.finestra.unwrap_or(self.trasporto.finestra).clamp(1, i32::MAX as u32);
        let alla_lettura = self.delayed_ack && opzioni.conferma_alla_lettura;
        let locale = self.prossimo_id.fetch_add(1, Ordering::Relaxed);
        let (tx_apertura, rx_apertura) = oneshot::channel();
        let (tx_dati, dati) = mpsc::unbounded_channel();
        let (tx_conferme, conferme) = mpsc::unbounded_channel();
        {
            let mut r = self.registro.lock().unwrap();
            r.in_apertura.insert(locale, tx_apertura);
            r.aperti.insert(locale, Voce { dati: tx_dati, conferme: tx_conferme, alla_lettura });
        }
        let risposta = async {
            self.invia(flusso::apertura(locale, self.delayed_ack.then_some(finestra), servizio)).await?;
            tokio::time::timeout(Duration::from_secs(10), rx_apertura)
                .await
                .map_err(|_| anyhow!("il telefono non risponde all'apertura di «{servizio}»"))?
                .map_err(|_| anyhow!("collegamento perso"))?
                .ok_or_else(|| anyhow!("il telefono ha rifiutato «{servizio}»"))
        }
        .await;
        let (remoto, concesso) = match risposta {
            Ok(r) => r,
            Err(e) => {
                let mut r = self.registro.lock().unwrap();
                r.in_apertura.remove(&locale);
                r.aperti.remove(&locale);
                return Err(e);
            }
        };
        let invio = match (self.delayed_ack, concesso) {
            (true, Some(concesso)) => Invio::ASaldo(Saldo::nuovo(concesso, finestra)),
            (true, None) => {
                log_errore(&format!("«{servizio}»: OKAY senza saldo col delayed ack, un WRTE alla volta"));
                Invio::UnoAllaVolta
            }
            (false, _) => Invio::UnoAllaVolta,
        };
        Ok(Canale { adb: self.clone(), locale, remoto, dati, conferme, avanzo: Vec::new(), invio, alla_lettura })
    }

    /// Esegue un comando e ne restituisce l'uscita (servizio `exec:`, senza
    /// terminale: i byte arrivano intatti).
    pub async fn esegui(&self, comando: &str) -> Result<String> {
        let mut c = self.apri(&format!("exec:{comando}")).await?;
        let uscita = c.leggi_tutto().await?;
        Ok(String::from_utf8_lossy(&uscita).trim_end().to_string())
    }
}

impl Canale {
    /// Scrive dati, spezzandoli secondo il massimo concordato. Senza *delayed
    /// ack* aspetta la conferma di ogni blocco; con, manda finché c'è saldo e
    /// torna appena l'ultimo blocco è partito (adbd consegna comunque tutto
    /// prima di chiudere il canale).
    pub async fn scrivi(&mut self, dati: &[u8]) -> Result<()> {
        for blocco in dati.chunks(self.adb.max_dati) {
            match &mut self.invio {
                Invio::UnoAllaVolta => {
                    self.adb.invia(Messaggio::new(WRTE, self.locale, self.remoto, blocco.to_vec())).await?;
                    self.conferme.recv().await.ok_or_else(|| anyhow!("canale chiuso dal telefono"))?;
                }
                Invio::ASaldo(saldo) => {
                    while let Ok(n) = self.conferme.try_recv() {
                        saldo.confermati(n);
                    }
                    while !saldo.puo_mandare() {
                        let n = self.conferme.recv().await.ok_or_else(|| anyhow!("canale chiuso dal telefono"))?;
                        saldo.confermati(n);
                    }
                    self.adb.invia(Messaggio::new(WRTE, self.locale, self.remoto, blocco.to_vec())).await?;
                    saldo.mandati(blocco.len());
                }
            }
        }
        Ok(())
    }

    /// Prossimo blocco di dati; `None` quando il canale è chiuso.
    pub async fn leggi(&mut self) -> Option<Vec<u8>> {
        if !self.avanzo.is_empty() {
            return Some(std::mem::take(&mut self.avanzo));
        }
        let blocco = self.dati.recv().await;
        // Niente attese dopo `recv`: se il chiamante annulla (timeout,
        // `select!`) il blocco non va perso. La conferma parte dalla posta.
        if self.alla_lettura
            && let Some(b) = &blocco
        {
            let _ = self.adb.posta.send(flusso::conferma(self.locale, self.remoto, Some(b.len() as u32)));
        }
        blocco
    }

    /// Esattamente `n` byte (per i protocolli a lunghezza fissa).
    pub async fn leggi_esatti(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut v = Vec::with_capacity(n);
        while v.len() < n {
            let mut blocco = self.leggi().await.ok_or_else(|| anyhow!("canale chiuso a metà"))?;
            let manca = n - v.len();
            if blocco.len() > manca {
                self.avanzo = blocco.split_off(manca);
            }
            v.extend_from_slice(&blocco);
        }
        Ok(v)
    }

    pub async fn leggi_tutto(&mut self) -> Result<Vec<u8>> {
        let mut v = Vec::new();
        while let Some(blocco) = self.leggi().await {
            v.extend_from_slice(&blocco);
        }
        Ok(v)
    }

    pub async fn chiudi(self) -> Result<()> {
        self.chiusore().chiudi().await
    }

    /// Maniglia per chiudere il canale anche dopo averlo passato a un altro
    /// compito (per esempio quello che legge il video).
    pub fn chiusore(&self) -> Chiusore {
        Chiusore { adb: self.adb.clone(), locale: self.locale, remoto: self.remoto }
    }
}

/// Vedi [`Canale::chiusore`].
pub struct Chiusore {
    adb: Adb,
    locale: u32,
    remoto: u32,
}

impl Chiusore {
    pub async fn chiudi(self) -> Result<()> {
        self.adb.registro.lock().unwrap().aperti.remove(&self.locale);
        self.adb.invia(Messaggio::new(CLSE, self.locale, self.remoto, Vec::new())).await
    }
}

fn log_errore(testo: &str) {
    eprintln!("[adb] {testo}");
}
