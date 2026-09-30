//! Strumento di misura dell'audio del telefono (fase 0 del componente nostro,
//! `notes/study/audio.md`, prove A2, A4, A5, A6): le righe di misura che
//! l'aiutante manda ogni secondo e l'analisi, sul PC, del PCM ricevuto.
//!
//! Il PCM è 16 bit, 48 kHz, stereo, campioni alternati sinistro/destro. I
//! livelli sono in dBFS come RMS riferito a 32768 (un'onda quadra a fondo
//! scala vale 0 dB).

/// Frequenza del PCM dell'aiutante.
pub const FREQUENZA: usize = 48_000;
/// Campioni (per canale) in 1 ms.
const MS: usize = FREQUENZA / 1000;

/// Sequenze di zeri esatti: almeno 1 ms…
const ZERI_MINIMI: usize = 48;
/// …dopo 10 ms con un livello sopra −40 dBFS.
const PRIMA_DEGLI_ZERI: usize = 480;
const SOGLIA_ZERI_DB: f64 = -40.0;

/// Tagli netti: da sopra −30 dB a sotto −50 dB in 2 ms, e sotto per ≥ 30 ms.
const TAGLIO_PRIMA_DB: f64 = -30.0;
const TAGLIO_DOPO_DB: f64 = -50.0;
const TAGLIO_DURATA_MS: usize = 30;

/// Una riga di testo dell'aiutante: `tipo chiave=valore …` (`inizio`,
/// `lettura`, `misura`, `avviso`, `errore`).
#[derive(Debug, Clone, PartialEq)]
pub struct Riga {
    pub tipo: String,
    pub valori: Vec<(String, String)>,
    /// La riga intera, per stamparla.
    pub testo: String,
}

impl Riga {
    pub fn leggi(testo: &str) -> Self {
        let testo = testo.trim().to_string();
        let mut parole = testo.split_whitespace();
        let tipo = parole.next().unwrap_or("").to_string();
        let valori = parole
            .filter_map(|p| p.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Riga { tipo, valori, testo }
    }

    pub fn valore(&self, chiave: &str) -> Option<&str> {
        self.valori.iter().find(|(k, _)| k == chiave).map(|(_, v)| v.as_str())
    }

    pub fn numero(&self, chiave: &str) -> Option<f64> {
        self.valore(chiave)?.parse().ok()
    }
}

/// Un tratto del PCM: inizio e durata in campioni (per canale).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tratto {
    pub inizio: usize,
    pub durata: usize,
}

impl Tratto {
    pub fn secondi(&self) -> f64 {
        self.inizio as f64 / FREQUENZA as f64
    }

    pub fn millisecondi(&self) -> f64 {
        self.durata as f64 * 1000.0 / FREQUENZA as f64
    }
}

/// Quello che l'analisi del PCM trova.
#[derive(Debug, Default)]
pub struct Analisi {
    /// Sequenze ≥ 1 ms di zeri esatti su entrambi i canali, dopo suono sopra
    /// −40 dBFS e seguite da suono (lo stesso criterio dell'aiutante).
    pub zeri: Vec<Tratto>,
    /// Cadute da sopra −30 dB a sotto −50 dB in 2 ms che restano sotto −50 dB
    /// per almeno 30 ms (livello a finestre di 1 ms).
    pub tagli: Vec<Tratto>,
    /// Durata analizzata, in campioni per canale.
    pub campioni: usize,
}

/// Campioni da byte little-endian (come li legge `AudioRecord`).
pub fn campioni(byte: &[u8]) -> Vec<i16> {
    byte.as_chunks::<2>().0.iter().map(|&c| i16::from_le_bytes(c)).collect()
}

/// Livello in dBFS di un tratto stereo alternato.
fn livello(stereo: &[i16]) -> f64 {
    if stereo.is_empty() {
        return f64::NEG_INFINITY;
    }
    let energia: f64 = stereo.iter().map(|&c| c as f64 * c as f64).sum::<f64>() / stereo.len() as f64;
    10.0 * (energia.max(1e-12) / (32768.0 * 32768.0)).log10()
}

/// Analizza il PCM stereo alternato.
pub fn analizza(stereo: &[i16]) -> Analisi {
    let n = stereo.len() / 2;
    Analisi { zeri: zeri(stereo, n), tagli: tagli(stereo, n), campioni: n }
}

fn zeri(stereo: &[i16], n: usize) -> Vec<Tratto> {
    let mut trovati = Vec::new();
    let mut inizio = None;
    for i in 0..n {
        let zero = stereo[2 * i] == 0 && stereo[2 * i + 1] == 0;
        match (zero, inizio) {
            (true, None) => inizio = Some(i),
            (false, Some(s)) => {
                inizio = None;
                let durata = i - s;
                let prima = s.saturating_sub(PRIMA_DEGLI_ZERI);
                if durata >= ZERI_MINIMI && s > prima && livello(&stereo[2 * prima..2 * s]) > SOGLIA_ZERI_DB {
                    trovati.push(Tratto { inizio: s, durata });
                }
            }
            _ => {}
        }
    }
    trovati
}

fn tagli(stereo: &[i16], n: usize) -> Vec<Tratto> {
    let livelli: Vec<f64> = (0..n / MS).map(|f| livello(&stereo[2 * f * MS..2 * (f + 1) * MS])).collect();
    let mut trovati = Vec::new();
    let mut f = 2;
    while f < livelli.len() {
        let prima = livelli[f - 2].max(livelli[f - 1]);
        if prima > TAGLIO_PRIMA_DB && livelli[f] < TAGLIO_DOPO_DB {
            let fine = (f..livelli.len()).find(|&g| livelli[g] >= TAGLIO_DOPO_DB).unwrap_or(livelli.len());
            if fine - f >= TAGLIO_DURATA_MS && fine < livelli.len() {
                trovati.push(Tratto { inizio: f * MS, durata: (fine - f) * MS });
            }
            f = fine.max(f + 1);
            continue;
        }
        f += 1;
    }
    trovati
}

/// Intestazione WAV (PCM 16 bit, 48 kHz, stereo) per `byte_dati` byte di campioni.
pub fn intestazione_wav(byte_dati: u32) -> [u8; 44] {
    let mut h = [0u8; 44];
    let canali = 2u16;
    let byte_al_secondo = FREQUENZA as u32 * 4;
    h[0..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&(36 + byte_dati).to_le_bytes());
    h[8..16].copy_from_slice(b"WAVEfmt ");
    h[16..20].copy_from_slice(&16u32.to_le_bytes());
    h[20..22].copy_from_slice(&1u16.to_le_bytes());
    h[22..24].copy_from_slice(&canali.to_le_bytes());
    h[24..28].copy_from_slice(&(FREQUENZA as u32).to_le_bytes());
    h[28..32].copy_from_slice(&byte_al_secondo.to_le_bytes());
    h[32..34].copy_from_slice(&4u16.to_le_bytes());
    h[34..36].copy_from_slice(&16u16.to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&byte_dati.to_le_bytes());
    h
}

#[cfg(test)]
mod prove {
    use super::*;

    /// Onda quadra stereo di ampiezza `a` (−20 dB con a = 3277) per `ms` millisecondi.
    fn suono(a: i16, ms: usize) -> Vec<i16> {
        (0..ms * MS).flat_map(|i| if i % 48 < 24 { [a, a] } else { [-a, -a] }).collect()
    }

    fn silenzio(ms: usize) -> Vec<i16> {
        vec![0; ms * MS * 2]
    }

    #[test]
    fn riga_di_misura() {
        let r = Riga::leggi("misura t=3 campioni=144384 zeri=2 zeri_ms=4.50 deriva_ms=-0.12 nice=-19\n");
        assert_eq!(r.tipo, "misura");
        assert_eq!(r.numero("campioni"), Some(144_384.0));
        assert_eq!(r.numero("zeri_ms"), Some(4.5));
        assert_eq!(r.numero("deriva_ms"), Some(-0.12));
        assert_eq!(r.valore("nice"), Some("-19"));
        assert_eq!(r.valore("assente"), None);
        let e = Riga::leggi("errore java.io.IOException: cattura dell'audio non disponibile");
        assert_eq!(e.tipo, "errore");
        assert!(e.testo.contains("non disponibile"));
    }

    #[test]
    fn zeri_in_mezzo_al_suono() {
        let mut pcm = suono(3277, 50);
        pcm.extend(silenzio(5));
        pcm.extend(suono(3277, 50));
        // 0,5 ms di zeri: troppo pochi.
        pcm.extend(vec![0; 24 * 2]);
        pcm.extend(suono(3277, 50));
        let a = analizza(&pcm);
        assert_eq!(a.zeri, vec![Tratto { inizio: 50 * MS, durata: 5 * MS }]);
        assert!((a.zeri[0].millisecondi() - 5.0).abs() < 1e-9);
        // 5 ms di zeri non sono un taglio (ne servono 30).
        assert!(a.tagli.is_empty());
    }

    #[test]
    fn zeri_dopo_suono_debole_o_in_fondo_non_contano() {
        let mut pcm = suono(100, 50); // circa −50 dB
        pcm.extend(silenzio(5));
        pcm.extend(suono(3277, 50));
        pcm.extend(silenzio(5)); // in fondo: non seguito da suono
        assert!(analizza(&pcm).zeri.is_empty());
    }

    #[test]
    fn taglio_netto() {
        let mut pcm = suono(3277, 100);
        pcm.extend(suono(10, 40)); // circa −70 dB, non zeri esatti
        pcm.extend(suono(3277, 100));
        let a = analizza(&pcm);
        assert_eq!(a.tagli, vec![Tratto { inizio: 100 * MS, durata: 40 * MS }]);
        assert!(a.zeri.is_empty());
    }

    #[test]
    fn dissolvenza_e_pausa_breve_non_sono_tagli() {
        // Discesa di 5 dB al millisecondo: da −20 a −70 dB in 10 ms.
        let mut pcm = suono(3277, 50);
        for passo in 1..=10 {
            let a = (3277.0 * 10f64.powf(-0.25 * passo as f64)) as i16;
            pcm.extend(suono(a.max(1), 1));
        }
        pcm.extend(suono(1, 50));
        pcm.extend(suono(3277, 50));
        // Caduta netta ma solo 20 ms sotto −50 dB.
        pcm.extend(suono(10, 20));
        pcm.extend(suono(3277, 50));
        assert!(analizza(&pcm).tagli.is_empty());
    }

    #[test]
    fn wav_e_campioni() {
        let h = intestazione_wav(4 * 48_000);
        assert_eq!(&h[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(h[4..8].try_into().unwrap()), 36 + 192_000);
        assert_eq!(u32::from_le_bytes(h[24..28].try_into().unwrap()), 48_000);
        assert_eq!(u32::from_le_bytes(h[28..32].try_into().unwrap()), 192_000);
        assert_eq!(u32::from_le_bytes(h[40..44].try_into().unwrap()), 192_000);
        assert_eq!(campioni(&[0x01, 0x00, 0xff, 0xff, 0x00, 0x80]), vec![1, -1, i16::MIN]);
    }
}
