//! L'audio del telefono dalle casse del PC (SPECIFICHE §10): una sola cattura
//! per telefono, indipendente dalle finestre. Il componente cattura l'uscita
//! audio del telefono (che intanto non suona) e la manda in Opus; GStreamer la
//! riproduce col volume del PC.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use gst::prelude::*;

use crate::adb::Adb;
use crate::sessione::{Pacchetto, apri_canale, diagnosi, lancia, leggi_pacchetto, u32_be};

const OPUS: u32 = 0x6f70_7573;
const RAW: u32 = 0x0072_6177;

/// Un pacchetto Opus del telefono: orario del telefono (µs) e dati.
pub type PacchettoAudio = (u64, Vec<u8>);

/// Copia dei pacchetti audio per chi registra (una finestra che registra lo
/// schermo ci mette anche l'audio, SPECIFICHE §13).
fn copie() -> &'static tokio::sync::broadcast::Sender<PacchettoAudio> {
    static COPIE: std::sync::OnceLock<tokio::sync::broadcast::Sender<PacchettoAudio>> = std::sync::OnceLock::new();
    COPIE.get_or_init(|| tokio::sync::broadcast::channel(256).0)
}

/// I pacchetti audio che arriveranno da adesso in poi.
pub fn ascolta() -> tokio::sync::broadcast::Receiver<PacchettoAudio> {
    copie().subscribe()
}

/// Orari regolari per i pacchetti audio. Il telefono segna i pacchetti a
/// raffiche (a 1–3 ms e poi a 30–40 ms l'uno dall'altro, anche se ognuno dura
/// 20 ms): riprodotti a quegli orari si sovrappongono o lasciano buchi, e
/// l'audio ha micro-interruzioni (provato il 28 set, anche con VLC sulle
/// registrazioni; con gli orari rifatti regolari erano sparite). Ogni pacchetto
/// va quindi subito dopo il precedente; l'orario del telefono conta solo se se
/// ne discosta più di `SCOSTAMENTO_MASSIMO_US` (vera pausa, orologi allontanati).
#[derive(Default)]
struct Orari {
    prossimo: Option<u64>,
}

/// Oltre questo scostamento dall'orario del telefono ci si riallinea a lui.
const SCOSTAMENTO_MASSIMO_US: u64 = 60_000;

impl Orari {
    /// Orario regolare (µs, scala del telefono) di un pacchetto lungo
    /// `durata` µs e segnato a `pts`.
    fn regola(&mut self, pts: u64, durata: u64) -> u64 {
        let orario = match self.prossimo {
            Some(p) if p.abs_diff(pts) <= SCOSTAMENTO_MASSIMO_US => p,
            _ => pts,
        };
        self.prossimo = Some(orario + durata);
        orario
    }
}

/// Durata (µs) di un pacchetto Opus, dall'intestazione (RFC 6716 §3.1).
fn durata_opus(dati: &[u8]) -> Option<u64> {
    let toc = *dati.first()?;
    let configurazione = toc >> 3;
    // Durata di un frame in decimi di millisecondo.
    let frame: u64 = match configurazione {
        0..=11 => [100, 200, 400, 600][configurazione as usize % 4],
        12..=15 => [100, 200][configurazione as usize % 2],
        _ => [25, 50, 100, 200][configurazione as usize % 4],
    };
    let frame_nel_pacchetto = match toc & 3 {
        0 => 1,
        1 | 2 => 2,
        _ => u64::from(*dati.get(1)? & 0x3f),
    };
    Some(frame * frame_nel_pacchetto * 100)
}

/// Margine iniziale di riproduzione dell'audio: assorbe le irregolarità del Wi-Fi.
const MARGINE_NS: i64 = 80_000_000;
/// Limite del margine: oltre, l'audio sarebbe visibilmente in ritardo sul video.
const MARGINE_MASSIMO_NS: i64 = 300_000_000;

/// Riproduce l'audio del telefono finché il collegamento resta aperto o il
/// componente si ferma. Il telefono torna a suonare da sé quando finisce.
pub async fn riproduci(adb: &Adb) -> Result<()> {
    // Ogni pacchetto si riproduce all'orario del telefono più un margine
    // fisso (`MARGINE`): le raffiche del Wi-Fi vengono assorbite e il ritardo
    // resta costante; un pacchetto più in ritardo del margine si scarta.
    // Formato: Opus; PHONESTRA_AUDIO_CODEC=raw per l'audio non compresso
    // (prove: esclude il codificatore del telefono). Con PHONESTRA_AUDIO_FILE
    // l'audio non compresso si salva anche in quel file (s16le, 48 kHz, 2 canali).
    let grezzo = std::env::var("PHONESTRA_AUDIO_CODEC").is_ok_and(|v| v == "raw");
    let mut salva = match std::env::var_os("PHONESTRA_AUDIO_FILE") {
        Some(percorso) if grezzo => Some(std::fs::File::create(percorso).context("file dell'audio")?),
        _ => None,
    };
    let pipeline = gst::parse::launch(if grezzo {
        "appsrc name=sorgente is-live=true format=time max-bytes=262144 \
         caps=audio/x-raw,format=S16LE,layout=interleaved,rate=(int)48000,channels=(int)2 \
         ! audioconvert ! audioresample ! autoaudiosink"
    } else {
        "appsrc name=sorgente is-live=true format=time max-bytes=65536 \
         caps=audio/x-opus,channel-mapping-family=(int)0,rate=(int)48000,channels=(int)2 \
         ! opusdec plc=true ! audioconvert ! audioresample ! autoaudiosink"
    })
    .context("pipeline audio (manca gstreamer1.0-plugins-base?)")?
    .downcast::<gst::Pipeline>()
    .map_err(|_| anyhow::anyhow!("pipeline audio"))?;
    let sorgente = pipeline.by_name("sorgente").context("sorgente audio")?.downcast::<gst_app::AppSrc>().unwrap();

    // Sorgente: l'uscita intera del telefono. La cattura per singola app
    // («playback», provabile con PHONESTRA_AUDIO=playback) lasciava vuoti di
    // 50–120 ms, circa uno al secondo, nei reel di Facebook e in Chrome: Android
    // mandava in affanno i loro lettori (provato il 28 set: 59 vuoti contro 1).
    let sorgente_audio = std::env::var("PHONESTRA_AUDIO").unwrap_or_else(|_| "output".into());
    let (scid, mut server) =
        lancia(adb, &format!("video=false control=false audio=true audio_codec={} audio_source={sorgente_audio}", if grezzo { "raw" } else { "opus" }))
            .await?;
    let esito = async {
        let mut audio = apri_canale(adb, scid).await?;
        let intestazione = tokio::time::timeout(Duration::from_secs(10), audio.leggi_esatti(1 + 64 + 4))
            .await
            .context("il componente non ha mandato l'intestazione audio")??;
        let codec = u32_be(&intestazione[65..69]);
        if codec != if grezzo { RAW } else { OPUS } {
            // 0: audio disattivato dal telefono (es. Android troppo vecchio); 1: errore.
            bail!("il telefono non ha avviato l'audio (codice {codec})");
        }
        diagnosi("audio avviato");
        pipeline.set_state(gst::State::Playing).context("la riproduzione audio non parte")?;
        // Tempo della pipeline adesso (quello con cui il riproduttore confronta gli orari).
        let adesso = || -> Option<gst::ClockTime> { Some(pipeline.clock()?.time().saturating_sub(pipeline.base_time()?)) };
        // Differenza tra l'orologio del telefono e quello della pipeline.
        let mut scarto: Option<i64> = None;
        // Margine attuale: cresce quando i pacchetti arrivano in ritardo (il
        // Wi-Fi a volte trattiene tutto per 200–300 ms, coi video di Facebook).
        let mut margine = MARGINE_NS;
        let mut orari = Orari::default();
        let (mut pacchetti, mut ritardi, mut ultimo_resoconto) = (0u32, 0u32, std::time::Instant::now());
        loop {
            match leggi_pacchetto(&mut audio).await? {
                // L'intestazione Opus: le caps bastano al decodificatore.
                Pacchetto::Dati { config: true, .. } | Pacchetto::Dimensione { .. } => {}
                Pacchetto::Dati { dati, pts, .. } => {
                    // Orari regolari anche per chi registra.
                    let durata = if grezzo {
                        dati.len() as u64 / 4 * 1_000_000 / 48_000
                    } else {
                        durata_opus(&dati).unwrap_or(20_000)
                    };
                    let pts = orari.regola(pts, durata);
                    if let Some(file) = salva.as_mut() {
                        use std::io::Write;
                        file.write_all(&dati)?;
                    }
                    // Chi registra si aspetta Opus.
                    if !grezzo && copie().receiver_count() > 0 {
                        let _ = copie().send((pts, dati.clone()));
                    }
                    let pts_ns = pts as i64 * 1000;
                    let mut buffer = gst::Buffer::from_mut_slice(dati);
                    if let Some(ora) = adesso().map(|t| t.nseconds() as i64) {
                        pacchetti += 1;
                        let orario = scarto.map(|s| pts_ns + s);
                        match orario {
                            // Primo pacchetto, o telefono molto più avanti
                            // (orologi allontanati): si riallinea.
                            None => scarto = Some(ora + margine - pts_ns),
                            Some(o) if o > ora + margine + 200_000_000 => {
                                diagnosi("audio: riallineamento degli orologi");
                                scarto = Some(ora + margine - pts_ns);
                            }
                            // In ritardo: invece di scartarlo si sposta tutto
                            // più avanti (un attimo di silenzio, poi niente buchi).
                            Some(o) if o < ora + 10_000_000 => {
                                ritardi += 1;
                                let aumento = (ora + margine - o).min(MARGINE_MASSIMO_NS - margine).max(0);
                                if margine < MARGINE_MASSIMO_NS {
                                    margine = (margine + 40_000_000).min(MARGINE_MASSIMO_NS);
                                }
                                scarto = scarto.map(|s| s + aumento);
                            }
                            Some(_) => {}
                        }
                        let orario = pts_ns + scarto.unwrap();
                        buffer.get_mut().unwrap().set_pts(gst::ClockTime::from_nseconds(orario.max(0) as u64));
                        if ultimo_resoconto.elapsed() >= Duration::from_secs(5) {
                            diagnosi(&format!(
                                "audio: {pacchetti} pacchetti, {ritardi} in ritardo, margine {} ms",
                                margine / 1_000_000
                            ));
                            (pacchetti, ritardi, ultimo_resoconto) = (0, 0, std::time::Instant::now());
                        }
                    }
                    if sorgente.push_buffer(buffer).is_err() {
                        bail!("riproduzione audio fermata");
                    }
                }
            }
        }
    }
    .await;
    let _ = pipeline.set_state(gst::State::Null);
    // Messaggi del componente, utili se l'audio non è partito.
    if esito.is_err() {
        while let Ok(Some(blocco)) = tokio::time::timeout(Duration::from_millis(300), server.leggi()).await {
            for riga in String::from_utf8_lossy(&blocco).lines() {
                eprintln!("[telefono] {riga}");
            }
        }
    }
    let _ = server.chiudi().await;
    esito
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn durata_dei_pacchetti_opus() {
        // CELT 20 ms, un frame (quello che manda il telefono).
        assert_eq!(durata_opus(&[31 << 3]), Some(20_000));
        // SILK 10 ms, due frame.
        assert_eq!(durata_opus(&[1]), Some(20_000));
        // CELT 2,5 ms, codice 3 con 8 frame.
        assert_eq!(durata_opus(&[(16 << 3) | 3, 8]), Some(20_000));
        assert_eq!(durata_opus(&[]), None);
    }

    #[test]
    fn orari_regolari_salvo_scostamenti_grandi() {
        let mut o = Orari::default();
        // Raffica del telefono: 0, 22, 23, 50 ms → 0, 20, 40, 60 ms.
        let orari: Vec<u64> = [0, 22_000, 23_000, 50_000].iter().map(|&t| o.regola(t, 20_000)).collect();
        assert_eq!(orari, [0, 20_000, 40_000, 60_000]);
        // Pausa vera di un secondo: ci si riallinea al telefono.
        assert_eq!(o.regola(1_080_000, 20_000), 1_080_000);
        assert_eq!(o.regola(1_101_000, 20_000), 1_100_000);
    }
}
