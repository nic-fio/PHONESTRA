// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Misura del trasporto ADB (`phonestra-prova throughput`, `notes/adb.md`):
//! velocità di un canale carico, pause tra i blocchi e, a richiesta, latenza
//! di piccoli messaggi su un secondo canale mentre il primo è pieno (è quello
//! che vivrebbero l'audio e i comandi dietro al video).
//!
//! Il carico è `head -c <N> /dev/zero` sul telefono, avviato con
//! `shell,v2,raw:` (socketpair, niente terminale di mezzo) o, con `--exec`,
//! con `exec:` (terminale in modo raw, com'è oggi il componente). La latenza
//! è l'andata e ritorno di 8 byte attraverso `cat` (`shell,v2,raw:cat`).

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};

use super::{Adb, Canale, OpzioniCanale, Trasporto, flusso};

/// Pacchetti del protocollo shell v2: `id u8 · lunghezza u32 LE · dati`.
pub const STDIN: u8 = 0;
pub const STDOUT: u8 = 1;
pub const STDERR: u8 = 2;
pub const USCITA: u8 = 3;

/// Soglia oltre la quale una pausa tra blocchi si conta a parte: a 50 ms un
/// buffer audio di 100–200 ms regge ancora, oltre comincia a rischiare.
const PAUSA_LUNGA: Duration = Duration::from_millis(50);
/// Intervallo tra le misure di latenza durante il carico.
const OGNI: Duration = Duration::from_millis(50);

/// Un pacchetto shell v2 da mandare.
pub fn pacchetto_shell2(id: u8, dati: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(5 + dati.len());
    v.push(id);
    v.extend_from_slice(&(dati.len() as u32).to_le_bytes());
    v.extend_from_slice(dati);
    v
}

/// Ricompone i pacchetti shell v2 da blocchi ADB qualsiasi.
#[derive(Default)]
pub struct LettoreShell2 {
    resto: Vec<u8>,
}

impl LettoreShell2 {
    pub fn aggiungi(&mut self, blocco: &[u8]) -> Vec<(u8, Vec<u8>)> {
        self.resto.extend_from_slice(blocco);
        let mut pacchetti = Vec::new();
        let mut inizio = 0;
        while self.resto.len() - inizio >= 5 {
            let lunghezza = u32::from_le_bytes(self.resto[inizio + 1..inizio + 5].try_into().unwrap()) as usize;
            if self.resto.len() - inizio - 5 < lunghezza {
                break;
            }
            let id = self.resto[inizio];
            pacchetti.push((id, self.resto[inizio + 5..inizio + 5 + lunghezza].to_vec()));
            inizio += 5 + lunghezza;
        }
        self.resto.drain(..inizio);
        pacchetti
    }
}

/// Opzioni del comando `throughput`.
#[derive(Debug, Clone, PartialEq)]
pub struct Opzioni {
    /// Byte da far mandare al telefono.
    pub byte: u64,
    pub trasporto: Trasporto,
    /// Finestra del solo canale carico (altrimenti quella del trasporto).
    pub finestra_carico: Option<u32>,
    pub alla_lettura: bool,
    pub exec: bool,
    pub latenza: bool,
}

pub const USO: &str = "uso: phonestra-prova throughput [MB] [--senza-delayed-ack] [--payload N] [--finestra N] \
    [--latenza] [--exec] [--alla-lettura]   (N in byte, o 64k, 1m)";

impl Opzioni {
    pub fn da_argomenti(argomenti: &[String]) -> Result<Self> {
        let mut o = Self {
            byte: 100 * 1_000_000,
            trasporto: Trasporto::default(),
            finestra_carico: None,
            alla_lettura: false,
            exec: false,
            latenza: false,
        };
        let mut i = argomenti.iter();
        while let Some(a) = i.next() {
            let mut valore = |nome: &str| {
                i.next()
                    .and_then(|v| flusso::dimensione(v))
                    .ok_or_else(|| anyhow!("{nome} vuole una dimensione (byte, 64k, 1m)\n{USO}"))
            };
            match a.as_str() {
                "--senza-delayed-ack" => o.trasporto.delayed_ack = false,
                "--payload" => o.trasporto.max_payload = valore("--payload")?,
                "--finestra" => o.finestra_carico = Some(valore("--finestra")?),
                "--latenza" => o.latenza = true,
                "--exec" => o.exec = true,
                "--alla-lettura" => o.alla_lettura = true,
                mb => match mb.parse::<f64>() {
                    Ok(mb) if mb > 0.0 => o.byte = (mb * 1_000_000.0) as u64,
                    _ => bail!("argomento sconosciuto «{mb}»\n{USO}"),
                },
            }
        }
        o.trasporto = o.trasporto.normalizzato();
        Ok(o)
    }
}

/// Minimo, mediana, 95° percentile e massimo.
pub fn riassunto(valori: &mut [Duration]) -> Option<(Duration, Duration, Duration, Duration)> {
    if valori.is_empty() {
        return None;
    }
    valori.sort();
    let n = valori.len();
    let p95 = valori[((n * 95).div_ceil(100)).clamp(1, n) - 1];
    Some((valori[0], valori[n / 2], p95, valori[n - 1]))
}

fn ms(d: Duration) -> String {
    format!("{:.1} ms", d.as_secs_f64() * 1000.0)
}

fn stampa_latenze(nome: &str, valori: &mut [Duration], persi: usize) {
    match riassunto(valori) {
        Some((min, mediana, p95, max)) => println!(
            "{nome}: {} misure, min {} · mediana {} · 95% {} · max {}{}",
            valori.len(),
            ms(min),
            ms(mediana),
            ms(p95),
            ms(max),
            if persi > 0 { format!(" · {persi} senza risposta") } else { String::new() }
        ),
        None => println!("{nome}: nessuna misura"),
    }
}

/// Canale `cat` per le misure di latenza.
struct Eco {
    canale: Canale,
    lettore: LettoreShell2,
    numero: u64,
}

impl Eco {
    async fn apri(adb: &Adb) -> Result<Self> {
        let canale = adb.apri("shell,v2,raw:cat").await.context("apertura di cat per la latenza")?;
        Ok(Self { canale, lettore: LettoreShell2::default(), numero: 0 })
    }

    /// Andata e ritorno di 8 byte.
    async fn misura(&mut self) -> Result<Duration> {
        self.numero += 1;
        let atteso = self.numero.to_le_bytes();
        let inizio = Instant::now();
        self.canale.scrivi(&pacchetto_shell2(STDIN, &atteso)).await?;
        let mut ricevuti = Vec::new();
        while ricevuti.len() < atteso.len() {
            let blocco = self.canale.leggi().await.ok_or_else(|| anyhow!("cat si è chiuso"))?;
            for (id, dati) in self.lettore.aggiungi(&blocco) {
                if id == STDOUT {
                    ricevuti.extend_from_slice(&dati);
                }
            }
        }
        if ricevuti[..8] != atteso {
            bail!("l'eco non corrisponde");
        }
        Ok(inizio.elapsed())
    }

    async fn serie(&mut self, quante: usize, ogni: Duration, fine: Option<&AtomicBool>) -> (Vec<Duration>, usize) {
        let (mut valori, mut persi) = (Vec::new(), 0);
        for _ in 0..quante {
            if fine.is_some_and(|f| f.load(Ordering::Relaxed)) {
                break;
            }
            match tokio::time::timeout(Duration::from_secs(5), self.misura()).await {
                Ok(Ok(d)) => valori.push(d),
                Ok(Err(e)) => {
                    eprintln!("latenza: {e:#}");
                    persi += 1;
                    break;
                }
                // Il blocco annullato potrebbe arrivare dopo: meglio fermarsi.
                Err(_) => {
                    persi += 1;
                    break;
                }
            }
            tokio::time::sleep(ogni).await;
        }
        (valori, persi)
    }
}

/// Esegue la misura e stampa il risultato.
pub async fn esegui(indirizzo: SocketAddr, chiave: &Path, opzioni: &Opzioni) -> Result<()> {
    let adb = Adb::wifi_con(indirizzo, chiave, opzioni.trasporto).await?;
    let offre = flusso::offre_delayed_ack(&adb.dispositivo);
    println!(
        "Telefono: delayed_ack {} · attivo: {} · blocchi fino a {} byte · finestra {} byte{}",
        if offre { "offerto" } else { "NON offerto" },
        if adb.delayed_ack() { "sì" } else { "no" },
        adb.max_dati(),
        opzioni.finestra_carico.unwrap_or(adb.trasporto().finestra),
        if opzioni.alla_lettura && adb.delayed_ack() { " · conferma alla lettura" } else { "" },
    );

    let mut eco = if opzioni.latenza { Some(Eco::apri(&adb).await?) } else { None };
    if let Some(e) = eco.as_mut() {
        let (mut valori, persi) = e.serie(20, Duration::from_millis(20), None).await;
        stampa_latenze("latenza a riposo", &mut valori, persi);
    }
    let fine = Arc::new(AtomicBool::new(false));
    let latenze = eco.map(|mut e| {
        let fine = fine.clone();
        tokio::spawn(async move {
            let esito = e.serie(usize::MAX, OGNI, Some(&fine)).await;
            let _ = e.canale.chiudi().await;
            esito
        })
    });

    let comando = format!("head -c {} /dev/zero", opzioni.byte);
    let servizio = if opzioni.exec { format!("exec:{comando}") } else { format!("shell,v2,raw:{comando}") };
    let canale_opzioni =
        OpzioniCanale { finestra: opzioni.finestra_carico, conferma_alla_lettura: opzioni.alla_lettura };
    let inizio = Instant::now();
    let mut carico = adb.apri_con(&servizio, canale_opzioni).await?;
    let (mut primo, mut ultimo) = (None::<Instant>, None::<Instant>);
    let (mut ricevuti, mut blocchi, mut pause_lunghe) = (0u64, 0u64, 0u64);
    let mut pausa_massima = Duration::ZERO;
    let mut lettore = LettoreShell2::default();
    let mut errori = Vec::new();
    let mut prossimo_avviso = Instant::now() + Duration::from_secs(1);
    while let Some(blocco) = carico.leggi().await {
        let adesso = Instant::now();
        primo.get_or_insert(adesso);
        if let Some(prima) = ultimo {
            let pausa = adesso - prima;
            pausa_massima = pausa_massima.max(pausa);
            if pausa >= PAUSA_LUNGA {
                pause_lunghe += 1;
            }
        }
        ultimo = Some(adesso);
        blocchi += 1;
        if opzioni.exec {
            ricevuti += blocco.len() as u64;
        } else {
            for (id, dati) in lettore.aggiungi(&blocco) {
                match id {
                    STDOUT => ricevuti += dati.len() as u64,
                    STDERR => errori.extend_from_slice(&dati),
                    _ => {}
                }
            }
        }
        if adesso >= prossimo_avviso {
            eprint!("\r{:.0} %  ", ricevuti as f64 * 100.0 / opzioni.byte as f64);
            prossimo_avviso = adesso + Duration::from_secs(1);
        }
    }
    eprint!("\r        \r");
    fine.store(true, Ordering::Relaxed);
    let totale = inizio.elapsed();

    if !errori.is_empty() {
        println!("errori del telefono: {}", String::from_utf8_lossy(&errori).trim_end());
    }
    let mb = ricevuti as f64 / 1_000_000.0;
    let dal_primo = primo.zip(ultimo).map(|(p, u)| u - p).unwrap_or_default();
    println!(
        "{mb:.1} MB in {:.2} s: {:.1} MB/s ({:.0} Mbit/s) · dal primo byte {:.1} MB/s · primo byte dopo {}",
        totale.as_secs_f64(),
        mb / totale.as_secs_f64(),
        mb * 8.0 / totale.as_secs_f64(),
        if dal_primo.is_zero() { 0.0 } else { mb / dal_primo.as_secs_f64() },
        primo.map(|p| ms(p - inizio)).unwrap_or_else(|| "—".into()),
    );
    println!(
        "{blocchi} blocchi (media {} byte) · pausa massima tra blocchi {} · pause ≥ {}: {pause_lunghe}",
        ricevuti.checked_div(blocchi).unwrap_or(0),
        ms(pausa_massima),
        ms(PAUSA_LUNGA),
    );
    if ricevuti < opzioni.byte {
        println!("ATTENZIONE: attesi {} byte, arrivati {ricevuti}", opzioni.byte);
    }
    if let Some(compito) = latenze {
        let (mut valori, persi) = compito.await?;
        stampa_latenze("latenza sotto carico", &mut valori, persi);
    }
    Ok(())
}

#[cfg(test)]
mod prove {
    use super::*;

    fn argomenti(testo: &str) -> Vec<String> {
        testo.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn pacchetti_shell2_a_pezzi() {
        let mut flusso = pacchetto_shell2(STDOUT, b"ciao");
        flusso.extend(pacchetto_shell2(STDERR, b""));
        flusso.extend(pacchetto_shell2(USCITA, &[0]));
        let mut l = LettoreShell2::default();
        let mut tutti = Vec::new();
        // Un byte alla volta: i pacchetti escono solo quando sono interi.
        for b in &flusso {
            tutti.extend(l.aggiungi(std::slice::from_ref(b)));
        }
        assert_eq!(tutti, vec![(STDOUT, b"ciao".to_vec()), (STDERR, vec![]), (USCITA, vec![0])]);
        assert_eq!(LettoreShell2::default().aggiungi(&flusso).len(), 3);
    }

    #[test]
    fn argomenti_del_comando() {
        let o = Opzioni::da_argomenti(&[]).unwrap();
        assert_eq!(o.byte, 100_000_000);
        assert_eq!(o.trasporto, Trasporto::default());
        let o = Opzioni::da_argomenti(&argomenti("50 --senza-delayed-ack --payload 1m --latenza --exec")).unwrap();
        assert_eq!(o.byte, 50_000_000);
        assert!(!o.trasporto.delayed_ack && o.latenza && o.exec && !o.alla_lettura);
        assert_eq!(o.trasporto.max_payload, 1024 * 1024);
        let o = Opzioni::da_argomenti(&argomenti("0.5 --finestra 64k --alla-lettura --payload 100")).unwrap();
        assert_eq!(o.byte, 500_000);
        assert_eq!(o.finestra_carico, Some(65_536));
        assert!(o.alla_lettura);
        // Sotto il minimo del protocollo si torna a 4 KiB.
        assert_eq!(o.trasporto.max_payload, 4096);
        assert!(Opzioni::da_argomenti(&argomenti("--payload")).is_err());
        assert!(Opzioni::da_argomenti(&argomenti("--boh")).is_err());
        assert!(Opzioni::da_argomenti(&argomenti("-3")).is_err());
    }

    #[test]
    fn riassunto_delle_latenze() {
        assert!(riassunto(&mut []).is_none());
        let mut v: Vec<Duration> = (1..=100).rev().map(Duration::from_millis).collect();
        let (min, mediana, p95, max) = riassunto(&mut v).unwrap();
        assert_eq!(
            (min, mediana, p95, max),
            (
                Duration::from_millis(1),
                Duration::from_millis(51),
                Duration::from_millis(95),
                Duration::from_millis(100)
            )
        );
        let mut uno = [Duration::from_millis(7)];
        assert_eq!(riassunto(&mut uno).unwrap().2, Duration::from_millis(7));
    }
}
