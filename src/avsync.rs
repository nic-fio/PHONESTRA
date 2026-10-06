//! Misura della sincronia audio-video (`notes/av-sync-tests.md`), solo con
//! `PHONESTRA_AVSYNC=<file.csv>`: sonde sulle pipeline vere, che scrivono
//! per ogni fotogramma la luminosità media e per ogni blocco di audio
//! decodificato l'istante in cui uscirà dalle casse. Nessun effetto senza la
//! variabile.
//!
//! Righe del CSV (`ms` = millisecondi dall'avvio della misura, orologio
//! monotono del PC):
//! - `V,ms,luma`: fotogramma decodificato, luminosità media del piano Y (0–255);
//! - `A,ms,rms,inizio,campioni`: blocco audio, `ms` = istante di uscita del
//!   primo campione (orario del blocco + latenza dell'uscita), `inizio` =
//!   primo campione sopra la soglia (−1 se nessuno);
//! - `M,ms,margine`: margine audio in ms (§52), a ogni cambiamento.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use gst::prelude::*;

/// Soglia di un campione «sonoro» (audio in virgola mobile, fondo scala 1).
const SOGLIA: f32 = 0.05;

struct Misura {
    inizio: Instant,
    file: Mutex<BufWriter<File>>,
}

fn misura() -> Option<&'static Misura> {
    static MISURA: OnceLock<Option<Misura>> = OnceLock::new();
    MISURA
        .get_or_init(|| {
            let percorso = std::env::var_os("PHONESTRA_AVSYNC")?;
            let file = File::create(&percorso).map_err(|e| eprintln!("[avsync] {}: {e}", percorso.to_string_lossy())).ok()?;
            eprintln!("[avsync] misura in {}", percorso.to_string_lossy());
            Some(Misura { inizio: Instant::now(), file: Mutex::new(BufWriter::new(file)) })
        })
        .as_ref()
}

/// Vero se la misura è attiva.
pub fn attiva() -> bool {
    misura().is_some()
}

fn scrivi(riga: std::fmt::Arguments) {
    if let Some(m) = misura() {
        let mut file = m.file.lock().unwrap();
        let _ = file.write_fmt(riga);
        let _ = file.write_all(b"\n");
        let _ = file.flush();
    }
}

fn ms(istante: Instant) -> f64 {
    misura().map_or(0.0, |m| istante.saturating_duration_since(m.inizio).as_secs_f64() * 1000.0)
}

/// Il margine audio è cambiato.
pub fn margine(margine_ms: i64) {
    scrivi(format_args!("M,{:.1},{margine_ms}", ms(Instant::now())));
}

/// Sonda sul pad d'ingresso di `elemento` (video grezzo decodificato).
pub fn sonda_video(pipeline: &gst::Pipeline, elemento: &str) {
    if !attiva() {
        return;
    }
    let Some(pad) = pipeline.by_name(elemento).and_then(|e| e.static_pad("sink")) else {
        eprintln!("[avsync] {elemento} senza pad");
        return;
    };
    pad.add_probe(gst::PadProbeType::BUFFER, |pad, info| {
        let adesso = Instant::now();
        let dimensioni = pad.current_caps().and_then(|c| {
            let s = c.structure(0)?;
            Some((s.get::<i32>("width").ok()? as usize, s.get::<i32>("height").ok()? as usize))
        });
        if let (Some(buffer), Some((larghezza, altezza))) = (info.buffer(), dimensioni)
            && let Ok(mappa) = buffer.map_readable()
        {
            // Piano Y all'inizio del buffer (I420/NV12): un byte ogni 61.
            let dati = &mappa[..(larghezza * altezza).min(mappa.len())];
            let (somma, n) = dati.iter().step_by(61).fold((0u64, 0u64), |(s, n), &b| (s + b as u64, n + 1));
            scrivi(format_args!("V,{:.1},{:.1}", ms(adesso), somma as f64 / n.max(1) as f64));
        }
        gst::PadProbeReturn::Ok
    });
}

/// Sonda sul pad d'uscita di `elemento` (audio decodificato in F32).
pub fn sonda_audio(pipeline: &gst::Pipeline, elemento: &str) {
    if !attiva() {
        return;
    }
    let Some(pad) = pipeline.by_name(elemento).and_then(|e| e.static_pad("src")) else {
        eprintln!("[avsync] {elemento} senza pad");
        return;
    };
    // Latenza dell'uscita, letta ogni 2 s fuori dal flusso dei dati.
    let latenza = Arc::new(AtomicU64::new(0));
    {
        let (latenza, debole) = (latenza.clone(), pipeline.downgrade());
        std::thread::spawn(move || {
            while let Some(pipeline) = debole.upgrade() {
                let mut domanda = gst::query::Latency::new();
                if pipeline.query(&mut domanda) {
                    latenza.store(domanda.result().1.nseconds(), Ordering::Relaxed);
                }
                drop(pipeline);
                std::thread::sleep(Duration::from_secs(2));
            }
        });
    }
    let debole = pipeline.downgrade();
    pad.add_probe(gst::PadProbeType::BUFFER, move |pad, info| {
        let adesso = Instant::now();
        let Some(pipeline) = debole.upgrade() else { return gst::PadProbeReturn::Remove };
        let (Some(buffer), Some(orologio), Some(base)) = (info.buffer(), pipeline.clock(), pipeline.base_time()) else {
            return gst::PadProbeReturn::Ok;
        };
        let Some(pts) = buffer.pts() else { return gst::PadProbeReturn::Ok };
        let (canali, intercalato) = pad
            .current_caps()
            .and_then(|c| {
                let s = c.structure(0)?;
                Some((s.get::<i32>("channels").unwrap_or(2).max(1) as usize, s.get::<&str>("layout").unwrap_or("interleaved") == "interleaved"))
            })
            .unwrap_or((2, true));
        // Quando suonerà: orario del blocco + latenza, rispetto all'orologio
        // della pipeline letto adesso.
        let ora = orologio.time().nseconds() as i64 - base.nseconds() as i64;
        let attesa_ns = pts.nseconds() as i64 + latenza.load(Ordering::Relaxed) as i64 - ora;
        let uscita = ms(adesso) + attesa_ns as f64 / 1e6;
        if let Ok(mappa) = buffer.map_readable() {
            let campioni: Vec<f32> = mappa.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect();
            let per_canale = campioni.len() / canali;
            // Primo canale: un campione ogni `canali` se intercalato, la prima metà se a piani.
            let primo: Vec<f32> = if intercalato { campioni.iter().step_by(canali).copied().collect() } else { campioni[..per_canale].to_vec() };
            let rms = (primo.iter().map(|x| x * x).sum::<f32>() / primo.len().max(1) as f32).sqrt();
            let inizio = primo.iter().position(|x| x.abs() > SOGLIA).map_or(-1, |i| i as i64);
            scrivi(format_args!("A,{uscita:.1},{rms:.4},{inizio},{per_canale}"));
        }
        gst::PadProbeReturn::Ok
    });
}
