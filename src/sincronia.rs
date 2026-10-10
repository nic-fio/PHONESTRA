// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Sincronia audio-video (notes/av-sync-tests.md §12). Dal 6 ottobre 2026
//! audio e fotogrammi hanno orari sullo stesso orologio, quello monotono del
//! telefono: l'audio annuncia a che ora del PC suonerà un certo orario del
//! telefono, e il video mostra ogni fotogramma alla stessa ora. Solo mentre
//! sul telefono un lettore suona sul canale dei media (silenzi compresi: il
//! telefono lo dice ogni ~250 ms); senza lettori o senza audio da più di un
//! secondo i fotogrammi si mostrano appena decodificati, così navigando o
//! scrivendo la risposta resta immediata.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gst::prelude::*;

/// Ultimo annuncio dell'audio: orario del telefono (µs), ora d'uscita sul PC,
/// quando è stato fatto.
static ANNUNCIO: Mutex<Option<(u64, Instant, Instant)>> = Mutex::new(None);

/// Vero se sul telefono un lettore suona sul canale dei media (e finché il
/// telefono non dice niente: un helper vecchio).
static LETTORI: AtomicBool = AtomicBool::new(true);

/// Il telefono dice se un lettore suona.
pub fn lettori(attivi: bool) {
    LETTORI.store(attivi, Ordering::Relaxed);
}

/// Oltre questo anticipo un orario è sbagliato (telefono cambiato, orologio
/// ripartito): il fotogramma si mostra subito.
const ANTICIPO_MASSIMO: Duration = Duration::from_millis(1500);

/// L'audio con orario `pts_us` (telefono) uscirà dalle casse all'ora `uscita`.
pub fn annuncia(pts_us: u64, uscita: Instant) {
    *ANNUNCIO.lock().unwrap() = Some((pts_us, uscita, Instant::now()));
}

/// Ora del PC a cui suona l'orario `pts_us` del telefono, se l'audio è attivo.
pub fn quando(pts_us: u64) -> Option<Instant> {
    if !LETTORI.load(Ordering::Relaxed) {
        return None;
    }
    let (pts, uscita, fatto) = (*ANNUNCIO.lock().unwrap())?;
    if fatto.elapsed() > Duration::from_secs(1) {
        return None;
    }
    let ora = if pts_us >= pts {
        uscita.checked_add(Duration::from_micros(pts_us - pts))?
    } else {
        uscita.checked_sub(Duration::from_micros(pts - pts_us))?
    };
    (ora.saturating_duration_since(Instant::now()) <= ANTICIPO_MASSIMO).then_some(ora)
}

/// Dà l'orario al fotogramma `buffer` (orario `pts_us` del telefono) per la
/// pipeline di `sorgente`: all'ora dell'audio corrispondente con lo schermo
/// sincronizzato, altrimenti subito con lo schermo libero.
pub fn orario_fotogramma(sorgente: &gst_app::AppSrc, schermo: &gst::Element, buffer: &mut gst::BufferRef, pts_us: u64) {
    let (Some(orologio), Some(base)) = (sorgente.clock(), sorgente.base_time()) else {
        return;
    };
    let ora = orologio.time().saturating_sub(base);
    let quando = quando(pts_us);
    let anticipo = quando.map_or(Duration::ZERO, |q| q.saturating_duration_since(Instant::now()));
    buffer.set_pts(ora + gst::ClockTime::from_nseconds(anticipo.as_nanos() as u64));
    let sincronizzato = quando.is_some();
    if schermo.property::<bool>("sync") != sincronizzato {
        schermo.set_property("sync", sincronizzato);
    }
}

/// Latenza dell'uscita di `pipeline` (ns), letta ogni 2 s fuori dal flusso dei dati.
pub fn latenza(pipeline: &gst::Pipeline) -> Arc<AtomicU64> {
    let latenza = Arc::new(AtomicU64::new(0));
    let (valore, debole) = (latenza.clone(), pipeline.downgrade());
    std::thread::spawn(move || {
        while let Some(pipeline) = debole.upgrade() {
            let mut domanda = gst::query::Latency::new();
            if pipeline.query(&mut domanda) {
                valore.store(domanda.result().1.nseconds(), Ordering::Relaxed);
            }
            drop(pipeline);
            std::thread::sleep(Duration::from_secs(2));
        }
    });
    latenza
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn orari_del_telefono_in_ore_del_pc() {
        let uscita = Instant::now() + Duration::from_millis(300);
        annuncia(10_000_000, uscita);
        assert_eq!(quando(10_040_000), Some(uscita + Duration::from_millis(40)));
        assert_eq!(quando(9_960_000), Some(uscita - Duration::from_millis(40)));
        // Troppo avanti: orario sbagliato, il fotogramma va mostrato subito.
        assert_eq!(quando(12_000_000), None);
        // Nessun lettore attivo: subito.
        lettori(false);
        assert_eq!(quando(10_040_000), None);
        lettori(true);
    }
}
