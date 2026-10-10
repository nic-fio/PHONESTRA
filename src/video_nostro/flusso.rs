// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Il flusso video di una sessione: opzioni dello schermo e pacchetti come li
//! scrive il componente sul telefono (`SessioneVideo.java`).
//!
//! Sul canale video: pacchetti con intestazione di 12 byte big-endian;
//! misura (bit 63) = larghezza e altezza; dati = orario/bandiere (u64) +
//! lunghezza (u32), seguiti dai dati del codec.

use anyhow::Result;

use crate::adb::Canale;

const FLAG_SESSIONE: u64 = 1 << 63;
const FLAG_CONFIG: u64 = 1 << 62;
const FLAG_CHIAVE: u64 = 1 << 61;

/// Nome del codec dal suo id (vedi [`super::id_codec`]).
pub fn nome_codec(id: u32) -> &'static str {
    match id {
        0x6832_3634 => "h264",
        0x6832_3635 => "h265",
        0x0061_7631 => "av1",
        _ => "sconosciuto",
    }
}

/// Come aprire lo schermo di una sessione.
pub struct Opzioni {
    /// Dimensione del display virtuale: larghezza, altezza, densità.
    pub display: (u32, u32, u32),
    /// Display ridimensionabile con [`super::ComandiVideo::ridimensiona`].
    pub ridimensionabile: bool,
    pub codec: &'static str,
    /// Schermo principale del telefono invece di un display virtuale.
    pub specchio: bool,
}

impl Default for Opzioni {
    fn default() -> Self {
        Self { display: (720, 1280, 320), ridimensionabile: true, codec: "h264", specchio: false }
    }
}

/// Un pacchetto del flusso video.
pub enum Pacchetto {
    /// Nuova sessione di cattura (inizio o cambio di dimensione).
    Dimensione { larghezza: u32, altezza: u32 },
    /// Dati del codec: parametri (`config`) o un fotogramma.
    Dati { pts: u64, config: bool, chiave: bool, dati: Vec<u8> },
}

fn u32_be(b: &[u8]) -> u32 {
    u32::from_be_bytes(b[0..4].try_into().unwrap())
}

/// Legge il prossimo pacchetto video. Non va interrotta a metà (es. dentro
/// `select!`): i byte già letti andrebbero persi. Usarla in un compito dedicato.
pub async fn leggi_pacchetto(video: &mut Canale) -> Result<Pacchetto> {
    let h = video.leggi_esatti(12).await?;
    match intestazione(&h) {
        Intestazione::Dimensione(p) => Ok(p),
        Intestazione::Dati { pts, config, chiave, lunghezza } => {
            let dati = video.leggi_esatti(lunghezza).await?;
            Ok(Pacchetto::Dati { pts, config, chiave, dati })
        }
    }
}

/// L'intestazione di 12 byte di un pacchetto video.
pub enum Intestazione {
    /// Pacchetto completo: nuova misura.
    Dimensione(Pacchetto),
    /// Seguono `lunghezza` byte di dati.
    Dati { pts: u64, config: bool, chiave: bool, lunghezza: usize },
}

pub fn intestazione(h: &[u8]) -> Intestazione {
    let testa = u64::from_be_bytes(h[0..8].try_into().unwrap());
    if testa & FLAG_SESSIONE != 0 {
        return Intestazione::Dimensione(Pacchetto::Dimensione { larghezza: u32_be(&h[4..8]), altezza: u32_be(&h[8..12]) });
    }
    Intestazione::Dati {
        pts: testa & !(FLAG_CONFIG | FLAG_CHIAVE),
        config: testa & FLAG_CONFIG != 0,
        chiave: testa & FLAG_CHIAVE != 0,
        lunghezza: u32_be(&h[8..12]) as usize,
    }
}
