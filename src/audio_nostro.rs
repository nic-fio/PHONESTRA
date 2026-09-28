//! L'audio del telefono col componente nostro (memoria/componente.md,
//! «Audio»): il canale `audio` del servizio (`CanaleAudio.java`), cattura
//! loopback, AAC-LC 192 kbit/s (PCM come riserva), orari dal conteggio dei
//! campioni. Prenderà il posto di [`crate::audio`] (scrcpy, Opus): stessa
//! uscita (GStreamer, `autoaudiosink`), stesso trattamento degli orari e del
//! margine, copie dei pacchetti per chi registra.
//!
//! Non ancora collegato all'interfaccia: la funzione da chiamare al posto di
//! [`crate::audio::riproduci`] è [`riproduci`], che vuole il [`Componente`]
//! già avviato.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use gst::prelude::*;

use crate::adb::Canale;
use crate::componente::Componente;
use crate::misura_audio::Riga;

/// Frequenza e canali dell'audio del telefono.
pub const FREQUENZA: u64 = 48_000;
pub const CANALI: u32 = 2;
/// Campioni (per canale) in un frame AAC-LC.
pub const CAMPIONI_AAC: u64 = 1024;

/// Bandiere nell'orario dei pacchetti (`Audio.java`): configurazione del codec, testo.
const CONFIGURAZIONE: u64 = 1 << 62;
const TESTO: u64 = 1 << 61;
/// Un pacchetto più grande è un flusso rovinato (un frame AAC è ~600 byte, uno PCM 4 KB).
const MASSIMO: usize = 1 << 20;
/// Tempo massimo per la riga `inizio` dopo l'apertura del canale.
const ATTESA_INIZIO: Duration = Duration::from_secs(10);

/// Formato dell'audio sul canale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formato {
    /// AAC-LC 192 kbit/s, 48 kHz, stereo (la scelta, prove §42).
    Aac,
    /// PCM 16 bit little-endian, 48 kHz, stereo (riserva e misure).
    Pcm,
}

impl Formato {
    /// Tipo del canale da aprire (`CanaleAudio.formato`).
    pub fn tipo_canale(self) -> &'static str {
        match self {
            Formato::Aac => "audio:aac",
            Formato::Pcm => "audio:pcm",
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Formato::Aac => "aac",
            Formato::Pcm => "pcm",
        }
    }

    /// Dal nome (`aac`, `pcm`; `raw` come in [`crate::audio`]).
    pub fn da_nome(nome: &str) -> Option<Self> {
        match nome {
            "aac" => Some(Formato::Aac),
            "pcm" | "raw" => Some(Formato::Pcm),
            _ => None,
        }
    }

    /// Campioni (per canale) in un pacchetto di dati lungo `byte`.
    pub fn campioni(self, byte: usize) -> u64 {
        match self {
            Formato::Aac => CAMPIONI_AAC,
            Formato::Pcm => byte as u64 / (2 * u64::from(CANALI)),
        }
    }
}

/// Un pacchetto del canale audio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pacchetto {
    /// Configurazione del codec (AAC: AudioSpecificConfig), prima dei dati.
    Configurazione(Vec<u8>),
    /// Riga di testo del telefono: `inizio`, `lettura`, `misura`, `avviso`, `errore`.
    Testo(String),
    /// Audio: orario in µs dal conteggio dei campioni.
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
    pub async fn apri(componente: &Componente, formato: Formato) -> Result<Self> {
        let canale = componente.apri_canale(formato.tipo_canale()).await?;
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
/// l'errore che si accumulerebbe sommando 21 333 µs a ogni frame AAC.
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

/// Orari regolari (come `audio::Orari`): ogni pacchetto subito dopo il
/// precedente; l'orario del telefono conta solo se se ne discosta più di
/// [`SCOSTAMENTO_MASSIMO_US`] (pausa vera, pacchetti persi per strada). Col
/// componente nostro gli orari del telefono sono già regolari; resta come
/// difesa, identico a quello in uso con scrcpy.
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
/// (orologio del telefono → orologio della pipeline, più il margine). Stessa
/// regola di `audio::riproduci`: il primo pacchetto fissa lo scarto; un
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
}

impl Default for Margine {
    fn default() -> Self {
        Self { scarto: None, margine: MARGINE_NS, ritardi: 0, riallineamenti: 0 }
    }
}

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
                let aumento = (ora + self.margine - o).min(MARGINE_MASSIMO_NS - self.margine).max(0);
                if self.margine < MARGINE_MASSIMO_NS {
                    self.margine = (self.margine + 40_000_000).min(MARGINE_MASSIMO_NS);
                }
                self.scarto = self.scarto.map(|s| s + aumento);
            }
            Some(_) => {}
        }
        pts_ns + self.scarto.unwrap()
    }
}

/// Controllo degli orari del telefono (per le prove): l'orario di ogni
/// pacchetto deve essere campioni precedenti × 10⁶ / 48000. Uno scostamento
/// vuol dire pacchetti persi (coda piena sul telefono) o orari sbagliati.
#[derive(Debug, Default)]
pub struct ControlloOrari {
    campioni: Option<u64>,
    /// Pacchetti con l'orario diverso dall'atteso.
    pub irregolari: u64,
    /// Campioni mancanti (salti in avanti), in tutto.
    pub mancanti: u64,
}

impl ControlloOrari {
    /// Controlla un pacchetto di `campioni` campioni segnato a `pts`.
    pub fn controlla(&mut self, pts: u64, campioni: u64) {
        let atteso = self.campioni.map(|c| c * 1_000_000 / FREQUENZA);
        let attuali = match atteso {
            // Primo pacchetto: il suo orario fissa il punto di partenza.
            None => (pts * FREQUENZA).div_ceil(1_000_000),
            Some(a) if a == pts => self.campioni.unwrap(),
            Some(_) => {
                self.irregolari += 1;
                let da_orario = (pts * FREQUENZA).div_ceil(1_000_000);
                self.mancanti += da_orario.saturating_sub(self.campioni.unwrap());
                da_orario
            }
        };
        self.campioni = Some(attuali + campioni);
    }
}

/// Caps GStreamer dell'AAC grezzo con la sua configurazione (`codec_data`):
/// servono a `avdec_aac` e a `mp4mux`.
pub fn caps_aac(configurazione: &[u8]) -> gst::Caps {
    gst::Caps::builder("audio/mpeg")
        .field("mpegversion", 4i32)
        .field("stream-format", "raw")
        .field("rate", FREQUENZA as i32)
        .field("channels", CANALI as i32)
        .field("codec_data", gst::Buffer::from_slice(configurazione.to_vec()))
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

fn caps(formato: Formato, configurazione: &[u8]) -> gst::Caps {
    match formato {
        Formato::Aac => caps_aac(configurazione),
        Formato::Pcm => caps_pcm(),
    }
}

/// Intestazione ADTS di 7 byte (AAC-LC, 48 kHz, stereo) per un frame di
/// `lunghezza` byte: per salvare un `.aac` leggibile da ffprobe e lettori.
pub fn intestazione_adts(lunghezza: usize) -> [u8; 7] {
    let n = lunghezza + 7;
    [0xff, 0xf1, 0x4c, 0x80 | ((n >> 11) & 0x03) as u8, ((n >> 3) & 0xff) as u8, (((n & 0x07) << 5) | 0x1f) as u8, 0xfc]
}

/// Messaggi di diagnosi, solo con `PHONESTRA_DEBUG=1` (come `sessione::diagnosi`).
fn diagnosi(testo: &str) {
    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
        eprintln!("[audio] {testo}");
    }
}

/// La riproduzione dalle casse del PC: la stessa pipeline di [`crate::audio`]
/// (`audioconvert ! audioresample ! autoaudiosink`), con `avdec_aac` per l'AAC.
pub struct Riproduzione {
    pipeline: gst::Pipeline,
    sorgente: gst_app::AppSrc,
    orari: Orari,
    durate: Durate,
    formato: Formato,
    pub margine: Margine,
    resoconto: (u64, u64, Instant),
}

impl Riproduzione {
    /// Pipeline verso le casse; per l'AAC serve la configurazione del codec.
    pub fn nuova(formato: Formato, configurazione: &[u8]) -> Result<Self> {
        Self::con_uscita(formato, configurazione, "audioconvert ! audioresample ! autoaudiosink")
    }

    /// Come [`Riproduzione::nuova`] con un'altra uscita (per le prove).
    pub fn con_uscita(formato: Formato, configurazione: &[u8], uscita: &str) -> Result<Self> {
        let descrizione = match formato {
            Formato::Aac => format!("appsrc name=sorgente is-live=true format=time max-bytes=65536 ! avdec_aac ! {uscita}"),
            Formato::Pcm => format!("appsrc name=sorgente is-live=true format=time max-bytes=262144 ! {uscita}"),
        };
        let pipeline = gst::parse::launch(&descrizione)
            .context("pipeline audio (manca gstreamer1.0-libav per avdec_aac?)")?
            .downcast::<gst::Pipeline>()
            .map_err(|_| anyhow!("pipeline audio"))?;
        let sorgente = pipeline
            .by_name("sorgente")
            .and_then(|s| s.downcast::<gst_app::AppSrc>().ok())
            .context("sorgente audio")?;
        sorgente.set_caps(Some(&caps(formato, configurazione)));
        pipeline.set_state(gst::State::Playing).context("la riproduzione audio non parte")?;
        Ok(Self {
            pipeline,
            sorgente,
            orari: Orari::default(),
            durate: Durate::default(),
            formato,
            margine: Margine::default(),
            resoconto: (0, 0, Instant::now()),
        })
    }

    /// Orario regolare (µs, scala del telefono) di un pacchetto di dati: da
    /// usare anche per le copie a chi registra.
    pub fn regola(&mut self, pts: u64, dati: &[u8]) -> u64 {
        let durata = self.durate.prossima(self.formato.campioni(dati.len()));
        self.orari.regola(pts, durata)
    }

    /// Manda alla pipeline un pacchetto con l'orario già regolato.
    pub fn spingi(&mut self, pts: u64, dati: Vec<u8>) -> Result<()> {
        let pts_ns = pts as i64 * 1000;
        let mut buffer = gst::Buffer::from_mut_slice(dati);
        let adesso = self.pipeline.clock().zip(self.pipeline.base_time()).map(|(c, b)| c.time().saturating_sub(b));
        if let Some(ora) = adesso.map(|t| t.nseconds() as i64) {
            let riallineamenti = self.margine.riallineamenti;
            let orario = self.margine.orario(ora, pts_ns);
            if self.margine.riallineamenti != riallineamenti {
                diagnosi("audio: riallineamento degli orologi");
            }
            buffer.get_mut().unwrap().set_pts(gst::ClockTime::from_nseconds(orario.max(0) as u64));
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
/// dati (AAC grezzo o PCM, vedi [`caps_registrazione`]).
pub type PacchettoAudio = (u64, Vec<u8>);

fn copie() -> &'static tokio::sync::broadcast::Sender<PacchettoAudio> {
    static COPIE: std::sync::OnceLock<tokio::sync::broadcast::Sender<PacchettoAudio>> = std::sync::OnceLock::new();
    COPIE.get_or_init(|| tokio::sync::broadcast::channel(256).0)
}

/// Formato e configurazione dell'audio in corso (per le caps di chi registra).
static IN_CORSO: Mutex<Option<(Formato, Vec<u8>)>> = Mutex::new(None);

/// I pacchetti audio che arriveranno da adesso in poi (come `audio::ascolta`).
pub fn ascolta() -> tokio::sync::broadcast::Receiver<PacchettoAudio> {
    copie().subscribe()
}

/// Le caps per l'`appsrc` audio di una registrazione (`mp4mux` accetta l'AAC
/// grezzo con `codec_data`); `None` se l'audio non è in corso.
pub fn caps_registrazione() -> Option<gst::Caps> {
    let in_corso = IN_CORSO.lock().unwrap();
    let (formato, configurazione) = in_corso.as_ref()?;
    Some(caps(*formato, configurazione))
}

/// Durata (µs) di un pacchetto per chi registra (AAC: 1024 campioni, 21,333 ms).
pub fn durata_pacchetto(formato: Formato, byte: usize) -> u64 {
    formato.campioni(byte) * 1_000_000 / FREQUENZA
}

/// Riproduce l'audio del telefono dalle casse del PC finché il canale resta
/// aperto: da chiamare al posto di [`crate::audio::riproduci`], col servizio
/// già avviato. AAC; con `PHONESTRA_AUDIO_CODEC=pcm` (o `raw`) il PCM.
/// Quando la funzione finisce (o il compito viene annullato) il canale si
/// chiude e il telefono torna a suonare da sé.
pub async fn riproduci(componente: &Componente) -> Result<()> {
    let formato = match std::env::var("PHONESTRA_AUDIO_CODEC") {
        Ok(v) if v == "pcm" || v == "raw" => Formato::Pcm,
        _ => Formato::Aac,
    };
    let mut flusso = Flusso::apri(componente, formato).await?;
    diagnosi(&format!("audio avviato: {}", flusso.inizio.testo));
    let esito = riproduci_flusso(&mut flusso).await;
    *IN_CORSO.lock().unwrap() = None;
    let _ = flusso.chiudi().await;
    esito
}

async fn riproduci_flusso(flusso: &mut Flusso) -> Result<()> {
    let formato = flusso.formato;
    let mut riproduzione = match formato {
        Formato::Pcm => Some(Riproduzione::nuova(formato, &[])?),
        Formato::Aac => None,
    };
    if formato == Formato::Pcm {
        *IN_CORSO.lock().unwrap() = Some((formato, Vec::new()));
    }
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
                diagnosi(&t);
            }
            Some(Pacchetto::Configurazione(c)) => {
                if formato == Formato::Aac {
                    *IN_CORSO.lock().unwrap() = Some((formato, c.clone()));
                    riproduzione = Some(Riproduzione::nuova(formato, &c)?);
                }
            }
            Some(Pacchetto::Dati { pts, dati }) => {
                let Some(r) = riproduzione.as_mut() else {
                    bail!("audio AAC senza configurazione del codec");
                };
                let pts = r.regola(pts, &dati);
                if copie().receiver_count() > 0 {
                    let _ = copie().send((pts, dati.clone()));
                }
                r.spingi(pts, dati)?;
            }
        }
    }
}

/// Decodifica tutto l'audio ricevuto in PCM S16LE, 48 kHz, stereo (per
/// l'analisi delle prove con [`crate::misura_audio`]). Bloccante.
pub fn decodifica(formato: Formato, configurazione: &[u8], pacchetti: &[Vec<u8>]) -> Result<Vec<u8>> {
    if formato == Formato::Pcm {
        return Ok(pacchetti.concat());
    }
    let pipeline = gst::parse::launch(
        "appsrc name=sorgente format=time ! avdec_aac ! audioconvert ! audioresample \
         ! audio/x-raw,format=S16LE,layout=interleaved,rate=48000,channels=2 ! appsink name=uscita sync=false",
    )
    .context("pipeline di decodifica (manca avdec_aac?)")?
    .downcast::<gst::Pipeline>()
    .map_err(|_| anyhow!("pipeline di decodifica"))?;
    let sorgente = pipeline.by_name("sorgente").and_then(|s| s.downcast::<gst_app::AppSrc>().ok()).context("appsrc")?;
    let uscita = pipeline.by_name("uscita").and_then(|s| s.downcast::<gst_app::AppSink>().ok()).context("appsink")?;
    sorgente.set_caps(Some(&caps_aac(configurazione)));
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
        let durata = durate.prossima(CAMPIONI_AAC);
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
        let mut flusso = pacchetto(TESTO, b"inizio formato=aac frequenza=48000\n");
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
                Pacchetto::Testo("inizio formato=aac frequenza=48000".into()),
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
        assert_eq!(Formato::Aac.tipo_canale(), "audio:aac");
        assert_eq!(Formato::Pcm.tipo_canale(), "audio:pcm");
        assert_eq!(Formato::da_nome("raw"), Some(Formato::Pcm));
        assert_eq!(Formato::da_nome("opus"), None);
        assert_eq!(Formato::Aac.campioni(600), 1024);
        assert_eq!(Formato::Pcm.campioni(4096), 1024);
        assert_eq!(durata_pacchetto(Formato::Aac, 600), 21_333);
    }

    #[test]
    fn durate_senza_errore_accumulato() {
        let mut d = Durate::default();
        let prime: Vec<u64> = (0..3).map(|_| d.prossima(1024)).collect();
        assert_eq!(prime, [21_333, 21_333, 21_334]);
        // Dopo un'ora di frame AAC la somma è esattamente l'orario del telefono.
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
    fn margine_come_con_scrcpy() {
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
    fn adts() {
        let h = intestazione_adts(593);
        // Lunghezza del frame (13 bit, intestazione compresa) = 600.
        let n = ((h[3] as usize & 3) << 11) | ((h[4] as usize) << 3) | (h[5] as usize >> 5);
        assert_eq!(n, 600);
        assert_eq!(&h[..3], &[0xff, 0xf1, 0x4c]);
    }

    #[test]
    fn politiche_nel_dumpsys() {
        let prima = "Audio policies:\n\nMediaFocusControl dump time: 10:00\n";
        let durante = "Audio policies:\nandroid.media.audiopolicy.AudioPolicyConfig:\n1 AudioMix, reg:x:ap:0\n\
                       * route flags=0x2\n  rate=48000Hz\nUid: 2000\n\nMediaFocusControl dump time: 10:00\n";
        assert_eq!(conta_politiche(prima), (0, 0));
        assert_eq!(conta_politiche(durante), (1, 1));
    }

    /// AAC di prova codificato da GStreamer (`avenc_aac`), se c'è: frame grezzi e configurazione.
    fn aac_di_prova(secondi: u64) -> Option<(Vec<u8>, Vec<Vec<u8>>)> {
        gst::init().ok()?;
        gst::ElementFactory::find("avenc_aac")?;
        gst::ElementFactory::find("avdec_aac")?;
        let n = secondi * FREQUENZA / 1024;
        let pipeline = gst::parse::launch(&format!(
            "audiotestsrc num-buffers={n} samplesperbuffer=1024 wave=sine freq=440 volume=0.3 \
             ! audio/x-raw,rate=48000,channels=2 ! audioconvert ! avenc_aac bitrate=192000 \
             ! audio/mpeg,stream-format=raw ! appsink name=uscita sync=false"
        ))
        .ok()?
        .downcast::<gst::Pipeline>()
        .ok()?;
        let uscita = pipeline.by_name("uscita")?.downcast::<gst_app::AppSink>().ok()?;
        pipeline.set_state(gst::State::Playing).ok()?;
        let (mut config, mut frame) = (None, Vec::new());
        while let Some(s) = uscita.try_pull_sample(gst::ClockTime::from_seconds(5)) {
            if config.is_none() {
                config = s.caps()?.structure(0)?.get::<gst::Buffer>("codec_data").ok()?.map_readable().ok().map(|m| m.to_vec());
            }
            frame.push(s.buffer()?.map_readable().ok()?.to_vec());
        }
        let _ = pipeline.set_state(gst::State::Null);
        Some((config?, frame))
    }

    #[test]
    fn decodifica_aac_e_riproduzione() {
        let Some((config, frame)) = aac_di_prova(2) else {
            eprintln!("avenc_aac/avdec_aac assenti: prova saltata");
            return;
        };
        // AAC-LC, 48 kHz, stereo come il telefono (ffmpeg aggiunge un'estensione).
        assert_eq!(&config[..2], &[0x11, 0x90]);
        let pcm = decodifica(Formato::Aac, &config, &frame).unwrap();
        let campioni = crate::misura_audio::campioni(&pcm);
        // Circa 2 s di audio stereo, con suono vero (non silenzio).
        assert!((campioni.len() as i64 - 2 * 96_000).abs() < 4 * 1024, "{} campioni", campioni.len());
        let a = crate::misura_audio::analizza(&campioni);
        assert!(a.zeri.is_empty() && a.tagli.is_empty());
        assert!(campioni.iter().map(|c| c.unsigned_abs()).max().unwrap() > 5000);

        // La pipeline di riproduzione accetta gli stessi frame (uscita finta).
        let mut r = Riproduzione::con_uscita(Formato::Aac, &config, "audioconvert ! fakesink sync=false").unwrap();
        let mut d = Durate::default();
        let mut pts = 0;
        for f in frame.iter().take(20) {
            let orario = r.regola(pts, f);
            assert_eq!(orario, pts);
            r.spingi(orario, f.clone()).unwrap();
            pts += d.prossima(1024);
        }
    }

    #[test]
    fn registrazione_mp4_con_aac() {
        let Some((config, frame)) = aac_di_prova(1) else {
            eprintln!("avenc_aac assente: prova saltata");
            return;
        };
        if gst::ElementFactory::find("mp4mux").is_none() {
            return;
        }
        let file = std::env::temp_dir().join(format!("phonestra-prova-aac-{}.mp4", std::process::id()));
        let pipeline = gst::parse::launch("appsrc name=audio format=time ! mp4mux ! filesink name=file")
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();
        pipeline.by_name("file").unwrap().set_property("location", file.to_string_lossy().to_string());
        let audio = pipeline.by_name("audio").unwrap().downcast::<gst_app::AppSrc>().unwrap();
        audio.set_caps(Some(&caps_aac(&config)));
        pipeline.set_state(gst::State::Playing).unwrap();
        let mut d = Durate::default();
        let mut orario = 0;
        for f in &frame {
            let mut b = gst::Buffer::from_slice(f.clone());
            let durata = d.prossima(1024);
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
