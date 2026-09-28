//! Servizio `shell,v2,raw:` di ADB: un processo senza terminale, con ingresso,
//! uscita e uscita d'errore separati e il codice d'uscita (memoria/studio/sistema.md
//! §1.2). Serve ad avviare il servizio di Phonestra (`crate::componente`).
//!
//! Sul canale viaggiano pacchetti `id u8 · lunghezza u32 LE · dati`: 0 ingresso,
//! 1 uscita, 2 errori, 3 codice d'uscita (1 byte), 4 chiusura dell'ingresso.
//! Quando il canale si chiude, adbd manda SIGHUP al processo.

use anyhow::{Context, Result};

use super::{Adb, Canale, Chiusore};

const INGRESSO: u8 = 0;
const USCITA: u8 = 1;
const ERRORI: u8 = 2;
const CODICE: u8 = 3;
const CHIUDI_INGRESSO: u8 = 4;

/// Quello che arriva dal processo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evento {
    Uscita(Vec<u8>),
    Errori(Vec<u8>),
    /// Il processo è terminato con questo codice (128 + segnale se ucciso).
    Codice(u8),
}

/// Un processo avviato con `shell,v2,raw:`.
pub struct ShellV2 {
    canale: Canale,
    ricevuti: Vec<u8>,
}

impl ShellV2 {
    /// Avvia `comando` (eseguito da `sh -c` sul telefono).
    pub async fn avvia(adb: &Adb, comando: &str) -> Result<Self> {
        let canale = adb.apri(&format!("shell,v2,raw:{comando}")).await.context("avvio con shell,v2")?;
        Ok(Self { canale, ricevuti: Vec::new() })
    }

    /// Scrive sull'ingresso del processo.
    pub async fn scrivi(&mut self, dati: &[u8]) -> Result<()> {
        self.canale.scrivi(&pacchetto(INGRESSO, dati)).await
    }

    /// Chiude l'ingresso del processo (fine del file per chi lo legge).
    pub async fn chiudi_ingresso(&mut self) -> Result<()> {
        self.canale.scrivi(&pacchetto(CHIUDI_INGRESSO, &[])).await
    }

    /// Il prossimo evento; `None` quando il canale è chiuso.
    pub async fn leggi(&mut self) -> Option<Evento> {
        loop {
            while let Some((id, dati)) = estrai(&mut self.ricevuti) {
                match id {
                    USCITA => return Some(Evento::Uscita(dati)),
                    ERRORI => return Some(Evento::Errori(dati)),
                    CODICE => return Some(Evento::Codice(dati.first().copied().unwrap_or(0))),
                    _ => {}
                }
            }
            let blocco = self.canale.leggi().await?;
            self.ricevuti.extend_from_slice(&blocco);
        }
    }

    /// Per chiudere il canale (e mandare SIGHUP al processo) da un altro compito.
    pub fn chiusore(&self) -> Chiusore {
        self.canale.chiusore()
    }
}

/// Un pacchetto del protocollo shell v2.
pub fn pacchetto(id: u8, dati: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(5 + dati.len());
    v.push(id);
    v.extend_from_slice(&(dati.len() as u32).to_le_bytes());
    v.extend_from_slice(dati);
    v
}

/// Toglie da `ricevuti` il primo pacchetto completo, se c'è.
pub fn estrai(ricevuti: &mut Vec<u8>) -> Option<(u8, Vec<u8>)> {
    if ricevuti.len() < 5 {
        return None;
    }
    let lunghezza = u32::from_le_bytes(ricevuti[1..5].try_into().unwrap()) as usize;
    if ricevuti.len() < 5 + lunghezza {
        return None;
    }
    let id = ricevuti[0];
    let dati = ricevuti[5..5 + lunghezza].to_vec();
    ricevuti.drain(..5 + lunghezza);
    Some((id, dati))
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn pacchetti_interi_e_a_pezzi() {
        let mut flusso = pacchetto(USCITA, b"pronto\n");
        flusso.extend(pacchetto(ERRORI, b"avviso"));
        flusso.extend(pacchetto(CODICE, &[3]));
        // Arrivano a pezzi di 3 byte, come possono arrivare i blocchi ADB.
        let mut ricevuti = Vec::new();
        let mut letti = Vec::new();
        for pezzo in flusso.chunks(3) {
            ricevuti.extend_from_slice(pezzo);
            while let Some(p) = estrai(&mut ricevuti) {
                letti.push(p);
            }
        }
        assert_eq!(letti, vec![(USCITA, b"pronto\n".to_vec()), (ERRORI, b"avviso".to_vec()), (CODICE, vec![3])]);
        assert!(ricevuti.is_empty());
    }

    #[test]
    fn intestazione() {
        assert_eq!(pacchetto(INGRESSO, b"ab"), vec![0, 2, 0, 0, 0, b'a', b'b']);
        assert_eq!(pacchetto(CHIUDI_INGRESSO, &[]), vec![4, 0, 0, 0, 0]);
    }
}
