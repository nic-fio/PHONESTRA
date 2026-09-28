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
    let pipeline = gst::parse::launch(
        "appsrc name=sorgente is-live=true format=time max-bytes=65536 \
         caps=audio/x-opus,channel-mapping-family=(int)0,rate=(int)48000,channels=(int)2 \
         ! opusdec plc=true ! audioconvert ! audioresample ! autoaudiosink",
    )
    .context("pipeline audio (manca gstreamer1.0-plugins-base?)")?
    .downcast::<gst::Pipeline>()
    .map_err(|_| anyhow::anyhow!("pipeline audio"))?;
    let sorgente = pipeline.by_name("sorgente").context("sorgente audio")?.downcast::<gst_app::AppSrc>().unwrap();

    // Sorgente: PHONESTRA_AUDIO=output per catturare l'uscita intera (prove).
    let sorgente_audio = std::env::var("PHONESTRA_AUDIO").unwrap_or_else(|_| "playback".into());
    let (scid, mut server) =
        lancia(adb, &format!("video=false control=false audio=true audio_codec=opus audio_source={sorgente_audio}")).await?;
    let esito = async {
        let mut audio = apri_canale(adb, scid).await?;
        let intestazione = tokio::time::timeout(Duration::from_secs(10), audio.leggi_esatti(1 + 64 + 4))
            .await
            .context("il componente non ha mandato l'intestazione audio")??;
        let codec = u32_be(&intestazione[65..69]);
        if codec != OPUS {
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
        let (mut pacchetti, mut ritardi, mut ultimo_resoconto) = (0u32, 0u32, std::time::Instant::now());
        loop {
            match leggi_pacchetto(&mut audio).await? {
                // L'intestazione Opus: le caps bastano al decodificatore.
                Pacchetto::Dati { config: true, .. } | Pacchetto::Dimensione { .. } => {}
                Pacchetto::Dati { dati, pts, .. } => {
                    if copie().receiver_count() > 0 {
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
