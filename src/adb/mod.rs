//! Livello ADB di Phonestra, con più canali aperti insieme.
//!
//! `adb_client` legge tutti i canali dallo stesso collegamento senza smistare i
//! messaggi, quindi non regge video + comandi + audio in parallelo. Qui un
//! compito di lettura smista ogni messaggio al canale giusto (per id locale) e
//! ogni canale rispetta il controllo di flusso di ADB: un solo `WRTE` in volo,
//! il successivo solo dopo l'`OKAY` del telefono.

pub mod abbina;
mod messaggio;
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

use messaggio::{CLSE, CNXN, MAX_DATI, Messaggio, OKAY, OPEN, STLS, VERSIONE, VERSIONE_STLS, WRTE};

type Scrittore = Box<dyn AsyncWrite + Send + Unpin>;

/// Stato di un canale lato lettore.
struct Registro {
    /// Aperture in attesa dell'OKAY (o del CLSE di rifiuto).
    in_apertura: HashMap<u32, oneshot::Sender<Option<u32>>>,
    /// Canali aperti: dati in arrivo e conferme dei nostri WRTE.
    aperti: HashMap<u32, (mpsc::UnboundedSender<Vec<u8>>, mpsc::UnboundedSender<()>)>,
}

/// Un collegamento ADB autenticato con il telefono.
#[derive(Clone)]
pub struct Adb {
    scrittore: Arc<Mutex<Scrittore>>,
    registro: Arc<std::sync::Mutex<Registro>>,
    prossimo_id: Arc<AtomicU32>,
    max_dati: usize,
    pub dispositivo: String,
}

/// Un canale aperto verso un servizio del telefono (`shell:…`, `sync:`,
/// `localabstract:…`).
pub struct Canale {
    adb: Adb,
    locale: u32,
    remoto: u32,
    dati: mpsc::UnboundedReceiver<Vec<u8>>,
    conferme: mpsc::UnboundedReceiver<()>,
    avanzo: Vec<u8>,
}

impl Adb {
    /// Collegamento Wi-Fi cifrato all'indirizzo del Debug wireless.
    pub async fn wifi(indirizzo: SocketAddr, chiave: &Path) -> Result<Self> {
        let config = tls::configurazione_client(chiave)?;
        let mut tcp = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(indirizzo))
            .await
            .context("tempo scaduto")?
            .with_context(|| format!("collegamento a {indirizzo} non riuscito"))?;
        tcp.set_nodelay(true)?;
        Messaggio::new(CNXN, VERSIONE, MAX_DATI, b"host::features=shell_v2,cmd,stat_v2\0".to_vec())
            .scrivi(&mut tcp)
            .await?;
        let risposta = Messaggio::leggi(&mut tcp).await?;
        if risposta.comando != STLS {
            bail!("il telefono ha risposto {} invece di STLS", messaggio::nome(risposta.comando));
        }
        Messaggio::new(STLS, VERSIONE_STLS, 0, Vec::new()).scrivi(&mut tcp).await?;
        let nome = rustls::pki_types::ServerName::IpAddress(indirizzo.ip().into());
        let mut flusso = TlsConnector::from(config)
            .connect(nome, tcp)
            .await
            .context("handshake TLS rifiutato (Phonestra è autorizzato con «Consenti sempre»?)")?;
        let benvenuto = Messaggio::leggi(&mut flusso).await?;
        if benvenuto.comando != CNXN {
            bail!("atteso CNXN dopo TLS, arrivato {}", messaggio::nome(benvenuto.comando));
        }
        let max_dati = benvenuto.arg1.clamp(4096, MAX_DATI) as usize;
        let dispositivo = String::from_utf8_lossy(&benvenuto.dati).trim_end_matches('\0').to_string();
        let (lettore, scrittore) = tokio::io::split(flusso);
        Ok(Self::avvia(lettore, scrittore, max_dati, dispositivo))
    }

    fn avvia<T>(lettore: ReadHalf<T>, scrittore: WriteHalf<T>, max_dati: usize, dispositivo: String) -> Self
    where
        T: AsyncRead + AsyncWrite + Send + 'static,
    {
        let adb = Self {
            scrittore: Arc::new(Mutex::new(Box::new(scrittore))),
            registro: Arc::new(std::sync::Mutex::new(Registro { in_apertura: HashMap::new(), aperti: HashMap::new() })),
            prossimo_id: Arc::new(AtomicU32::new(1)),
            max_dati,
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
                    let mut r = self.registro.lock().unwrap();
                    if let Some(tx) = r.in_apertura.remove(&m.arg1) {
                        let _ = tx.send(Some(m.arg0));
                    } else if let Some((_, conferme)) = r.aperti.get(&m.arg1) {
                        let _ = conferme.send(());
                    }
                }
                WRTE => {
                    let consegnato = {
                        let r = self.registro.lock().unwrap();
                        r.aperti.get(&m.arg1).map(|(dati, _)| dati.send(m.dati).is_ok())
                    };
                    if consegnato == Some(true) {
                        self.invia(Messaggio::new(OKAY, m.arg1, m.arg0, Vec::new())).await?;
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

    /// Apre un canale verso un servizio del telefono.
    pub async fn apri(&self, servizio: &str) -> Result<Canale> {
        let locale = self.prossimo_id.fetch_add(1, Ordering::Relaxed);
        let (tx_apertura, rx_apertura) = oneshot::channel();
        let (tx_dati, dati) = mpsc::unbounded_channel();
        let (tx_conferme, conferme) = mpsc::unbounded_channel();
        {
            let mut r = self.registro.lock().unwrap();
            r.in_apertura.insert(locale, tx_apertura);
            r.aperti.insert(locale, (tx_dati, tx_conferme));
        }
        let mut destinazione = servizio.as_bytes().to_vec();
        destinazione.push(0);
        self.invia(Messaggio::new(OPEN, locale, 0, destinazione)).await?;
        let remoto = tokio::time::timeout(Duration::from_secs(10), rx_apertura)
            .await
            .map_err(|_| anyhow!("il telefono non risponde all'apertura di «{servizio}»"))?
            .map_err(|_| anyhow!("collegamento perso"))?
            .ok_or_else(|| anyhow!("il telefono ha rifiutato «{servizio}»"))?;
        Ok(Canale { adb: self.clone(), locale, remoto, dati, conferme, avanzo: Vec::new() })
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
    /// Scrive dati, spezzandoli secondo il massimo concordato e aspettando la
    /// conferma di ogni blocco.
    pub async fn scrivi(&mut self, dati: &[u8]) -> Result<()> {
        for blocco in dati.chunks(self.adb.max_dati) {
            self.adb.invia(Messaggio::new(WRTE, self.locale, self.remoto, blocco.to_vec())).await?;
            self.conferme.recv().await.ok_or_else(|| anyhow!("canale chiuso dal telefono"))?;
        }
        Ok(())
    }

    /// Prossimo blocco di dati; `None` quando il canale è chiuso.
    pub async fn leggi(&mut self) -> Option<Vec<u8>> {
        if !self.avanzo.is_empty() {
            return Some(std::mem::take(&mut self.avanzo));
        }
        self.dati.recv().await
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
