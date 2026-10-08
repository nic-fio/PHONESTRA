//! L'audio del telefono col componente nostro (notes/component.md,
//! «Audio»): il canale `audio` del servizio (`CanaleAudio.java`), cattura
//! loopback, Opus 128 kbit/s (PCM come riserva; fino alla 1.1.1 AAC-LC, tolto
//! insieme a FFmpeg l'8 ott 2026), orari dal conteggio dei
//! campioni sull'orologio monotono del telefono. Uscita con GStreamer (`autoaudiosink`), orari regolari e margine
//! di riproduzione, copie dei pacchetti per chi registra.
//!
//! Il collegamento chiama [`riproduci`] col servizio condiviso già avviato.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use gst::prelude::*;

use crate::adb::Canale;
use crate::componente::Apritore;
use crate::misura_audio::Riga;

/// Frequenza e canali dell'audio del telefono.
pub const FREQUENZA: u64 = 48_000;
pub const CANALI: u32 = 2;
/// Campioni (per canale) in un pacchetto Opus del codificatore di Android (20 ms).
pub const CAMPIONI_OPUS: u64 = 960;

/// Bandiere nell'orario dei pacchetti (`Audio.java`): configurazione del codec, testo.
const CONFIGURAZIONE: u64 = 1 << 62;
const TESTO: u64 = 1 << 61;
/// Un pacchetto più grande è un flusso rovinato (uno Opus è ~300 byte, uno PCM 4 KB).
const MASSIMO: usize = 1 << 20;
/// Tempo massimo per la riga `inizio` dopo l'apertura del canale.
const ATTESA_INIZIO: Duration = Duration::from_secs(10);

/// Formato dell'audio sul canale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formato {
    /// Opus 128 kbit/s, 48 kHz, stereo: decodificato da libopus, senza FFmpeg
    /// (notes/user-decisions.md, 8 ott 2026; prima AAC-LC, prove §42).
    Opus,
    /// PCM 16 bit little-endian, 48 kHz, stereo (riserva e misure).
    Pcm,
}

impl Formato {
    /// Tipo del canale da aprire (`CanaleAudio.formato`).
    pub fn tipo_canale(self) -> &'static str {
        match self {
            Formato::Opus => "audio:opus",
            Formato::Pcm => "audio:pcm",
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Formato::Opus => "opus",
            Formato::Pcm => "pcm",
        }
    }

    /// Dal nome (`opus`, `pcm`; `raw` vale come `pcm`).
    pub fn da_nome(nome: &str) -> Option<Self> {
        match nome {
            "opus" => Some(Formato::Opus),
            "pcm" | "raw" => Some(Formato::Pcm),
            _ => None,
        }
    }

    /// Campioni (per canale) nel pacchetto di dati `dati`.
    pub fn campioni(self, dati: &[u8]) -> u64 {
        match self {
            Formato::Opus => campioni_opus(dati).unwrap_or(CAMPIONI_OPUS),
            Formato::Pcm => dati.len() as u64 / (2 * u64::from(CANALI)),
        }
    }
}

/// Campioni (per canale, a 48 kHz) di un pacchetto Opus, dal suo primo byte
/// (TOC, RFC 6716 §3.1): durata di un frame dalla configurazione, numero di
/// frame dal codice. `None` se il pacchetto è vuoto o rovinato.
pub fn campioni_opus(dati: &[u8]) -> Option<u64> {
    let toc = *dati.first()?;
    let configurazione = toc >> 3;
    // Durata di un frame in 1/400 di secondo (120 campioni a 48 kHz).
    let quarti_di_ms: u64 = match configurazione {
        0..=11 => [4, 8, 16, 24][usize::from(configurazione % 4)], // SILK: 10, 20, 40, 60 ms
        12..=15 => [4, 8][usize::from(configurazione % 2)],        // ibrido: 10, 20 ms
        _ => [1, 2, 4, 8][usize::from(configurazione % 4)],        // CELT: 2,5, 5, 10, 20 ms
    };
    let frame = match toc & 0x03 {
        0 => 1,
        1 | 2 => 2,
        _ => u64::from(*dati.get(1)? & 0x3f),
    };
    (frame > 0).then_some(frame * quarti_di_ms * 120)
}

/// Un pacchetto del canale audio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pacchetto {
    /// Configurazione del codec (Opus: quella del codificatore di Android), prima dei dati.
    Configurazione(Vec<u8>),
    /// Riga di testo del telefono: `inizio`, `lettura`, `misura`, `avviso`, `errore`.
    Testo(String),
    /// Audio: orario in µs dal conteggio dei campioni, sull'orologio monotono
    /// del telefono (lo stesso dei fotogrammi).
    Dati { pts: u64, dati: Vec<u8> },
}

/// Ricompone i pacchetti dai blocchi ADB, che li tagliano in qualsiasi punto:
/// `orario u64 BE (bit 62 configurazione, bit 61 testo) · lunghezza u32 BE · dati`.
#[derive(Debug, Default)]
pub struct Lettore {
    ricevuti: Vec<u8>,
}

impl Lettore {
    pub fn aggiungi(&mut self, blocco: &[u8]) {
        self.ricevuti.extend_from_slice(blocco);
    }

    /// Il prossimo pacchetto completo; errore se la lunghezza è impossibile.
    pub fn prossimo(&mut self) -> Result<Option<Pacchetto>> {
        if self.ricevuti.len() < 12 {
            return Ok(None);
        }
        let testa = u64::from_be_bytes(self.ricevuti[0..8].try_into().unwrap());
        let lunghezza = u32::from_be_bytes(self.ricevuti[8..12].try_into().unwrap()) as usize;
        if lunghezza > MASSIMO {
            bail!("pacchetto audio troppo grande ({lunghezza} byte)");
        }
        if self.ricevuti.len() < 12 + lunghezza {
            return Ok(None);
        }
        let dati: Vec<u8> = self.ricevuti[12..12 + lunghezza].to_vec();
        self.ricevuti.drain(..12 + lunghezza);
        Ok(Some(if testa & TESTO != 0 {
            Pacchetto::Testo(String::from_utf8_lossy(&dati).trim().to_string())
        } else if testa & CONFIGURAZIONE != 0 {
            Pacchetto::Configurazione(dati)
        } else {
            Pacchetto::Dati { pts: testa & !(TESTO | CONFIGURAZIONE), dati }
        }))
    }
}

/// Il canale audio aperto, già oltre la riga `inizio`.
pub struct Flusso {
    canale: Canale,
    lettore: Lettore,
    pub formato: Formato,
    /// La riga `inizio` del telefono (buffer, registrazione della politica…).
    pub inizio: Riga,
}

impl Flusso {
    /// Apre il canale `audio` del servizio e aspetta la riga `inizio` (o
    /// l'errore del telefono).
    pub async fn apri(servizio: &Apritore, formato: Formato) -> Result<Self> {
        let canale = servizio.apri(formato.tipo_canale()).await?;
        let mut f = Flusso { canale, lettore: Lettore::default(), formato, inizio: Riga::leggi("") };
        let primo = tokio::time::timeout(ATTESA_INIZIO, f.prossimo())
            .await
            .map_err(|_| anyhow!("il telefono non ha avviato l'audio entro {} s", ATTESA_INIZIO.as_secs()));
        let primo = match primo.and_then(|p| p) {
            Ok(p) => p,
            Err(e) => {
                let _ = f.chiudi().await;
                return Err(e);
            }
        };
        match primo {
            Some(Pacchetto::Testo(t)) if t.starts_with("inizio") => {
                f.inizio = Riga::leggi(&t);
                let dichiarato = f.inizio.valore("formato").and_then(Formato::da_nome);
                if dichiarato != Some(formato) {
                    let testo = f.inizio.testo.clone();
                    let _ = f.chiudi().await;
                    bail!("il telefono non manda l'audio in {} («{testo}»)", formato.nome());
                }
                Ok(f)
            }
            altro => {
                let _ = f.chiudi().await;
                match altro {
                    Some(Pacchetto::Testo(t)) => bail!("il telefono non ha avviato l'audio: {t}"),
                    None => bail!("il telefono ha chiuso il canale audio (servizio vecchio senza audio?)"),
                    Some(_) => bail!("primo pacchetto audio inatteso (manca la riga «inizio»)"),
                }
            }
        }
    }

    /// Il prossimo pacchetto; `None` quando il telefono chiude il canale.
    pub async fn prossimo(&mut self) -> Result<Option<Pacchetto>> {
        loop {
            if let Some(p) = self.lettore.prossimo()? {
                return Ok(Some(p));
            }
            match self.canale.leggi().await {
                Some(blocco) => self.lettore.aggiungi(&blocco),
                None => return Ok(None),
            }
        }
    }

    /// Chiude il canale: il telefono ferma la cattura e toglie la politica audio.
    pub async fn chiudi(self) -> Result<()> {
        self.canale.chiudi().await
    }
}

/// Durate esatte dei pacchetti dal conteggio dei campioni: la somma resta
/// uguale agli orari del telefono (campioni × 10⁶ / 48000, per difetto), senza
/// l'errore che si accumulerebbe sommando durate arrotondate (con l'AAC di
/// prima, 21 333 µs a frame).
#[derive(Debug, Default)]
pub struct Durate {
    campioni: u64,
}

impl Durate {
    /// Durata (µs) del prossimo pacchetto di `campioni` campioni.
    pub fn prossima(&mut self, campioni: u64) -> u64 {
        let prima = self.campioni * 1_000_000 / FREQUENZA;
        self.campioni += campioni;
        self.campioni * 1_000_000 / FREQUENZA - prima
    }
}

/// Orari regolari: ogni pacchetto subito dopo il precedente; l'orario del
/// telefono conta solo se se ne discosta più di [`SCOSTAMENTO_MASSIMO_US`]
/// (pausa vera, pacchetti persi per strada). Col componente nostro gli orari
/// del telefono sono già regolari; resta come difesa (con l'audio di prima i
/// pacchetti arrivavano a raffiche e senza questa regola l'audio aveva
/// micro-interruzioni, prove §41).
#[derive(Debug, Default)]
pub struct Orari {
    prossimo: Option<u64>,
}

/// Oltre questo scostamento dall'orario del telefono ci si riallinea a lui.
pub const SCOSTAMENTO_MASSIMO_US: u64 = 60_000;

impl Orari {
    /// Orario regolare (µs, scala del telefono) di un pacchetto lungo
    /// `durata` µs e segnato a `pts`.
    pub fn regola(&mut self, pts: u64, durata: u64) -> u64 {
        let orario = match self.prossimo {
            Some(p) if p.abs_diff(pts) <= SCOSTAMENTO_MASSIMO_US => p,
            _ => pts,
        };
        self.prossimo = Some(orario + durata);
        orario
    }
}

/// Margine iniziale di riproduzione: assorbe le irregolarità del Wi-Fi.
pub const MARGINE_NS: i64 = 80_000_000;
/// Limite del margine: oltre, l'audio sarebbe visibilmente in ritardo sul video.
pub const MARGINE_MASSIMO_NS: i64 = 300_000_000;

/// Quando riprodurre ogni pacchetto: orario del telefono più uno scarto fisso
/// (orologio del telefono → orologio della pipeline, più il margine). Il
/// primo pacchetto fissa lo scarto; un
/// pacchetto in ritardo sposta tutto più avanti (un attimo di silenzio, poi
/// niente buchi) e allarga il margine di 40 ms fino a 300; un telefono molto
/// più avanti (orologi allontanati) fa riallineare.
#[derive(Debug)]
pub struct Margine {
    scarto: Option<i64>,
    /// Margine attuale (ns).
    pub margine: i64,
    /// Pacchetti arrivati in ritardo (dall'inizio).
    pub ritardi: u64,
    /// Riallineamenti degli orologi (dall'inizio).
    pub riallineamenti: u64,
    /// Ultimo cambiamento del margine (ns, orologio della pipeline).
    ultimo_cambio: i64,
    /// Passi in giù fatti (dall'inizio).
    pub discese: u64,
}

impl Default for Margine {
    fn default() -> Self {
        Self { scarto: None, margine: MARGINE_NS, ritardi: 0, riallineamenti: 0, ultimo_cambio: 0, discese: 0 }
    }
}

/// Senza ritardi per tanto tempo il margine può scendere di un passo (§52):
/// cresce a ogni pacchetto in ritardo e, senza discesa, resterebbe alto per
/// sempre (audio in ritardo sul video di qualche decina di ms).
const CALMA_PER_SCENDERE_NS: i64 = 10_000_000_000;

impl Margine {
    /// Orario nella pipeline (ns) del pacchetto segnato `pts_ns` (orologio del
    /// telefono), arrivato quando la pipeline era a `ora` ns.
    pub fn orario(&mut self, ora: i64, pts_ns: i64) -> i64 {
        match self.scarto.map(|s| pts_ns + s) {
            None => self.scarto = Some(ora + self.margine - pts_ns),
            Some(o) if o > ora + self.margine + 200_000_000 => {
                self.riallineamenti += 1;
                self.scarto = Some(ora + self.margine - pts_ns);
            }
            Some(o) if o < ora + 10_000_000 => {
                self.ritardi += 1;
                self.ultimo_cambio = ora;
                // Lo spostamento c'è sempre, anche col margine al massimo: i due
                // orologi (scheda audio del PC e telefono) si allontanano di
                // qualche decimillesimo, e senza spostamento da lì in poi ogni
                // pacchetto resterebbe in ritardo (6 ott 2026, av-sync-tests §13).
                let aumento = (ora + self.margine - o).max(0);
                if self.margine < MARGINE_MASSIMO_NS {
                    self.margine = (self.margine + 40_000_000).min(MARGINE_MASSIMO_NS);
                }
                self.scarto = self.scarto.map(|s| s + aumento);
            }
            Some(_) => {}
        }
        pts_ns + self.scarto.unwrap()
    }

    /// Se il margine è sopra il minimo e da 10 s non ci sono ritardi, scende
    /// di `durata_ns` saltando il pacchetto che dura tanto (un silenzio: lo
    /// sceglie chi chiama). Restituisce se il pacchetto va saltato.
    pub fn scendi(&mut self, ora: i64, durata_ns: i64) -> bool {
        if self.scarto.is_none() || self.margine - durata_ns < MARGINE_NS || ora - self.ultimo_cambio < CALMA_PER_SCENDERE_NS {
            return false;
        }
        self.margine -= durata_ns;
        self.scarto = self.scarto.map(|s| s - durata_ns);
        self.ultimo_cambio = ora;
        self.discese += 1;
        true
    }
}

/// Controllo degli orari del telefono (per le prove): l'orario di ogni
/// pacchetto deve essere quello del primo più i campioni arrivati da allora
/// × 10⁶ / 48000. Uno scostamento vuol dire pacchetti persi (coda piena sul
/// telefono) o orari sbagliati. Il primo orario non cade per forza su un
/// campione intero (con l'Opus quasi mai): si conta da lui, non da un campione
/// arrotondato.
#[derive(Debug, Default)]
pub struct ControlloOrari {
    /// Orario del primo pacchetto (o dell'ultimo salto) e campioni da allora.
    origine: Option<(u64, u64)>,
    /// Pacchetti con l'orario diverso dall'atteso.
    pub irregolari: u64,
    /// Campioni mancanti (salti in avanti), in tutto.
    pub mancanti: u64,
}

impl ControlloOrari {
    /// Controlla un pacchetto di `campioni` campioni segnato a `pts`.
    pub fn controlla(&mut self, pts: u64, campioni: u64) {
        let (origine, contati) = match self.origine {
            None => (pts, 0),
            Some((o, c)) if o + c * 1_000_000 / FREQUENZA == pts => (o, c),
            Some((o, c)) => {
                self.irregolari += 1;
                let atteso = o + c * 1_000_000 / FREQUENZA;
                // Arrotondato: gli orari sono in µs interi, i campioni no.
                self.mancanti += (pts.saturating_sub(atteso) * FREQUENZA + 500_000) / 1_000_000;
                (pts, 0)
            }
        };
        self.origine = Some((origine, contati + campioni));
    }
}

/// Caps GStreamer dei pacchetti Opus del telefono: stereo a 48 kHz con la
/// mappa dei canali 0 (RFC 7845), che non ha bisogno dell'OpusHead. Servono
/// a `opusdec` e a `mp4mux`.
pub fn caps_opus() -> gst::Caps {
    gst::Caps::builder("audio/x-opus")
        .field("channel-mapping-family", 0i32)
        .field("rate", FREQUENZA as i32)
        .field("channels", CANALI as i32)
        .build()
}

/// Caps del PCM del telefono.
pub fn caps_pcm() -> gst::Caps {
    gst::Caps::builder("audio/x-raw")
        .field("format", "S16LE")
        .field("layout", "interleaved")
        .field("rate", FREQUENZA as i32)
        .field("channels", CANALI as i32)
        .build()
}

fn caps(formato: Formato) -> gst::Caps {
    match formato {
        Formato::Opus => caps_opus(),
        Formato::Pcm => caps_pcm(),
    }
}

/// Messaggi di diagnosi, solo con `PHONESTRA_DEBUG=1` (come `crate::diagnosi`).
fn diagnosi(testo: &str) {
    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
        eprintln!("[audio] {testo}");
    }
}

/// La riproduzione dalle casse del PC (`audioconvert ! audioresample !
/// autoaudiosink`), con `opusdec` per l'Opus.
pub struct Riproduzione {
    pipeline: gst::Pipeline,
    sorgente: gst_app::AppSrc,
    orari: Orari,
    durate: Durate,
    formato: Formato,
    pub margine: Margine,
    resoconto: (u64, u64, Instant),
    /// Dimensione media dei pacchetti (media mobile): un pacchetto Opus molto
    /// più piccolo è un silenzio, che si può saltare per far scendere il margine.
    media_byte: f64,
    /// Latenza dell'uscita (ns), per annunciare al video quando suona l'audio.
    latenza: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Riproduzione {
    /// Pipeline verso le casse.
    pub fn nuova(formato: Formato) -> Result<Self> {
        Self::con_uscita(formato, "audioconvert ! audioresample ! autoaudiosink")
    }

    /// Come [`Riproduzione::nuova`] con un'altra uscita (per le prove).
    pub fn con_uscita(formato: Formato, uscita: &str) -> Result<Self> {
        let descrizione = match formato {
            Formato::Opus => format!("appsrc name=sorgente is-live=true format=time max-bytes=65536 ! opusdec name=decodifica ! {uscita}"),
            Formato::Pcm => format!("appsrc name=sorgente is-live=true format=time max-bytes=262144 ! {uscita}"),
        };
        let pipeline = gst::parse::launch(&descrizione)
            .context("pipeline audio (manca il plugin opus di GStreamer?)")?
            .downcast::<gst::Pipeline>()
            .map_err(|_| anyhow!("pipeline audio"))?;
        let sorgente = pipeline
            .by_name("sorgente")
            .and_then(|s| s.downcast::<gst_app::AppSrc>().ok())
            .context("sorgente audio")?;
        sorgente.set_caps(Some(&caps(formato)));
        crate::avsync::sonda_audio(&pipeline, "decodifica");
        let latenza = crate::sincronia::latenza(&pipeline);
        pipeline.set_state(gst::State::Playing).context("la riproduzione audio non parte")?;
        Ok(Self {
            pipeline,
            sorgente,
            orari: Orari::default(),
            durate: Durate::default(),
            formato,
            margine: Margine::default(),
            resoconto: (0, 0, Instant::now()),
            media_byte: 0.0,
            latenza,
        })
    }

    /// Orario regolare (µs, scala del telefono) di un pacchetto di dati: da
    /// usare anche per le copie a chi registra.
    pub fn regola(&mut self, pts: u64, dati: &[u8]) -> u64 {
        let durata = self.durate.prossima(self.formato.campioni(dati));
        self.orari.regola(pts, durata)
    }

    /// Manda alla pipeline un pacchetto con l'orario già regolato.
    pub fn spingi(&mut self, pts: u64, dati: Vec<u8>) -> Result<()> {
        let pts_ns = pts as i64 * 1000;
        let byte = dati.len() as f64;
        // Un silenzio in Opus sono pochi byte (a 128 kbit/s un pacchetto pieno ne ha ~320).
        let silenzio = self.formato == Formato::Opus && self.media_byte > 0.0 && byte < 0.1 * self.media_byte;
        let durata_ns = self.formato.campioni(&dati) as i64 * 1_000_000_000 / FREQUENZA as i64;
        self.media_byte = if self.media_byte == 0.0 { byte } else { 0.98 * self.media_byte + 0.02 * byte };
        let mut buffer = gst::Buffer::from_mut_slice(dati);
        let adesso = self.pipeline.clock().zip(self.pipeline.base_time()).map(|(c, b)| c.time().saturating_sub(b));
        if let Some(ora) = adesso.map(|t| t.nseconds() as i64) {
            // Margine alto e Wi-Fi calmo: si salta un pacchetto di silenzio e
            // l'audio si riavvicina al video (§52).
            if silenzio && self.margine.scendi(ora, durata_ns) {
                crate::avsync::margine(self.margine.margine / 1_000_000);
                diagnosi(&format!("audio: margine sceso a {} ms", self.margine.margine / 1_000_000));
                return Ok(());
            }
            let riallineamenti = self.margine.riallineamenti;
            let prima = self.margine.margine;
            let orario = self.margine.orario(ora, pts_ns);
            if self.margine.margine != prima {
                crate::avsync::margine(self.margine.margine / 1_000_000);
            }
            if self.margine.riallineamenti != riallineamenti {
                diagnosi("audio: riallineamento degli orologi");
            }
            buffer.get_mut().unwrap().set_pts(gst::ClockTime::from_nseconds(orario.max(0) as u64));
            // Il video mostrerà il fotogramma di questo istante quando suona.
            let attesa = orario + self.latenza.load(std::sync::atomic::Ordering::Relaxed) as i64 - ora;
            crate::sincronia::annuncia(pts, Instant::now() + Duration::from_nanos(attesa.max(0) as u64));
            let (pacchetti, ritardi, quando) = &mut self.resoconto;
            *pacchetti += 1;
            if quando.elapsed() >= Duration::from_secs(5) {
                diagnosi(&format!(
                    "audio: {pacchetti} pacchetti, {} in ritardo, margine {} ms",
                    self.margine.ritardi - *ritardi,
                    self.margine.margine / 1_000_000
                ));
                (*pacchetti, *ritardi, *quando) = (0, self.margine.ritardi, Instant::now());
            }
        }
        if self.sorgente.push_buffer(buffer).is_err() {
            bail!("riproduzione audio fermata");
        }
        Ok(())
    }
}

impl Drop for Riproduzione {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

/// Un pacchetto per chi registra: orario regolare (µs, scala del telefono) e
/// dati (pacchetti Opus o PCM, vedi [`caps_registrazione`]).
pub type PacchettoAudio = (u64, Vec<u8>);

fn copie() -> &'static tokio::sync::broadcast::Sender<PacchettoAudio> {
    static COPIE: std::sync::OnceLock<tokio::sync::broadcast::Sender<PacchettoAudio>> = std::sync::OnceLock::new();
    COPIE.get_or_init(|| tokio::sync::broadcast::channel(256).0)
}

/// Formato dell'audio in corso (per le caps di chi registra).
static IN_CORSO: Mutex<Option<Formato>> = Mutex::new(None);

/// Il formato dell'audio del componente se sta suonando (`None` se l'audio
/// non è partito): serve alla registrazione.
pub fn formato_in_corso() -> Option<Formato> {
    *IN_CORSO.lock().unwrap()
}

/// I pacchetti audio che arriveranno da adesso in poi.
pub fn ascolta() -> tokio::sync::broadcast::Receiver<PacchettoAudio> {
    copie().subscribe()
}

/// Le caps per l'`appsrc` audio di una registrazione (`mp4mux` accetta
/// l'Opus così com'è); `None` se l'audio non è in corso.
pub fn caps_registrazione() -> Option<gst::Caps> {
    formato_in_corso().map(caps)
}

/// Durata (µs) di un pacchetto per chi registra (Opus: di solito 20 ms).
pub fn durata_pacchetto(formato: Formato, dati: &[u8]) -> u64 {
    formato.campioni(dati) * 1_000_000 / FREQUENZA
}

/// Riproduce l'audio del telefono dalle casse del PC finché il canale resta
/// aperto, col servizio già avviato. Opus; con `PHONESTRA_AUDIO_CODEC=pcm` (o `raw`) il PCM.
/// Quando la funzione finisce (o il compito viene annullato) il canale si
/// chiude e il telefono torna a suonare da sé.
pub async fn riproduci(servizio: &Apritore) -> Result<()> {
    let formato = match std::env::var("PHONESTRA_AUDIO_CODEC") {
        Ok(v) if v == "pcm" || v == "raw" => Formato::Pcm,
        _ => Formato::Opus,
    };
    let mut flusso = Flusso::apri(servizio, formato).await?;
    diagnosi(&format!("audio avviato: {}", flusso.inizio.testo));
    let esito = riproduci_flusso(&mut flusso).await;
    *IN_CORSO.lock().unwrap() = None;
    let _ = flusso.chiudi().await;
    esito
}

async fn riproduci_flusso(flusso: &mut Flusso) -> Result<()> {
    let formato = flusso.formato;
    // Né Opus (mappa dei canali 0) né PCM hanno bisogno della configurazione del codec.
    let mut riproduzione = Riproduzione::nuova(formato)?;
    *IN_CORSO.lock().unwrap() = Some(formato);
    loop {
        match flusso.prossimo().await? {
            None => bail!("il telefono ha chiuso il canale audio"),
            Some(Pacchetto::Testo(t)) => {
                if t.starts_with("errore") {
                    bail!("il telefono ha interrotto l'audio: {t}");
                }
                if t.starts_with("avviso") {
                    eprintln!("[audio] telefono: {t}");
                }
                if let Some(attivi) = t.strip_prefix("lettori attivi=") {
                    // «?»: il telefono non lo sa dire, si resta in sincrono.
                    crate::sincronia::lettori(attivi != "0");
                }
                diagnosi(&t);
            }
            Some(Pacchetto::Configurazione(c)) => diagnosi(&format!("configurazione del codec: {} byte", c.len())),
            Some(Pacchetto::Dati { pts, dati }) => {
                crate::avsync::arrivo('a', pts);
                let pts = riproduzione.regola(pts, &dati);
                if copie().receiver_count() > 0 {
                    let _ = copie().send((pts, dati.clone()));
                }
                riproduzione.spingi(pts, dati)?;
            }
        }
    }
}

/// Decodifica tutto l'audio ricevuto in PCM S16LE, 48 kHz, stereo (per
/// l'analisi delle prove con [`crate::misura_audio`]). Bloccante.
pub fn decodifica(formato: Formato, pacchetti: &[Vec<u8>]) -> Result<Vec<u8>> {
    if formato == Formato::Pcm {
        return Ok(pacchetti.concat());
    }
    let pipeline = gst::parse::launch(
        "appsrc name=sorgente format=time ! opusdec ! audioconvert ! audioresample \
         ! audio/x-raw,format=S16LE,layout=interleaved,rate=48000,channels=2 ! appsink name=uscita sync=false",
    )
    .context("pipeline di decodifica (manca opusdec?)")?
    .downcast::<gst::Pipeline>()
    .map_err(|_| anyhow!("pipeline di decodifica"))?;
    let sorgente = pipeline.by_name("sorgente").and_then(|s| s.downcast::<gst_app::AppSrc>().ok()).context("appsrc")?;
    let uscita = pipeline.by_name("uscita").and_then(|s| s.downcast::<gst_app::AppSink>().ok()).context("appsink")?;
    sorgente.set_caps(Some(&caps_opus()));
    pipeline.set_state(gst::State::Playing).context("la decodifica non parte")?;
    let mut pcm = Vec::new();
    let raccogli = |pcm: &mut Vec<u8>, attesa: gst::ClockTime| -> bool {
        match uscita.try_pull_sample(attesa) {
            Some(s) => {
                if let Some(m) = s.buffer().and_then(|b| b.map_readable().ok()) {
                    pcm.extend_from_slice(&m);
                }
                true
            }
            None => false,
        }
    };
    let mut durate = Durate::default();
    let mut orario = 0u64;
    for p in pacchetti {
        let mut b = gst::Buffer::from_slice(p.clone());
        let durata = durate.prossima(formato.campioni(p));
        {
            let b = b.get_mut().unwrap();
            b.set_pts(gst::ClockTime::from_useconds(orario));
            b.set_duration(gst::ClockTime::from_useconds(durata));
        }
        orario += durata;
        if sorgente.push_buffer(b).is_err() {
            let _ = pipeline.set_state(gst::State::Null);
            bail!("decodifica fermata");
        }
        while raccogli(&mut pcm, gst::ClockTime::ZERO) {}
    }
    let _ = sorgente.end_of_stream();
    while !uscita.is_eos() {
        if !raccogli(&mut pcm, gst::ClockTime::from_seconds(2)) {
            break;
        }
    }
    let _ = pipeline.set_state(gst::State::Null);
    Ok(pcm)
}

/// Politiche audio nel `dumpsys audio` (sezione «Audio policies» di
/// `AudioService`): quante in tutto (righe `AudioPolicyConfig`) e quante in
/// loop-back (`route flags=0x2`). Per le prove: durante la cattura devono
/// essere una in più di prima, dopo la chiusura come prima.
pub fn conta_politiche(dumpsys: &str) -> (usize, usize) {
    let politiche = dumpsys.lines().filter(|r| r.contains("AudioPolicyConfig")).count();
    let loopback = dumpsys.lines().filter(|r| r.trim_start().trim_start_matches('*').trim() == "route flags=0x2").count();
    (politiche, loopback)
}

#[cfg(test)]
mod prove {
    use super::*;

    fn pacchetto(testa: u64, dati: &[u8]) -> Vec<u8> {
        let mut v = testa.to_be_bytes().to_vec();
        v.extend_from_slice(&(dati.len() as u32).to_be_bytes());
        v.extend_from_slice(dati);
        v
    }

    #[test]
    fn pacchetti_a_pezzi() {
        let mut flusso = pacchetto(TESTO, b"inizio formato=opus frequenza=48000\n");
        flusso.extend(pacchetto(CONFIGURAZIONE, &[0x11, 0x90]));
        flusso.extend(pacchetto(21_333, &[1, 2, 3]));
        flusso.extend(pacchetto(0, &[]));
        let mut l = Lettore::default();
        let mut letti = Vec::new();
        for pezzo in flusso.chunks(5) {
            l.aggiungi(pezzo);
            while let Some(p) = l.prossimo().unwrap() {
                letti.push(p);
            }
        }
        assert_eq!(
            letti,
            vec![
                Pacchetto::Testo("inizio formato=opus frequenza=48000".into()),
                Pacchetto::Configurazione(vec![0x11, 0x90]),
                Pacchetto::Dati { pts: 21_333, dati: vec![1, 2, 3] },
                Pacchetto::Dati { pts: 0, dati: vec![] },
            ]
        );
        let mut rotto = Lettore::default();
        rotto.aggiungi(&[0, 0, 0, 0, 0, 0, 0, 0, 0x7f, 0xff, 0xff, 0xff]);
        assert!(rotto.prossimo().is_err());
    }

    #[test]
    fn formati_e_tipi_di_canale() {
        assert_eq!(Formato::Opus.tipo_canale(), "audio:opus");
        assert_eq!(Formato::Pcm.tipo_canale(), "audio:pcm");
        assert_eq!(Formato::da_nome("raw"), Some(Formato::Pcm));
        assert_eq!(Formato::da_nome("opus"), Some(Formato::Opus));
        assert_eq!(Formato::da_nome("aac"), None);
        // CELT 20 ms, un frame (TOC 0xfc: configurazione 31, codice 0).
        assert_eq!(Formato::Opus.campioni(&[0xfc, 1, 2]), 960);
        assert_eq!(Formato::Pcm.campioni(&[0; 4096]), 1024);
        assert_eq!(durata_pacchetto(Formato::Opus, &[0xfc, 1, 2]), 20_000);
    }

    #[test]
    fn durata_dei_pacchetti_opus() {
        // SILK 60 ms, un frame.
        assert_eq!(campioni_opus(&[3 << 3]), Some(2880));
        // Ibrido 10 ms, due frame (codice 1).
        assert_eq!(campioni_opus(&[(12 << 3) | 1]), Some(960));
        // CELT 2,5 ms, codice 3 con 4 frame.
        assert_eq!(campioni_opus(&[(16 << 3) | 3, 4]), Some(480));
        // Vuoto, o codice 3 senza il byte dei frame: durata predefinita.
        assert_eq!(campioni_opus(&[]), None);
        assert_eq!(campioni_opus(&[(31 << 3) | 3]), None);
        assert_eq!(Formato::Opus.campioni(&[]), CAMPIONI_OPUS);
    }

    #[test]
    fn durate_senza_errore_accumulato() {
        let mut d = Durate::default();
        let prime: Vec<u64> = (0..3).map(|_| d.prossima(1024)).collect();
        assert_eq!(prime, [21_333, 21_333, 21_334]);
        // Dopo un'ora di frame da 1024 campioni la somma è esattamente l'orario del telefono.
        let mut d = Durate::default();
        let frame = 3600 * FREQUENZA / 1024;
        let somma: u64 = (0..frame).map(|_| d.prossima(1024)).sum();
        assert_eq!(somma, frame * 1024 * 1_000_000 / FREQUENZA);
    }

    #[test]
    fn orari_regolari_salvo_scostamenti_grandi() {
        let mut o = Orari::default();
        let orari: Vec<u64> = [0, 22_000, 23_000, 50_000].iter().map(|&t| o.regola(t, 20_000)).collect();
        assert_eq!(orari, [0, 20_000, 40_000, 60_000]);
        assert_eq!(o.regola(1_080_000, 20_000), 1_080_000);
        assert_eq!(o.regola(1_101_000, 20_000), 1_100_000);
    }

    #[test]
    fn margine_che_scende() {
        let ms = 1_000_000i64;
        let mut m = Margine::default();
        m.orario(0, 0);
        // Un ritardo: margine a 120 ms.
        m.orario(1115 * ms, 1_000 * ms);
        assert_eq!(m.margine, 120 * ms);
        // Subito dopo non scende (Wi-Fi non ancora calmo).
        assert!(!m.scendi(2_000 * ms, 21 * ms));
        // Dopo 10 s di calma scende di un pacchetto, poi deve ricalmarsi.
        assert!(m.scendi(12_000 * ms, 21 * ms));
        assert_eq!(m.margine, 99 * ms);
        assert!(!m.scendi(13_000 * ms, 21 * ms));
        assert!(!m.scendi(23_000 * ms, 21 * ms), "sotto il minimo di 80 ms non scende");
        assert_eq!(m.discese, 1);
    }

    #[test]
    fn margine_di_riproduzione() {
        let ms = 1_000_000i64;
        let mut m = Margine::default();
        // Primo pacchetto: riprodotto 80 ms dopo l'arrivo.
        assert_eq!(m.orario(1000 * ms, 0), 1080 * ms);
        // In orario (arriva 20 ms dopo, segnato 20 ms dopo).
        assert_eq!(m.orario(1020 * ms, 20 * ms), 1100 * ms);
        // In ritardo: segnato 40 ms, arriva a 1115 ms (orario 1120 < 1125): tutto
        // si sposta di 1115 + 80 − 1120 = 75 ms e il margine sale a 120 ms.
        assert_eq!(m.orario(1115 * ms, 40 * ms), 1195 * ms);
        assert_eq!((m.ritardi, m.margine), (1, 120 * ms));
        // Telefono molto più avanti (orologi allontanati): riallineamento.
        assert_eq!(m.orario(1200 * ms, 2000 * ms), 1320 * ms);
        assert_eq!(m.riallineamenti, 1);
        // Il margine non supera 300 ms.
        for i in 0..20 {
            m.orario((5000 + i * 1000) * ms, 2020 * ms);
        }
        assert_eq!(m.margine, MARGINE_MASSIMO_NS);
    }

    #[test]
    fn ritardi_col_margine_al_massimo() {
        let ms = 1_000_000i64;
        let mut m = Margine { margine: MARGINE_MASSIMO_NS, ..Default::default() };
        assert_eq!(m.orario(1000 * ms, 0), 1300 * ms);
        // Il PC va più veloce: il pacchetto di 1000 ms arriva a 2295 ms invece
        // che a 2000 (orario 2300 < 2305). Si sposta tutto: suona 300 ms dopo.
        assert_eq!(m.orario(2295 * ms, 1000 * ms), 2595 * ms);
        assert_eq!(m.ritardi, 1);
        // Il seguente, alla stessa andatura, non è più in ritardo.
        assert_eq!(m.orario(2316 * ms, 1021 * ms), 2616 * ms);
        assert_eq!(m.ritardi, 1);
    }

    #[test]
    fn controllo_degli_orari_opus() {
        // Come il codificatore Opus del telefono: primo orario qualsiasi, poi +20 ms.
        let mut c = ControlloOrari::default();
        for k in 0..100u64 {
            c.controlla(232_806_486_617 + k * 20_000, 960);
        }
        assert_eq!((c.irregolari, c.mancanti), (0, 0));
        // Un pacchetto perso.
        c.controlla(232_806_486_617 + 101 * 20_000, 960);
        assert_eq!((c.irregolari, c.mancanti), (1, 960));
    }

    #[test]
    fn controllo_degli_orari() {
        let mut c = ControlloOrari::default();
        for k in 0..10u64 {
            c.controlla(k * 1024 * 1_000_000 / FREQUENZA, 1024);
        }
        assert_eq!((c.irregolari, c.mancanti), (0, 0));
        // Due frame persi: il 12 arriva dopo il 9.
        c.controlla(12 * 1024 * 1_000_000 / FREQUENZA, 1024);
        assert_eq!((c.irregolari, c.mancanti), (1, 2048));
        c.controlla(13 * 1024 * 1_000_000 / FREQUENZA, 1024);
        assert_eq!(c.irregolari, 1);
    }

    #[test]
    fn politiche_nel_dumpsys() {
        let prima = "Audio policies:\n\nMediaFocusControl dump time: 10:00\n";
        let durante = "Audio policies:\nandroid.media.audiopolicy.AudioPolicyConfig:\n1 AudioMix, reg:x:ap:0\n\
                       * route flags=0x2\n  rate=48000Hz\nUid: 2000\n\nMediaFocusControl dump time: 10:00\n";
        assert_eq!(conta_politiche(prima), (0, 0));
        assert_eq!(conta_politiche(durante), (1, 1));
    }

    /// Opus di prova codificato da GStreamer (`opusenc`), pacchetti di 20 ms come il telefono.
    fn opus_di_prova(secondi: u64) -> Option<Vec<Vec<u8>>> {
        gst::init().ok()?;
        gst::ElementFactory::find("opusenc")?;
        gst::ElementFactory::find("opusdec")?;
        let n = secondi * FREQUENZA / CAMPIONI_OPUS;
        let pipeline = gst::parse::launch(&format!(
            "audiotestsrc num-buffers={n} samplesperbuffer={CAMPIONI_OPUS} wave=sine freq=440 volume=0.3 \
             ! audio/x-raw,rate=48000,channels=2 ! audioconvert ! opusenc bitrate=128000 frame-size=20 \
             ! appsink name=uscita sync=false"
        ))
        .ok()?
        .downcast::<gst::Pipeline>()
        .ok()?;
        let uscita = pipeline.by_name("uscita")?.downcast::<gst_app::AppSink>().ok()?;
        pipeline.set_state(gst::State::Playing).ok()?;
        let mut pacchetti = Vec::new();
        while let Some(s) = uscita.try_pull_sample(gst::ClockTime::from_seconds(5)) {
            pacchetti.push(s.buffer()?.map_readable().ok()?.to_vec());
        }
        let _ = pipeline.set_state(gst::State::Null);
        Some(pacchetti)
    }

    #[test]
    fn decodifica_opus_e_riproduzione() {
        let Some(pacchetti) = opus_di_prova(2) else {
            eprintln!("opusenc/opusdec assenti: prova saltata");
            return;
        };
        assert!(pacchetti.iter().all(|p| campioni_opus(p) == Some(960)));
        let pcm = decodifica(Formato::Opus, &pacchetti).unwrap();
        let campioni = crate::misura_audio::campioni(&pcm);
        // Circa 2 s di audio stereo, con suono vero (non silenzio).
        assert!((campioni.len() as i64 - 2 * 96_000).abs() < 4 * 1024, "{} campioni", campioni.len());
        let a = crate::misura_audio::analizza(&campioni);
        assert!(a.zeri.is_empty() && a.tagli.is_empty());
        assert!(campioni.iter().map(|c| c.unsigned_abs()).max().unwrap() > 5000);

        // La pipeline di riproduzione accetta gli stessi pacchetti (uscita finta).
        let mut r = Riproduzione::con_uscita(Formato::Opus, "audioconvert ! fakesink sync=false").unwrap();
        let mut d = Durate::default();
        let mut pts = 0;
        for p in pacchetti.iter().take(20) {
            let orario = r.regola(pts, p);
            assert_eq!(orario, pts);
            r.spingi(orario, p.clone()).unwrap();
            pts += d.prossima(960);
        }
    }

    #[test]
    fn registrazione_mp4_con_opus() {
        let Some(pacchetti) = opus_di_prova(1) else {
            eprintln!("opusenc assente: prova saltata");
            return;
        };
        if gst::ElementFactory::find("mp4mux").is_none() {
            return;
        }
        let file = std::env::temp_dir().join(format!("phonestra-prova-opus-{}.mp4", std::process::id()));
        let pipeline = gst::parse::launch("appsrc name=audio format=time ! mp4mux ! filesink name=file")
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();
        pipeline.by_name("file").unwrap().set_property("location", file.to_string_lossy().to_string());
        let audio = pipeline.by_name("audio").unwrap().downcast::<gst_app::AppSrc>().unwrap();
        audio.set_caps(Some(&caps_opus()));
        pipeline.set_state(gst::State::Playing).unwrap();
        let mut orario = 0;
        for p in &pacchetti {
            let mut b = gst::Buffer::from_slice(p.clone());
            let durata = durata_pacchetto(Formato::Opus, p);
            b.get_mut().unwrap().set_pts(gst::ClockTime::from_useconds(orario));
            b.get_mut().unwrap().set_duration(gst::ClockTime::from_useconds(durata));
            orario += durata;
            audio.push_buffer(b).unwrap();
        }
        audio.end_of_stream().unwrap();
        let fine = pipeline.bus().unwrap().timed_pop_filtered(gst::ClockTime::from_seconds(5), &[gst::MessageType::Eos, gst::MessageType::Error]);
        let _ = pipeline.set_state(gst::State::Null);
        assert!(matches!(fine.map(|m| m.type_()), Some(gst::MessageType::Eos)));
        let dimensione = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
        let _ = std::fs::remove_file(&file);
        assert!(dimensione > 10_000, "file MP4 di {dimensione} byte");
    }
}
